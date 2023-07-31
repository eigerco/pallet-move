use std::sync::Arc;

use codec::Codec;
use frame_support::weights::Weight;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
pub use pallet_move_runtime_api::MoveApi as MoveRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;

/// Public RPC API of the Move pallet.
#[rpc(server)]
pub trait MoveApi<BlockHash, AccountId> {
    /// Convert gas to weight
    #[method(name = "mvm_gasToWeight")]
    fn gas_to_weight(&self, gas: u64, at: Option<BlockHash>) -> RpcResult<Weight>;

    /// Convert weight to gas
    #[method(name = "mvm_weightToGas")]
    fn weight_to_gas(&self, weight: Weight, at: Option<BlockHash>) -> RpcResult<u64>;
}

/// A struct that implements the `MoveApi`.
pub struct MovePallet<C, Block> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<Block>,
}

impl<C, Block> MovePallet<C, Block> {
    /// Create new `MovePallet` instance with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, AccountId> MoveApiServer<<Block as BlockT>::Hash, AccountId> for MovePallet<C, Block>
where
    Block: BlockT,
    AccountId: Clone + std::fmt::Display + Codec,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: MoveRuntimeApi<Block, AccountId>,
{
    fn gas_to_weight(&self, gas: u64, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Weight> {
        // Return a dummy weight for now
        Ok(Weight::from_parts(2_123_123, 0))
    }

    fn weight_to_gas(&self, weight: Weight, at: Option<<Block as BlockT>::Hash>) -> RpcResult<u64> {
        // Return a dummy gas for now
        Ok(1u64)
    }
}
