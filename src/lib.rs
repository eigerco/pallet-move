#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod storage;

pub mod transaction;

pub mod weights;
pub use weights::*;

// The pallet is defined below.
#[frame_support::pallet]
pub mod pallet {
    #[cfg(not(feature = "std"))]
    extern crate alloc;
    #[cfg(not(feature = "std"))]
    use alloc::format;

    use codec::{FullCodec, FullEncode};
    use frame_support::{
        dispatch::{DispatchResultWithPostInfo, PostDispatchInfo},
        pallet_prelude::*,
        traits::{Currency, ExistenceRequirement, ReservableCurrency},
    };
    use frame_system::pallet_prelude::*;
    use move_core_types::account_address::AccountAddress;
    use move_vm_backend::Mvm;
    use move_vm_types::gas::UnmeteredGasMeter;
    use sp_core::crypto::AccountId32;
    use sp_runtime::{DispatchResult, SaturatedConversion};
    use sp_std::{default::Default, vec::Vec};
    use transaction::Transaction;

    use super::*;
    use crate::storage::MoveVmStorage;

    #[pallet::pallet]
    #[pallet::without_storage_info] // Allows to define storage items without fixed size
    pub struct Pallet<T>(_);

    /// Storage item for MoveVM pallet - runtime storage
    /// Key-value map, where both key and value are vectors of bytes.
    /// Key is an access path (Move address), and a value is a Move resource.
    #[pallet::storage]
    pub type VMStorage<T> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>>;

