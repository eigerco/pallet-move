#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod types;

use alloc::{string::String, vec::Vec};

use frame_support::weights::Weight;
pub use pallet_move::pallet::ModuleAbi;

pub use crate::types::MoveApiEstimation;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime file (the `runtime/src/lib.rs` of the node)
sp_api::decl_runtime_apis! {
    pub trait MoveApi<AccountId> where      // AccountID is already here for the next API calls.
        AccountId: codec::Codec,
    {
        // Convert Weight to Gas.
        fn gas_to_weight(gas_limit: u64) -> Weight;

        // Convert Gas to Weight.
        fn weight_to_gas(weight: Weight) -> u64;

        // Estimate gas for publish module.
        fn estimate_gas_publish_module(account: AccountId, bytecode: Vec<u8>) -> Result<MoveApiEstimation, sp_runtime::DispatchError>;

        // Estimate gas for publish bundle.
        fn estimate_gas_publish_bundle(account: AccountId, bytecode: Vec<u8>) -> Result<MoveApiEstimation, sp_runtime::DispatchError>;

        // Estimate gas for execute script.
        fn estimate_gas_execute(account: AccountId, bytecode: Vec<u8>) -> Result<MoveApiEstimation, sp_runtime::DispatchError>;

        // Get module binary by its address
        fn get_module(address: String, name: String) -> Result<Option<Vec<u8>>, Vec<u8>>;

        // Get module ABI by its address
        fn get_module_abi(address: String, name: String) -> Result<Option<ModuleAbi>, Vec<u8>>;

        // Get resource
        fn get_resource(account: AccountId, tag: Vec<u8>) -> Result<Option<Vec<u8>>, Vec<u8>>;
    }
}
