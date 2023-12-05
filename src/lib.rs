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
    use core::marker::PhantomData;

    use codec::{FullCodec, FullEncode};
    use frame_support::{
        dispatch::{DispatchResultWithPostInfo, PostDispatchInfo},
        pallet_prelude::{Pays, *},
        traits::{Currency, ExistenceRequirement, Hooks, ReservableCurrency},
    };
    use frame_system::pallet_prelude::{BlockNumberFor, *};
    use move_core_types::{account_address::AccountAddress, value::MoveValue};
    use move_vm_backend::{
        deposit::{CORE_CODE_ADDRESS, MOVE_DEPOSIT_MODULE_BYTES, SIGNER_MODULE_BYTES},
        CompiledScript, Mvm, SignatureToken, SubstrateAPI, TransferError,
    };
    use move_vm_types::gas::UnmeteredGasMeter;
    use sp_core::{blake2_128, crypto::AccountId32};
    use sp_runtime::{DispatchResult, SaturatedConversion};
    use sp_std::{default::Default, vec::Vec};
    use transaction::Transaction;

    use super::*;
    use crate::storage::MoveVmStorage;

    /// Reports if module publish succedded or failed
    #[derive(Debug, Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
    pub enum ModulePublishStatus {
        Success,
        Failure(String),
    }

    /// Reports of script execution succedded or failed
    #[derive(Debug, Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
    pub enum ScriptExecutionStatus {
        Success,
        Failure(String),
    }

    #[pallet::pallet]
    #[pallet::without_storage_info] // Allows to define storage items without fixed size
    pub struct Pallet<T>(_);

    /// Storage item for MoveVM pallet - runtime storage
    /// Key-value map, where both key and value are vectors of bytes.
    /// Key is an access path (Move address), and a value is a Move resource.
    #[pallet::storage]
    pub type VMStorage<T> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>>;

    /// Published modules to be processed by `Mvm` instance
    /// Picked up one by one on `offchain_worker` execution
    /// Report of execution is done by emitting `Event::PublishModuleResult{ publisher, module, status }`
    #[pallet::storage]
    pub type ModulesToPublish<T: Config> =
        StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, u128, Vec<u8>>;

    /// Set of submitted scripts for execution by `Mvm` instance
    /// Picked up one by one on `offchain_worker` execution
    /// Report of execution is done by emitting `Event::ScriptExecutionResult { publisher, script, status }`
    #[pallet::storage]
    pub type ScriptsToExecute<T: Config> =
        StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, u128, Vec<u8>>;

    /// MoveVM pallet configuration trait
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The currency mechanism.
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Type representing the weight of this pallet
        type WeightInfo: WeightInfo;

        /// Government origin - allowes `MoveVM std` updates
        type GoverningOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;
    }

    // Pallets use events to inform users when important changes are made.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event about calling execute function.
        /// [account]
        ScriptScheduledForExecution { who: T::AccountId, script_id: u128 },

        /// Event about successful move-module publishing
        /// [account]
        /// blake2_128 hash of submitted bytecode Big Endian encoded into u128
        ModulePublished { who: T::AccountId, module_id: u128 },

        /// Event emitted by `offline_client` which allows module publishing execution monitoring
        /// * publisher - account of publish_module extrinsict caller.
        /// * module - u128 hash ID of module data
        /// * status - `modulePublishStatus` indicating result of operation
        PublishModuleResult {
            publisher: T::AccountId,
            module: u128,
            status: ModulePublishStatus,
        },

        /// Event emitted by `offline_client` which allows script execution monitoring
        /// * publisher - account of publish_module extrinsict caller.
        /// * script - u128 hash ID of script data
        /// * status - `modulePublishStatus` indicating result of operation
        ExecuteScriptResult {
            publisher: T::AccountId,
            script: u128,
            status: ScriptExecutionStatus,
        },

        /// Event about successful move-package published
        /// [account]
        PackagePublished { who: T::AccountId },

        /// Failed to clean up all the remaining session transfer permissions
        /// * diff - how many of permissions remained undeleted
        SessionTransferTokenCleanupFailed { diff: u32 },

        /// Failed to publish DepositModule from offchain_worker
        StdModulePublishFailed(String),

        /// Successfuly published DepositModule from offchain_worker
        StdModulePublished,
    }

    // Pallet hook[s] implementations
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // Offchain worker with Mvm initialization
        fn offchain_worker(_block_number: BlockNumberFor<T>) {
            let storage = Self::move_vm_storage();

            let vm = Mvm::new(storage, Gw::<T>::new(PhantomData)).unwrap();
            // inject std
            if let Err(e) = vm.publish_module(
                SIGNER_MODULE_BYTES.as_ref(),
                CORE_CODE_ADDRESS,
                &mut UnmeteredGasMeter {},
            ) {
                Self::deposit_event(Event::StdModulePublishFailed(e.to_string()));
                return; // can not proceed without deposit module injected
            }
            Self::deposit_event(Event::StdModulePublished);
            // inject deposit
            if let Err(e) = vm.publish_module(
                MOVE_DEPOSIT_MODULE_BYTES.as_ref(),
                CORE_CODE_ADDRESS,
                &mut UnmeteredGasMeter {},
            ) {
                Self::deposit_event(Event::StdModulePublishFailed(e.to_string()));
                return; // can not proceed without deposit module injected
            }
            Self::deposit_event(Event::StdModulePublished);

            // Processing users modules to be published
            ModulesToPublish::<T>::drain().for_each(|(account, id, module)| {
                if let Err(reason) = vm.publish_module(
                    &module,
                    Self::native_to_move(&account).unwrap(), //FIXME: safe to unwrap?
                    &mut UnmeteredGasMeter {},
                ) {
                    // report failure
                    Self::deposit_event(Event::PublishModuleResult {
                        publisher: account,
                        module: id,
                        status: ModulePublishStatus::Failure(reason.to_string()),
                    });
                } else {
                    // report success
                    Self::deposit_event(Event::PublishModuleResult {
                        publisher: account,
                        module: id,
                        status: ModulePublishStatus::Success,
                    });
                }
            });

            // Executing submitted scripts
            ScriptsToExecute::<T>::drain().for_each(|(account, id, script)| {
                match Transaction::try_from(script.as_slice()) {
                    Ok(transaction) => {
                        if let Err(reason) = vm.execute_script(
                            &transaction.script_bc,
                            transaction.type_args,
                            transaction.args.iter().map(|x| x.as_slice()).collect(),
                            &mut UnmeteredGasMeter {},
                        ) {
                            Self::deposit_event(Event::ExecuteScriptResult {
                                publisher: account,
                                script: id,
                                status: ScriptExecutionStatus::Failure(reason.to_string()),
                            });
                        } else {
                            Self::deposit_event(Event::ExecuteScriptResult {
                                publisher: account,
                                script: id,
                                status: ScriptExecutionStatus::Success,
                            });
                        }
                    }
                    Err(reason) => {
                        Self::deposit_event(Event::ExecuteScriptResult {
                            publisher: account,
                            script: id,
                            status: ScriptExecutionStatus::Failure(reason.to_string()),
                        });
                    }
                }
            });
        }
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Execute Move script bytecode sent by the user.
        /// Indicating `transfers` as `true` is more expensive but
        /// will prepand given parameters with origin as `Signer` preventing unauthorized token transfers
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::execute())]
        pub fn execute(
            origin: OriginFor<T>,
            transaction_bc: Vec<u8>,
            transfers: bool,
            _gas_limit: u64,
        ) -> DispatchResult {
            // Allow only signed calls.
            let who = ensure_signed(origin)?;
            let script_id = Self::get_id(&transaction_bc);
            let mut transaction = Transaction::try_from(transaction_bc.as_ref())
                .map_err(|_| Error::<T>::ExecuteFailed)?;
            let loaded_script = CompiledScript::deserialize(&transaction.script_bc)
                .map_err(|_| Error::<T>::ExecuteFailed)?;
            // make sure no `Signer` is injected into the script without signatures for security reasons
            // FIXME: are TypeParameter, Struct() and StructInstantiation also able to become Signer?
            if loaded_script
                .signatures
                .iter()
                // we allow first signer if transfers, reject otherwise
                .flat_map(|s| &s.0)
                .skip(if transfers { 1 } else { 0 })
                .any(Self::contains_signer)
            {
                return Err(Error::<T>::ExecuteFailed.into());
            }
            if transfers {
                transaction.args.reverse();
                // replace first one with proper signer regardless of what was given
                drop(transaction.args.pop());
                transaction.args.push(
                    bcs::to_bytes(&MoveValue::Signer(Self::native_to_move(&who)?))
                        .map_err(|_| Error::<T>::InvalidAccountSize)?,
                );
                transaction.args.reverse();
                ScriptsToExecute::<T>::insert(
                    who.clone(),
                    script_id,
                    bcs::to_bytes(&transaction).map_err(|_| Error::<T>::ExecuteFailed)?,
                );
            } else {
                // no signer - do as you please
                ScriptsToExecute::<T>::insert(who.clone(), script_id, transaction_bc);
            }
            // Emit an event.
            Self::deposit_event(Event::ScriptScheduledForExecution { who, script_id });
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

            // TODO: can we check if given bytecode is actually Move module?
            // FIXME: define size checks
            let module_id = Self::get_id(&bytecode);
            ModulesToPublish::<T>::insert(who.clone(), module_id, bytecode);

            // Emit an event.
            Self::deposit_event(Event::ModulePublished { who, module_id });

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

        /// Publish a Move 'std' module sent by the governing user[s].
        /// Module is published under `std`'s address.
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::update_std())]
        pub fn update_std(origin: OriginFor<T>, module: Vec<u8>) -> DispatchResultWithPostInfo {
            // make sure it's governance
            T::GoverningOrigin::ensure_origin(origin)?;
            // schedule `std` update
            ModulesToPublish::<T>::insert(
                Self::move_to_native(
                    &AccountAddress::from_hex_literal("0x01")
                        .map_err(|_| Error::<T>::InvalidAccountSize)?,
                )?,
                Self::get_id(&module),
                module,
            );
            // if extrinsic is actually from governing origin and everything executed successfuly - not paying for it
            Ok(Pays::No.into())
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
        #[allow(clippy::type_complexity)]
        fn move_vm() -> Result<Mvm<crate::storage::StorageAdapter<VMStorage<T>>, Gw<T>>, Vec<u8>> {
            let storage = Self::move_vm_storage();

            Mvm::new(storage, Gw::new(PhantomData)).map_err::<Vec<u8>, _>(|err| {
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

        // generates u128 id from set of bytes using blake_2_128 hash
        pub fn get_id(data: impl AsRef<[u8]>) -> u128 {
            u128::from_be_bytes(blake2_128(data.as_ref()))
        }

        fn contains_signer(v: &SignatureToken) -> bool {
            match v {
                SignatureToken::Signer => true,
                SignatureToken::Vector(v) => Self::contains_signer(&v),
                SignatureToken::Reference(v) => Self::contains_signer(&v),
                SignatureToken::MutableReference(v) => Self::contains_signer(&v),
                // FIXME: have to check all StructHandle->Abilities for `SIGNER`
                SignatureToken::Struct(v) => {
                    Mvm::new(Self::move_vm_storage(), Gw::<T>::new(PhantomData))
                        .unwrap() // FIXME: ?
                        .get_struct_members(*v)
                        .iter()
                        .all(Self::contains_signer)
                }
                SignatureToken::StructInstantiation(_, v) => v.iter().all(Self::contains_signer),
                _ => false,
            }
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
            &self,
            from: AccountAddress,
            to: AccountAddress,
            amount: u128,
        ) -> Result<(), TransferError> {
            // TODO: add conversion error
            let from = Pallet::<T>::move_to_native(&from)
                .map_err(|_| TransferError::InsuficientBalance)?;
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
            Ok(())
        }

        fn get_balance(&self, of: AccountAddress) -> u128 {
            Pallet::<T>::get_move_balance(&of).unwrap_or(0)
        }
    }
}