    /// MoveVM pallet configuration trait
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The currency mechanism.
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Type representing the weight of this pallet
        type WeightInfo: WeightInfo;
    }

    // Pallets use events to inform users when important changes are made.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event about calling execute function.
        /// [account]
        ExecuteCalled { who: T::AccountId },

        /// Event about successful move-module publishing
        /// [account]
        ModulePublished { who: T::AccountId },

        /// Event about successful move-package published
        /// [account]
        PackagePublished { who: T::AccountId },
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Execute Move script bytecode sent by the user.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::execute())]
        pub fn execute(
            origin: OriginFor<T>,
            transaction_bc: Vec<u8>,
            _gas_limit: u64,
        ) -> DispatchResult {
            // Allow only signed calls.
            let who = ensure_signed(origin)?;

            let storage = Self::move_vm_storage();
            let vm = Mvm::new(storage).map_err(|_err| Error::<T>::ExecuteFailed)?;

            let transaction = Transaction::try_from(transaction_bc.as_slice())
                .map_err(|_| Error::<T>::ExecuteFailed)?;

            vm.execute_script(
                transaction.script_bc.as_slice(),
                transaction.type_args,
                transaction.args.iter().map(|x| x.as_slice()).collect(),
                &mut UnmeteredGasMeter, // TODO(asmie): gas handling
            )
            .map_err(|_err| Error::<T>::ExecuteFailed)?;

            // Emit an event.
            Self::deposit_event(Event::ExecuteCalled { who });

            // Return a successful DispatchResult
            Ok(())
        }

        /// Publish a Move module sent by the user.
        /// Module is published under its sender's address.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::publish_module())]
        pub fn publish_module(
            origin: OriginFor<T>,
            bytecode: Vec<u8>,
            _gas_limit: u64,
        ) -> DispatchResultWithPostInfo {
            // Allow only signed calls.
            let who = ensure_signed(origin)?;

            let storage = Self::move_vm_storage();

            //TODO(asmie): future work:
            // - put Mvm initialization to some other place, to avoid doing it every time
            // - Substrate address to Move address conversion is missing in the move-cli
            let vm = Mvm::new(storage).map_err(|_err| Error::<T>::PublishModuleFailed)?;

            vm.publish_module(
                bytecode.as_slice(),
                Self::native_to_move(&who)?,
                &mut UnmeteredGasMeter, // TODO(asmie): gas handling
            )
            .map_err(|_err| Error::<T>::PublishModuleFailed)?;

            // Emit an event.
            Self::deposit_event(Event::ModulePublished { who });

            // Return a successful DispatchResultWithPostInfo
            Ok(PostDispatchInfo::default())
        }

        /// Publish a Move module packages sent by the user.
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::publish_package())]
        pub fn publish_package(
            origin: OriginFor<T>,
            _package: Vec<u8>,
            _gas_limit: u64,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // TODO: Publish module package

            // Emit an event.
            Self::deposit_event(Event::PackagePublished { who });

            Ok(PostDispatchInfo::default())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::transfer())]
        pub fn transfer(
            origin: OriginFor<T>,
            recepient: [u8; 32], // AccountAddress
            amount: u128,
        ) -> DispatchResult {
            let from = ensure_signed(origin)?;
            let recepient_account = AccountAddress::new(recepient);
            T::Currency::transfer(
                &from,
                &Self::move_to_native(&recepient_account)?,
                amount.saturated_into(),
                ExistenceRequirement::KeepAlive,
            )
        }
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Error returned when executing Move script bytecode failed.
        ExecuteFailed,
        /// Error returned when publishing Move module failed.
        PublishModuleFailed,
        /// Error returned when publishing Move module package failed.
        PublishPackageFailed,
        /// Native balance to u128 conversion failed
        BalanceConversionFailed,
        /// Invalid account size (expected 32 bytes)
        InvalidAccountSize,
    }

    /// Prepare a storage adapter ready for the Virtual Machine.
    /// This declares the storage for the Pallet with the configuration T.
    impl<T: Config, K, V> MoveVmStorage<T, K, V> for Pallet<T>
    where
        K: FullEncode,
        V: FullCodec,
    {
        type VmStorage = VMStorage<T>;
    }

    impl<T: Config> Pallet<T> {
        // Internal helper for creating new MoveVM instance with StorageAdapter.
        fn move_vm() -> Result<Mvm<crate::storage::StorageAdapter<VMStorage<T>>>, Vec<u8>> {
            let storage = Self::move_vm_storage();

            Mvm::new(storage).map_err::<Vec<u8>, _>(|err| {
                format!("error while creating the vm {:?}", err).into()
            })
        }

        pub fn get_module_abi(
            address: &T::AccountId,
            name: &str,
        ) -> Result<Option<Vec<u8>>, Vec<u8>> {
            let vm = Self::move_vm()?;

            let address = Self::native_to_move(address).unwrap();

            vm.get_module_abi(address, name)
                .map_err(|e| format!("error in get_module_abi: {:?}", e).into())
        }

        pub fn get_module(address: &T::AccountId, name: &str) -> Result<Option<Vec<u8>>, Vec<u8>> {
            let vm = Self::move_vm()?;

            let address =
                Self::native_to_move(address).map_err(|_| "Invalid address size".as_bytes())?;

            vm.get_module(address, name)
                .map_err(|e| format!("error in get_module: {:?}", e).into())
        }

        pub fn get_resource(
            account: &T::AccountId,
            tag: &[u8],
        ) -> Result<Option<Vec<u8>>, Vec<u8>> {
            let vm = Self::move_vm()?;

            vm.get_resource(
                &Self::native_to_move(account).map_err(|_| "Invalid address size".as_bytes())?,
                tag,
            )
            .map_err(|e| format!("error in get_resource: {:?}", e).into())
        }

        /// Get balance of given account in native currency converted to u128
        pub fn get_balance(of: T::AccountId) -> u128 {
            T::Currency::free_balance(&of).saturated_into::<u128>()
        }

        // Get balance of given Move account in native currecy converted to u128
        pub fn get_move_balance(of: &AccountAddress) -> Result<u128, Error<T>> {
            Ok(Self::get_balance(Self::move_to_native(of)?))
        }

        // Transparent conversion move -> native
        pub fn move_to_native(of: &AccountAddress) -> Result<T::AccountId, Error<T>> {
            T::AccountId::decode(&mut of.as_ref()).map_err(|_| Error::InvalidAccountSize)
        }

        // Transparent conversion native -> move
        pub fn native_to_move(of: &T::AccountId) -> Result<AccountAddress, Error<T>> {
            let of = AccountId32::decode(&mut of.encode().as_ref())
                .map_err(|_| Error::InvalidAccountSize)?;
            let account_bytes: [u8; 32] = of.into();
            Ok(AccountAddress::new(account_bytes))
        }
    }
}
