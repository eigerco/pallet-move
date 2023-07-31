#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::weights::Weight;

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

    }
}
