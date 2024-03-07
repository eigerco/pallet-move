#![cfg_attr(not(feature = "std"), no_std)]

pub mod api;
pub mod balance;
#[cfg(all(test, feature = "runtime-benchmarks"))]
mod benchmarking;
#[cfg(test)]
mod mock;
mod result;
mod signer;
mod storage;
#[cfg(test)]
mod tests;
pub mod weights;

pub use pallet::*;
pub use weights::*;

#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: "runtime::pallet-move",
			concat!("[{:?}] 📄 ", $patter), <frame_system::Pallet<T>>::block_number() $(, $values)*
		)
	};
}

// The pallet is defined below.
#[frame_support::pallet]
pub mod pallet {

    #[cfg(not(feature = "std"))]
    extern crate alloc;
    #[cfg(not(feature = "std"))]
    use alloc::format;
    use blake2::{Blake2s256, Digest};
    use core::convert::AsRef;

    use codec::{FullCodec, FullEncode};
    use frame_support::{
        dispatch::DispatchResultWithPostInfo,
        pallet_prelude::*,
        traits::{Currency, Get, LockableCurrency, ReservableCurrency},
    };
    use frame_system::pallet_prelude::*;
    pub use move_core_types::language_storage::TypeTag;
    use move_core_types::{account_address::AccountAddress, language_storage::CORE_CODE_ADDRESS};
    pub use move_vm_backend::types::{GasAmount, GasStrategy};
    use move_vm_backend::{
        balance::BalanceHandler, genesis::VmGenesisConfig, types::VmResult, Mvm,
    };
    use move_vm_backend_common::abi::ModuleAbi;
    pub use move_vm_backend_common::{
        bytecode::verify_script_integrity_and_check_signers, types::ScriptTransaction,
    };
    use sp_core::crypto::AccountId32;
    use sp_std::{vec, vec::Vec};

    use super::*;
    use crate::{
        balance::{BalanceAdapter, BalanceOf},
        signer::*,
        storage::{MoveVmStorage, StorageAdapter},
    };

    type MvmResult<T> = Result<Mvm<StorageAdapter<VMStorage<T>>, BalanceAdapter<T>>, Vec<u8>>;

    /// Maximum number of multisig storage entries to be checked per block.
    const MAX_MULTISIG_CHECKING_PER_BLOCK: u64 = 20;

    #[pallet::pallet]
    #[pallet::without_storage_info] // Allows to define storage items without fixed size
    pub struct Pallet<T>(_);

    /// Storage item for MoveVM pallet - runtime storage
    /// Key-value map, where both key and value are vectors of bytes.
    /// Key is an access path (Move address), and a value is a Move resource.
    #[pallet::storage]
    pub type VMStorage<T> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>>;

    /// Storage for multi-signature/signer requests.
    #[pallet::storage]
    pub type MultisigStorage<T> = StorageMap<_, Blake2_128Concat, CallHash, MultisigOf<T>>;

    /// Storage for chore mechanism for old Multisigs in `MultisigStorage`.
    #[pallet::storage]
    pub type ChoreOnIdleStorage<T> = StorageValue<_, u64>;

    /// MoveVM pallet configuration trait
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The currency mechanism.
        type Currency: Currency<Self::AccountId>
            + ReservableCurrency<Self::AccountId>
            + LockableCurrency<Self::AccountId>;

        /// Maximum life time for requests.
        #[pallet::constant]
        type MaxLifetimeRequests: Get<BlockNumberFor<Self>>;

