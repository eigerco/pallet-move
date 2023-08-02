#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{dispatch::Vec, weights::Weight};

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
        fn estimate_gas_publish(account: AccountId, bytecode: Vec<u8>, gas_limit: u64) -> u64;

        // Estimate gas for execute script.
        fn estimate_gas_execute(account: AccountId, bytecode: Vec<u8>, gas_limit: u64) -> u64;

        // Get module binary by its address
        fn get_module(module_id: Vec<u8>) -> Result<Option<Vec<u8>>, Vec<u8>>;

        // Get module ABI by its address
        fn get_module_abi(module_id: Vec<u8>) -> Result<Option<Vec<u8>>, Vec<u8>>;

        // Get resource
        fn get_resource(account: AccountId, tag: Vec<u8>) -> Result<Option<Vec<u8>>, Vec<u8>>;

    }
}
