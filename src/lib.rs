#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod address;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod storage;

pub mod weights;
pub use weights::*;

// The pallet is defined below.
#[frame_support::pallet]
pub mod pallet {
    #[cfg(not(feature = "std"))]
    extern crate alloc;
    #[cfg(not(feature = "std"))]
    use alloc::format;
    use core::marker::PhantomData;

    use arrayref::array_ref;
    use codec::{FullCodec, FullEncode};
    use frame_support::{
        dispatch::{DispatchResultWithPostInfo, PostDispatchInfo},
        pallet_prelude::*,
        traits::{Currency, ExistenceRequirement, Hooks, ReservableCurrency},
        Blake2_128Concat,
    };
    use frame_system::pallet_prelude::{BlockNumberFor, *};
    use move_core_types::account_address::AccountAddress;
    use move_vm_backend::{Mvm, SubstrateAPI, TransferError};
    use move_vm_types::gas::UnmeteredGasMeter;
    use sp_core::crypto::AccountId32;
    use sp_runtime::{DispatchResult, SaturatedConversion};
    use sp_std::{default::Default, vec::Vec};

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

    /// Storage of session/block allowed transfer executions
    /// Maps binary code to sender's account which is expected to be the source of the transfer
    #[pallet::storage]
    pub(super) type SessionTransferToken<T: Config> =
        StorageMap<_, Blake2_128Concat, Vec<u8>, T::AccountId>;

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

        /// Failed to clean up all the remaining session transfer permissions
        /// * diff - how many of permissions remained undeleted
        SessionTransferTokenCleanupFailed { diff: u32 },
    }

    // Pallet hook[s] implementations
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(_now: BlockNumberFor<T>) {
            let unused: u32 = <SessionTransferToken<T>>::iter().count().saturated_into();
            // if we failed to use permission to transfer tokens
            // TODO: make payed execute a separate extrinsic?
            if unused > 0 {
                // remove everything
                let deleted =
                    <SessionTransferToken<T>>::clear(unused.saturated_into(), None).unique;
                // report if something failed to be cleaned up - potential security risk
                if unused > deleted {
                    Self::deposit_event(Event::SessionTransferTokenCleanupFailed {
                        diff: unused.saturating_sub(deleted),
                    })
                }
            }
        }
        // Offchain worker with Mvm initialization
        fn offchain_worker(_block_number: BlockNumberFor<T>) {
            let storage = Self::move_vm_storage();

            let vm = Mvm::new(storage, Gw::<T>::new(PhantomData::default())).unwrap();
        }
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Execute Move script bytecode sent by the user.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::execute())]
        pub fn execute(origin: OriginFor<T>, bytecode: Vec<u8>, _gas_limit: u64) -> DispatchResult {
            // Allow only signed calls.
            let who = ensure_signed(origin)?;
            // store token for this session execution
            <SessionTransferToken<T>>::insert(bytecode, who.clone());

            // TODO: Execute bytecode

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
            let vm = Mvm::new(storage, Gw::<T>::new(PhantomData::default()))
                .map_err(|_err| Error::<T>::PublishModuleFailed)?;
            let encoded = who.encode();

            ensure!(encoded.len().eq(&32), Error::<T>::InvalidAccountSize);

            vm.publish_module(
                bytecode.as_slice(),
                Self::native_to_move(AccountId32::new(array_ref![encoded, 0, 32].to_owned()))?,
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
        /// No token for signer account present for transfer execution in `SessionExecutionToken`
        TransferNotAllowed,
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
        fn move_vm() -> Result<Mvm<crate::storage::StorageAdapter<VMStorage<T>>, Gw<T>>, Vec<u8>> {
            let storage = Self::move_vm_storage();

            Mvm::new(storage, Gw::new(PhantomData::default())).map_err::<Vec<u8>, _>(|err| {
                format!("error while creating the vm {:?}", err).into()
            })
        }

        pub fn get_module_abi(
            address: &T::AccountId,
            name: &str,
        ) -> Result<Option<Vec<u8>>, Vec<u8>> {
            let vm = Self::move_vm()?;

            let address = address::to_move_address(&address);

            vm.get_module_abi(address, name)
                .map_err(|e| format!("error in get_module_abi: {:?}", e).into())
        }

        pub fn get_module(address: &T::AccountId, name: &str) -> Result<Option<Vec<u8>>, Vec<u8>> {
            let vm = Self::move_vm()?;

            let address = address::to_move_address(&address);

            vm.get_module(address, name)
                .map_err(|e| format!("error in get_module: {:?}", e).into())
        }

        pub fn get_resource(
            account: &T::AccountId,
            tag: &[u8],
        ) -> Result<Option<Vec<u8>>, Vec<u8>> {
            let vm = Self::move_vm()?;

            vm.get_resource(
                &AccountAddress::new(address::account_to_bytes(account)),
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
        pub fn native_to_move(of: AccountId32) -> Result<AccountAddress, Error<T>> {
            let account_bytes: [u8; 32] = of.into();
            Ok(AccountAddress::new(
                array_ref![account_bytes, 0, 32].to_owned(),
            ))
        }
    }

    /// Implementing structure for 'SubstrateApi'
    pub struct Gw<T: Config> {
        api: PhantomData<T>,
    }

    impl<T> Gw<T>
    where
        T: Config,
    {
        pub fn new(api: PhantomData<T>) -> Self {
            Self { api }
        }
    }
    // Substrate glue for Move VM interaction with the chain
    impl<T> SubstrateAPI for Gw<T>
    where
        T: Config,
    {
        fn transfer(
            from: AccountAddress,
            to: AccountAddress,
            amount: u128,
        ) -> Result<(), TransferError> {
            // TODO: add conversion error
            let from = Pallet::<T>::move_to_native(&from)
                .map_err(|_| TransferError::InsuficientBalance)?;
            // Verify there's a token
            if !SessionTransferToken::<T>::iter().any(|v| v.1.eq(&from)) {
                return Err(TransferError::NoSessionTokenPresent);
            }
            // TODO: add conversion error
            let to =
                Pallet::<T>::move_to_native(&to).map_err(|_| TransferError::InsuficientBalance)?;
            T::Currency::transfer(
                &from,
                &to,
                amount.saturated_into(),
                ExistenceRequirement::KeepAlive,
            )
            .map_err(|_| TransferError::InsuficientBalance)?;
            //TODO: clean up transfer token
            Ok(())
        }

        fn get_balance(of: AccountAddress) -> u128 {
            Pallet::<T>::get_move_balance(&of).unwrap_or(0)
        }
    }
}