        /// Maximum number of signatories in multi-signer requests.
        #[pallet::constant]
        type MaxScriptSigners: Get<u32>;

        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Type representing the weight of this pallet
        type WeightInfo: WeightInfo;
    }

    // Pallets use events to inform users when important changes are made.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event about successful move-bundle published.
        /// [account]
        BundlePublished { who: T::AccountId },
        /// Event about calling execute function.
        /// [account]
        ExecuteCalled { who: Vec<T::AccountId> },
        /// Event about successful move-module publishing.
        /// [account]
        ModulePublished { who: T::AccountId },
        /// Event about removed multi-signing request.
        /// [vec<account>]
        MultiSignRequestRemoved { call: Vec<CallHash> },
        /// Event about another signature for a multi-signer execution request.
        /// [account, multisignstate]
        SignedMultisigScript { who: T::AccountId },
        /// Event about successful stdlib update executed
        /// No parameters.
        StdlibUpdated,
    }

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        #[serde(skip)]
        pub _phantom: core::marker::PhantomData<T>,

        /// Use this option to override the default move-stdlib provided by the move-vm-backend.
        pub change_default_move_stdlib_bundle_to: Option<Vec<u8>>,

        /// Use this option to override the default substrate-stdlib provided by the move-vm-backend.
        pub change_default_substrate_stdlib_bundle_to: Option<Vec<u8>>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            let mut genesis_cfg = VmGenesisConfig::default();
            if let Some(ref bundle) = self.change_default_move_stdlib_bundle_to {
                genesis_cfg.configure_stdlib(bundle.clone());
            }
            if let Some(ref bundle) = self.change_default_substrate_stdlib_bundle_to {
                genesis_cfg.configure_substrate_stdlib(bundle.clone());
            }

            let storage = Pallet::<T>::move_vm_storage();

            assert!(
                genesis_cfg.apply(storage).is_ok(),
                "failed to apply the move-vm genesis config"
            );
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
    where
        BalanceOf<T>: From<u128> + Into<u128>,
    {
        fn on_idle(block_height: BlockNumberFor<T>, mut remaining_weight: Weight) -> Weight {
            let len = MultisigStorage::<T>::iter_keys().count() as u64;
            let lifetime = T::MaxLifetimeRequests::get();
            // Abort if storage is empty or the allowed lifetime is longer than the blockchain's
            // existence (otherwise, an integer underflow can occur).
            if len == 0 || block_height < lifetime {
                return remaining_weight - T::DbWeight::get().reads(1);
            }

            // We will read three times for sure and write one time for sure for the storage,
            // no matter if we execute the internal chore method or not.
            remaining_weight -= T::DbWeight::get().reads_writes(3, 1);

            let mut idx: u64 = ChoreOnIdleStorage::<T>::get().unwrap_or(0);
            if idx >= len {
                idx = 0;
            }

            let keys = MultisigStorage::<T>::iter_keys().skip(idx as usize);
            let block_tbr = block_height - lifetime;
            let mut call = Vec::<CallHash>::new();
            let mut count: u64 = 0;

            for key in keys {
                if let Some(call_hash) = Self::chore_multisig_storage(key, block_tbr) {
                    call.push(call_hash);
                }
                count += 1;
                if let Some(weight) =
                    remaining_weight.checked_sub(&T::WeightInfo::chore_multisig_storage())
                {
                    remaining_weight = weight;
                    if count >= MAX_MULTISIG_CHECKING_PER_BLOCK {
                        break;
                    }
                } else {
                    remaining_weight = Weight::zero();
                    break;
                }
            }

            let n_removed = call.len() as u64;
            idx += count - n_removed;
            if idx >= len - n_removed {
                idx = 0;
            }
            ChoreOnIdleStorage::<T>::put(idx);

            if !call.is_empty() {
                Self::deposit_event(Event::<T>::MultiSignRequestRemoved { call });
            }

            remaining_weight
        }
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        BalanceOf<T>: From<u128> + Into<u128>,
    {
        /// Execute Move script transaction sent by the user.
        // TODO(eiger) in M3: ensure the weight depends on basic extrinsic cost + gas_limit + size of the
        // transaction_bc.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::execute())]
        pub fn execute(
            origin: OriginFor<T>,
            transaction_bc: Vec<u8>,
            gas_limit: u64,
            cheque_limit: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            // TODO(neutrinoks): Add currency locking in multi-signature executions.
            // A signer for the extrinsic and a signer for the Move script.
            let who = ensure_signed(origin)?;

            // We use gas in order to prevent infinite scripts from breaking the MoveVM.
            let gas_amount = GasAmount::new(gas_limit).map_err(|_| Error::<T>::GasLimitExceeded)?;
            let gas = GasStrategy::Metered(gas_amount);

            // Main input for the VM are these script parameters.
            let ScriptTransaction {
                bytecode,
                args,
                type_args,
            } = ScriptTransaction::try_from(transaction_bc.as_ref())
                .map_err(|_| Error::<T>::InvalidScriptTransaction)?;
            let args: Vec<&[u8]> = args.iter().map(AsRef::as_ref).collect();

            // Make sure the scripts are not maliciously trying to use forged signatures.
            let signer_count =
                verify_script_integrity_and_check_signers(&bytecode).map_err(Error::<T>::from)?;
            let accounts = Self::extract_account_ids_from_args(&args, signer_count)?;
            let block_height = <frame_system::Pallet<T>>::block_number();

            let (mut signature_handler, call_hash) = if signer_count > 1 {
                // Generate the call hash to identify this multi-sign call request.
                let call_hash = Self::transaction_bc_call_hash(&transaction_bc[..]);

                let multisig = MultisigStorage::<T>::get(call_hash).unwrap_or(
                    MultisigOf::<T>::new(accounts, block_height).map_err(Into::<Error<T>>::into)?,
                );

                (ScriptSignatureHandler::<T>::from(multisig), call_hash)
            } else {
                (
                    ScriptSignatureHandler::<T>::new(accounts, block_height)?,
                    [0u8; 32],
                )
            };
            if signer_count > 0 {
                signature_handler.sign_script(&who, cheque_limit.into())?;
            }

            // If the script is signed correctly, we can execute it in MoveVM and update the
            // blockchain storage or the balance sheet.
            // If we have only one signer, it will skip this; if not, we have to wait for more signers, so we store it as a multi-signer request.
            if !signature_handler.all_signers_approved() {
                MultisigStorage::<T>::insert(call_hash, signature_handler.into_inner());
                Self::deposit_event(Event::SignedMultisigScript { who });
                return result::execute_only_signing();
            }
            // If we have multiple signers and they all have signed, we have to remove the multi-signer request from the MultisigStorage.
            if signer_count > 1 {
                MultisigStorage::<T>::remove(call_hash);
            }

            // We need to provide MoveVM read only access to balance sheet - MoveVM is allowed to
            // update the cheques that are used afterwards to update the balances afterwards.
            let balance = signature_handler.write_cheques()?;

            // Let's try execute the script.
            let cheques = balance.clone(); // VM can only touch the cheque list, it cannot update balances directly.
            let vm_result = Self::raw_execute_script(&bytecode, type_args, args, gas, cheques)?;

            // Apply true transactions to blockchain - this can be done only from the pallet layer
            // after the script executed correctly without any issues.
            balance.apply_transactions()?;

            let result = result::from_vm_result::<T>(vm_result)?;

            // Emit an event.
            let signers = signature_handler.into_signer_accounts()?;
            Self::deposit_event(Event::ExecuteCalled { who: signers });

            Ok(result)
        }

        /// Publish a Move module sent by the user.
        /// Module is published under its sender's address.
        // TODO(eiger) in M3: ensure the weight depends on basic extrinsic cost + gas_limit
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::publish_module())]
        pub fn publish_module(
            origin: OriginFor<T>,
            bytecode: Vec<u8>,
            gas_limit: u64,
        ) -> DispatchResultWithPostInfo {
            // Allow only signed calls.
            let who = ensure_signed(origin)?;
            let address = Self::to_move_address(&who)?;

            let gas_amount = GasAmount::new(gas_limit).map_err(|_| Error::<T>::GasLimitExceeded)?;
            let gas = GasStrategy::Metered(gas_amount);

            let vm_result = Self::raw_publish_module(&address, bytecode, gas)?;

            // Produce a result with gas spent.
            let result = result::from_vm_result::<T>(vm_result)?;

            // Emit an event.
            Self::deposit_event(Event::ModulePublished { who });

            Ok(result)
        }

        /// Publish a Move bundle sent by the user.
        ///
        /// Bundle is just a set of multiple modules.
        // TODO(eiger) in M3: ensure the weight depends on basic extrinsic cost + gas_limit
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::publish_module_bundle())]
        pub fn publish_module_bundle(
            origin: OriginFor<T>,
            bundle: Vec<u8>,
            gas_limit: u64,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let address = Self::to_move_address(&who)?;

            let gas_amount = GasAmount::new(gas_limit).map_err(|_| Error::<T>::GasLimitExceeded)?;
            let gas = GasStrategy::Metered(gas_amount);

            let vm_result = Self::raw_publish_bundle(&address, bundle, gas)?;

            // Produce a result with gas spent.
            let result = result::from_vm_result::<T>(vm_result)?;

            // Emit an event.
            Self::deposit_event(Event::BundlePublished { who });

            Ok(result)
        }

        /// Publish a standard library bundle, e.g. Move-Stdlib or Substrate-Stdlib. Sudo user only.
        ///
        /// All standard libraries are published at their default address 0x1.
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::update_stdlib())]
        pub fn update_stdlib_bundle(
            origin: OriginFor<T>,
            stdlib: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            let vm_result =
                Self::raw_publish_bundle(&CORE_CODE_ADDRESS, stdlib, GasStrategy::Unmetered)?;
            let pd_info = result::from_vm_result::<T>(vm_result)?;

            Self::deposit_event(Event::<T>::StdlibUpdated);

            Ok(pd_info)
        }
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

    impl<T: Config> Pallet<T>
    where
        BalanceOf<T>: From<u128> + Into<u128>,
    {
        // Internal helper for creating new MoveVM instance with StorageAdapter.
        fn move_vm() -> MvmResult<T> {
            // Balance won't actually be used here.
            let balance = BalanceAdapter::new();
            let storage = Self::move_vm_storage();

            Mvm::new(storage, balance)
                .map_err::<Vec<u8>, _>(|e| format!("error while creating the vm {e:?}").into())
        }

        /// Convert Move address to Substrate native account.
        pub fn to_native_account(address: &AccountAddress) -> Result<T::AccountId, Error<T>> {
            T::AccountId::decode(&mut address.as_ref()).map_err(|_| Error::InvalidAccountSize)
        }

        /// Convert a native address to a Move address.
        pub fn to_move_address(address: &T::AccountId) -> Result<AccountAddress, Error<T>> {
            let address = AccountId32::decode(&mut address.encode().as_ref())
                .map_err(|_| Error::InvalidAccountSize)?;

            let account_bytes: [u8; 32] = address.into();
            let address = AccountAddress::new(account_bytes);
            if address == CORE_CODE_ADDRESS {
                Err(Error::<T>::StdlibAddressNotAllowed)
            } else {
                Ok(address)
            }
        }

        /// Execute the script using the appropriate gas strategy.
        pub fn raw_execute_script(
            script: &[u8],
            type_args: Vec<TypeTag>,
            args: Vec<&[u8]>,
            gas: GasStrategy,
            cheques: impl BalanceHandler,
        ) -> Result<VmResult, Error<T>> {
            let storage = Self::move_vm_storage();

            let vm = Mvm::new(storage, cheques).map_err(|_| Error::<T>::ExecuteFailed)?;

            let result = vm.execute_script(script, type_args, args, gas);

            Ok(result)
        }

        /// Publish the module using the appropriate gas strategy.
        pub fn raw_publish_module(
            address: &AccountAddress,
            bytecode: Vec<u8>,
            gas: GasStrategy,
        ) -> Result<VmResult, Error<T>> {
            let storage = Self::move_vm_storage();

            let vm = Mvm::new(storage, BalanceAdapter::<T>::new())
                .map_err(|_| Error::<T>::PublishModuleFailed)?;

            let result = vm.publish_module(&bytecode, *address, gas);

            Ok(result)
        }

        /// Publish the bundle using the appropriate gas strategy.
        pub fn raw_publish_bundle(
            address: &AccountAddress,
            bundle: Vec<u8>,
            gas: GasStrategy,
        ) -> Result<VmResult, Error<T>> {
            let storage = Self::move_vm_storage();

            let vm = Mvm::new(storage, BalanceAdapter::<T>::new())
                .map_err(|_| Error::<T>::PublishBundleFailed)?;

            let result = vm.publish_module_bundle(&bundle, *address, gas);

            Ok(result)
        }

        pub fn get_module_abi(
            address: &T::AccountId,
            name: &str,
        ) -> Result<Option<ModuleAbi>, Vec<u8>> {
            let vm = Self::move_vm()?;

            // TODO: Return a normal message to the RPC caller
            let address = Self::to_move_address(address).map_err(|_| vec![])?;

            vm.get_module_abi(address, name)
                .map_err(|e| format!("error in get_module_abi: {e:?}").into())
        }

        pub fn get_module(address: &T::AccountId, name: &str) -> Result<Option<Vec<u8>>, Vec<u8>> {
            let vm = Self::move_vm()?;

            // TODO: Return a normal message to the RPC caller
            let address = Self::to_move_address(address).map_err(|_| vec![])?;

            vm.get_module(address, name)
                .map_err(|e| format!("error in get_module: {e:?}").into())
        }

        pub fn get_resource(
            account: &T::AccountId,
            tag: &[u8],
        ) -> Result<Option<Vec<u8>>, Vec<u8>> {
            let vm = Self::move_vm()?;
            // TODO: Return a normal message to the RPC caller
            let address = Self::to_move_address(account).map_err(|_| vec![])?;

            vm.get_resource(&address, tag)
                .map_err(|e| format!("error in get_resource: {e:?}").into())
        }

        fn extract_account_ids_from_args(
            script_args: &[&[u8]],
            signer_count: usize,
        ) -> Result<Vec<T::AccountId>, Error<T>> {
            if signer_count > script_args.len() {
                return Err(Error::<T>::ScriptSignatureFailure);
            }

            let mut accounts = Vec::<T::AccountId>::new();
            for signer in &script_args[..signer_count] {
                let account_address =
                    bcs::from_bytes(signer).map_err(|_| Error::<T>::UnableToDeserializeAccount)?;
                let account = Self::to_native_account(&account_address)?;
                accounts.push(account);
            }

            Ok(accounts)
        }

        fn chore_multisig_storage(key: CallHash, block_tbr: BlockNumberFor<T>) -> Option<CallHash> {
            let multisig = MultisigStorage::<T>::get(key)?;
            if *multisig.block_number() > block_tbr {
                None
            } else {
                MultisigStorage::<T>::remove(key);
                Some(key)
            }
        }

        pub fn transaction_bc_call_hash(transaction_bc: &[u8]) -> CallHash {
            let mut hasher = Blake2s256::new();
            hasher.update(transaction_bc);
            hasher.finalize().into()
        }
    }

    #[pallet::error]
    pub enum Error<T> {
        // General errors
        /// Error returned when executing Move script bytecode failed.
        ExecuteFailed,
        /// Error returned when publishing Move module failed.
        PublishModuleFailed,
        /// Error returned when publishing Move bundle failed.
        PublishBundleFailed,
        /// Invalid account size (expected 32 bytes).
        InvalidAccountSize,
        /// Gas limit too big (maximum gas limit is: 2^64 / 1000).
        GasLimitExceeded,
        /// Invalid account size (expected 32 bytes).
        InsufficientBalance,
        /// Script signature failure.
        ScriptSignatureFailure,
        /// Script transaction cannot be deserialized.
        InvalidScriptTransaction,
        /// User tried to publish module in a protected memory area.
        StdlibAddressNotAllowed,
        /// Error about signing multi-signature execution request twice.
        UserHasAlreadySigned,
        /// Script contains more signers than allowed maximum number of signers.
        MaxSignersExceeded,

        // Errors that can be received from MoveVM
        /// Unknown validation status
        UnknownValidationStatus,
        /// The transaction has a bad signature
        InvalidSignature,
        /// Bad account authentication key
        InvalidAuthKey,
        /// Sequence number is too old
        SequenceNumberTooOld,
        /// Sequence number is too new
        SequenceNumberTooNew,
        /// The sequence number is too large and would overflow if the transaction were executed
        SequenceNumberTooBig,
        /// Insufficient balance to pay minimum transaction fee
        InsufficientBalanceForTransactionFee,
        /// The transaction has expired
        TransactionExpired,
        /// The sending account does not exist
        SendingAccountDoesNotExist,
        /// This write set transaction was rejected because it did not meet the requirements for one.
        RejectedWriteSet,
        /// This write set transaction cannot be applied to the current state.
        InvalidWriteSet,
        /// Length of program field in raw transaction exceeded max length
        ExceededMaxTransactionSize,
        /// This script is not in our allowlist of scripts.
        UnknownScript,
        /// Transaction is trying to publish a new module.
        UnknownModule,
        /// Max gas units submitted with transaction exceeds max gas units bound in VM
        MaxGasUnitsExceedsMaxGasUnitsBound,
        /// Max gas units submitted with transaction not enough to cover the intrinsic cost of the transaction.
        MaxGasUnitsBelowMinTransactionGasUnits,
        /// Gas unit price submitted with transaction is below minimum gas price set in the VM.
        GasUnitPriceBelowMinBound,
        /// Gas unit price submitted with the transaction is above the maximum gas price set in the VM.
        GasUnitPriceAboveMaxBound,
        /// Gas specifier submitted is either malformed (not a valid identifier), or does not refer to an accepted gas specifier
        InvalidGasSpecifier,
        /// The sending account is frozen
        SendingAccountFrozen,
        /// Unable to deserialize the account blob
        UnableToDeserializeAccount,
        /// The currency info was unable to be found
        CurrencyInfoDoesNotExist,
        /// The account sender doesn't have permissions to publish modules
        InvalidModulePublisher,
        /// The sending account has no role
        NoAccountRole,
        /// The transaction's chain_id does not match the one published on-chain
        BadChainId,
        /// Unknown verification error
        UnknownVerificationError,
        /// Index out of bounds
        IndexOutOfBounds,
        /// Invalid signature token
        InvalidSignatureToken,
        /// Recursive struct definition
        RecursiveStructDefinition,
        /// Invalid resource field
        InvalidResourceField,
        /// Invalid fall through
        InvalidFallThrough,
        /// Negative stack size within block
        NegativeStackSizeWithinBlock,
        /// Invalid main function signature
        InvalidMainFunctionSignature,
        /// Duplicate element
        DuplicateElement,
        /// Invalid module handle
        InvalidModuleHandle,
        /// Unimplemented handle
        UnimplementedHandle,
        /// Lookup failed
        LookupFailed,
        /// Type mismatch
        TypeMismatch,
        /// Missing dependency
        MissingDependency,
        /// Pop resource error
        PopResourceError,
        /// Br type mismatch
        BrTypeMismatchError,
        /// Abort type mismatch error
        AbortTypeMismatchError,
        /// Stloc type mismatch error
        StlocTypeMismatchError,
        /// Stloc unsafe to destroy error
        StlocUnsafeToDestroyError,
        /// Unsafe ret local or resource still borrowed
        UnsafeRetLocalOrResourceStillBorrowed,
        /// Ret type mismatch error
        RetTypeMismatchError,
        /// Ret borrowed mutable reference error
        RetBorrowedMutableReferenceError,
        /// Freezeref type mismatch error
        FreezerefTypeMismatchError,
        /// Freezeref exists mutable borrow error
        FreezerefExistsMutableBorrowError,
        /// Borrowfield type mismatch error
        BorrowfieldTypeMismatchError,
        /// Borrowfield bad field error
        BorrowfieldBadFieldError,
        /// Borrowfield exists mutable borrow error
        BorrowfieldExistsMutableBorrowError,
        /// Copyloc unavailable error
        CopylocUnavailableError,
        /// Copyloc resource error
        CopylocResourceError,
        /// Copyloc exists borrow error
        CopylocExistsBorrowError,
        /// Moveloc unavailable error
        MovelocUnavailableError,
        /// Moveloc exists borrow error
        MovelocExistsBorrowError,
        /// Borrowloc reference error
        BorrowlocReferenceError,
        /// Borrowloc unavailable error
        BorrowlocUnavailableError,
        /// Borrowloc exists borrow error
        BorrowlocExistsBorrowError,
        /// Call type mismatch error
        CallTypeMismatchError,
        /// Call borrowed mutable reference error
        CallBorrowedMutableReferenceError,
        /// Pack type mismatch error
        PackTypeMismatchError,
        /// Unpack type mismatch error
        UnpackTypeMismatchError,
        /// Readref type mismatch error
        ReadrefTypeMismatchError,
        /// Readref resource error
        ReadrefResourceError,
        /// Readref exists mutable borrow error
        ReadrefExistsMutableBorrowError,
        /// Writeref type mismatch error
        WriterefTypeMismatchError,
        /// Writeref resource error
        WriterefResourceError,
        /// Writeref exists borrow error
        WriterefExistsBorrowError,
        /// Writeref no mutable reference error
        WriterefNoMutableReferenceError,
        /// Integer op type mismatch error
        IntegerOpTypeMismatchError,
        /// Boolean op type mismatch error
        BooleanOpTypeMismatchError,
        /// Equality op type mismatch error
        EqualityOpTypeMismatchError,
        /// Exists resource type mismatch error
        ExistsResourceTypeMismatchError,
        /// Borrowglobal type mismatch error
        BorrowglobalTypeMismatchError,
        /// Borrowglobal no resource error
        BorrowglobalNoResourceError,
        /// Movefrom Type mismatch error
        MovefromTypeMismatchError,
        /// Movefrom no resource error
        MovefromNoResourceError,
        /// Moveto type mismatch error
        MovetoTypeMismatchError,
        /// Moveto no resource error
        MovetoNoResourceError,
        /// The self address of a module the transaction is publishing is not the sender address
        ModuleAddressDoesNotMatchSender,
        /// The module does not have any module handles. Each module or script must have at least one module handle.
        NoModuleHandles,
        /// Positive stack size at block end
        PositiveStackSizeAtBlockEnd,
        /// Missing acquires resource annotation error
        MissingAcquiresResourceAnnotationError,
        /// Extraneous acquires resource annotation error
        ExtraneousAcquiresResourceAnnotationError,
        /// Duplicate acquires resource annotation error
        DuplicateAcquiresResourceAnnotationError,
        /// Invalid acquires resource annotation error
        InvalidAcquiresResourceAnnotationError,
        /// Global reference error
        GlobalReferenceError,
        /// Constraint kind mismatch
        ConstraintKindMismatch,
        /// Number of type arguments mismatch
        NumberOfTypeArgumentsMismatch,
        /// Loop in instantiation graph
        LoopInInstantiationGraph,
        /// Zero sized struct.
        ZeroSizedStruct,
        /// Linker error
        LinkerError,
        /// Invalid constant type
        InvalidConstantType,
        /// Malformed constant data
        MalformedConstantData,
        /// Empty code unit
        EmptyCodeUnit,
        /// Invalid loop split
        InvalidLoopSplit,
        /// Invalid loop break
        InvalidLoopBreak,
        /// Invalid loop continue
        InvalidLoopContinue,
        /// Unsafe fet unused resources
        UnsafeRetUnusedResources,
        /// Too many locals
        TooManyLocals,
        /// Generic member opcode mismatch
        GenericMemberOpcodeMismatch,
        /// Function resolution failure
        FunctionResolutionFailure,
        /// Invalid operation in script
        InvalidOperationInScript,
        /// The sender is trying to publish a module named `M`, but the sender's account already contains a module with this name.
        DuplicateModuleName,
        /// Unknown invariant violation error
        UnknownInvariantViolationError,
        /// Empty value stack
        EmptyValueStack,
        /// Pc overflow
        PcOverflow,
        /// Verification error
        VerificationError,
        /// Storage error
        StorageError,
        /// Internal type error
        InternalTypeError,
        /// Event key mismatch
        EventKeyMismatch,
        /// Unreachable
        Unreachable,
        /// vm startup failure
        VmStartupFailure,
        /// Unexpected error from known move function
        UnexpectedErrorFromKnownMoveFunction,
        /// Verifier invariant violation
        VerifierInvariantViolation,
        /// Unexpected verifier error
        UnexpectedVerifierError,
        /// Unexpected deserialization error
        UnexpectedDeserializationError,
        /// Failed to serialize write set changes
        FailedToSerializeWriteSetChanges,
        /// Failed to deserialize resource
        FailedToDeserializeResource,
        /// Failed to resolve type due to linking being broken after verification
        TypeResolutionFailure,
        /// Unknown binary error
        UnknownBinaryError,
        /// Malformed
        Malformed,
        /// Bad magic
        BadMagic,
        /// Unknown version
        UnknownVersion,
        /// Unknown table type
        UnknownTableType,
        /// Unknown signature type
        UnknownSignatureType,
        /// Unknown serialized type
        UnknownSerializedType,
        /// Unknown opcode
        UnknownOpcode,
        /// BadHeader table
        BadHeaderTable,
        /// Unexpected signature type
        UnexpectedSignatureType,
        /// Duplicate table
        DuplicateTable,
        /// Unknown kind
        UnknownKind,
        /// Unknown native struct flag
        UnknownNativeStructFlag,
        /// Bad U64
        BadU64,
        /// Bad U128
        BadU128,
        /// Value serialization error
        ValueSerializationError,
        /// Value deserialization error
        ValueDeserializationError,
        /// Code deserialization error
        CodeDeserializationError,
        /// Unknown runtime status
        UnknownRuntimeStatus,
        /// Out of gas
        OutOfGas,
        /// We tried to access a resource that does not exist under the account.
        ResourceDoesNotExist,
        /// We tried to create a resource under an account where that resource already exists.
        ResourceAlreadyExists,
        /// Missing data
        MissingData,
        /// Data format error
        DataFormatError,
        /// Aborted
        Aborted,
        /// Arithmetic error
        ArithmeticError,
        /// Execution stack overflow
        ExecutionStackOverflow,
        /// Call stack overflow
        CallStackOverflow,
        /// Vm max type depth reached
        VmMaxTypeDepthReached,
        /// Vm max value depth reached
        VmMaxValueDepthReached,
        /// Unknown status.
        UnknownStatus,

        // Documentation_missing
        BadTransactionFeeCurrency,
        // Documentation_missing
        FeatureUnderGating,
        // Documentation_missing
        FieldMissingTypeAbility,
        // Documentation_missing
        PopWithoutDropAbility,
        // Documentation_missing
        CopylocWithoutCopyAbility,
        // Documentation_missing
        ReadrefWithoutCopyAbility,
        // Documentation_missing
        WriterefWithoutDropAbility,
        // Documentation_missing
        ExistsWithoutKeyAbilityOrBadArgument,
        // Documentation_missing
        BorrowglobalWithoutKeyAbility,
        // Documentation_missing
        MovefromWithoutKeyAbility,
        // Documentation_missing
        MovetoWithoutKeyAbility,
        // Documentation_missing
        MissingAcquiresAnnotation,
        // Documentation_missing
        ExtraneousAcquiresAnnotation,
        // Documentation_missing
        DuplicateAcquiresAnnotation,
        // Documentation_missing
        InvalidAcquiresAnnotation,
        // Documentation_missing
        ConstraintNotSatisfied,
        // Documentation_missing
        UnsafeRetUnusedValuesWithoutDrop,
        // Documentation_missing
        BackwardIncompatibleModuleUpdate,
        // Documentation_missing
        CyclicModuleDependency,
        // Documentation_missing
        NumberOfArgumentsMismatch,
        // Documentation_missing
        InvalidParamTypeForDeserialization,
        // Documentation_missing
        FailedToDeserializeArgument,
        // Documentation_missing
        NumberOfSignerArgumentsMismatch,
        // Documentation_missing
        CalledScriptVisibleFromNonScriptVisible,
        // Documentation_missing
        ExecuteScriptFunctionCalledOnNonScriptVisible,
        // Documentation_missing
        InvalidFriendDeclWithSelf,
        // Documentation_missing
        InvalidFriendDeclWithModulesOutsideAccountAddress,
        // Documentation_missing
        InvalidFriendDeclWithModulesInDependencies,
        // Documentation_missing
        CyclicModuleFriendship,
        // Documentation_missing
        UnknownAbility,
        // Documentation_missing
        InvalidFlagBits,
        // Wrong secondary keys addresses count
        SecondaryKeysAddressesCountMismatch,
        // List of signers contain duplicates
        SignersContainDuplicates,
        // Invalid sequence nonce
        SequenceNonceInvalid,
        // Invalid phantom type param position
        InvalidPhantomTypeParamPosition,
        // Documentation_missing
        VecUpdateExistsMutableBorrowError,
        // Documentation_missing
        VecBorrowElementExistsMutableBorrowError,
        // Found duplicate of native function
        DuplicateNativeFunction,
    }
}