use std::sync::Arc;

use codec::Codec;
use frame_support::{dispatch::Vec, weights::Weight};
use jsonrpsee::{
    core::{Error as JsonRpseeError, RpcResult},
    proc_macros::rpc,
    types::error::{CallError, ErrorObject},
};
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

    /// Estimate gas for publishing module
    #[method(name = "mvm_estimateGasPublish")]
    fn estimate_gas_publish(
        &self,
        account: AccountId,
        bytecode: Vec<u8>,
        gas_limit: u64,
        at: Option<BlockHash>,
    ) -> RpcResult<u64>;

    /// Estimate gas for executing Move script
    #[method(name = "mvm_estimateGasExecute")]
    fn estimate_gas_execute(
        &self,
        account: AccountId,
        bytecode: Vec<u8>,
        gas_limit: u64,
        at: Option<BlockHash>,
    ) -> RpcResult<u64>;

    /// Get resource
    #[method(name = "mvm_getResource")]
    fn get_resource(
        &self,
        account: AccountId,
        tag: Vec<u8>,
        at: Option<BlockHash>,
    ) -> RpcResult<Option<Vec<u8>>>;

    /// Get module ABI using address
    #[method(name = "mvm_getModuleABI")]
    fn get_module_abi(&self, module_id: Vec<u8>, at: Option<BlockHash>) -> RpcResult<Option<Vec<u8>>>;

    /// Get module binary using address
    #[method(name = "mvm_getModule")]
    fn get_module(&self, module_id: Vec<u8>, at: Option<BlockHash>) -> RpcResult<Option<Vec<u8>>>;
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
        let api = self.client.runtime_api();
        let res = api.gas_to_weight(at.unwrap_or_else(|| self.client.info().best_hash), gas);

        res.map_err(runtime_error_into_rpc_err)
    }

    fn weight_to_gas(&self, weight: Weight, at: Option<<Block as BlockT>::Hash>) -> RpcResult<u64> {
        let api = self.client.runtime_api();
        let res = api.weight_to_gas(at.unwrap_or_else(|| self.client.info().best_hash), weight);

        res.map_err(runtime_error_into_rpc_err)
    }

    fn estimate_gas_publish(
        &self,
        account: AccountId,
        bytecode: Vec<u8>,
        gas_limit: u64,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<u64> {
        let api = self.client.runtime_api();
        let res =
            api.estimate_gas_publish(at.unwrap_or_else(|| self.client.info().best_hash), account, bytecode, gas_limit);

        res.map_err(runtime_error_into_rpc_err)
    }

    fn estimate_gas_execute(
        &self,
        account: AccountId,
        bytecode: Vec<u8>,
        gas_limit: u64,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<u64> {
        let api = self.client.runtime_api();
        let res =
            api.estimate_gas_execute(at.unwrap_or_else(|| self.client.info().best_hash), account, bytecode, gas_limit);

        res.map_err(runtime_error_into_rpc_err)
    }

    fn get_resource(
        &self,
        account: AccountId,
        tag: Vec<u8>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Option<Vec<u8>>> {
        let api = self.client.runtime_api();
        let res = api.get_resource(at.unwrap_or_else(|| self.client.info().best_hash), account, tag);

        // Currently, there is always correct value returned so it's safe to unwrap here.
        res.unwrap().map_err(runtime_error_into_rpc_err)
    }

    fn get_module_abi(
        &self,
        module_id: Vec<u8>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Option<Vec<u8>>> {
        let api = self.client.runtime_api();
        let res = api.get_module_abi(at.unwrap_or_else(|| self.client.info().best_hash), module_id);

        // Currently, there is always correct value returned so it's safe to unwrap here.
        res.unwrap().map_err(runtime_error_into_rpc_err)
    }

    fn get_module(
        &self,
        module_id: Vec<u8>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Option<Vec<u8>>> {
        let api = self.client.runtime_api();
        let res = api.get_module(at.unwrap_or_else(|| self.client.info().best_hash), module_id);

        // Currently, there is always correct value returned so it's safe to unwrap here.
        res.unwrap().map_err(runtime_error_into_rpc_err)
    }
}

const RUNTIME_ERROR: i32 = 1;

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> JsonRpseeError {
    CallError::Custom(ErrorObject::owned(
        RUNTIME_ERROR,
        "Runtime error",
        Some(format!("{:?}", err)),
    ))
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_error_into_rpc_err_test_str() {
        let err_str = "test";
        let err_str_tst = "\"\\\"test\\\"\"";
        let res = runtime_error_into_rpc_err(err_str);
        match res {
            JsonRpseeError::Call(err) => {
                match err {
                    CallError::Custom(err) => {
                        assert_eq!(err.code(), RUNTIME_ERROR);
                        assert_eq!(err.message(), "Runtime error");
                        assert_eq!(err.data().unwrap().get(), err_str_tst)
                    }
                    _ => panic!("Wrong error type"),
                }
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn runtime_error_into_rpc_err_empty_str() {
        let err_str = "";
        let err_str_tst = "\"\\\"\\\"\"";
        let res = runtime_error_into_rpc_err(err_str);
        match res {
            JsonRpseeError::Call(err) => {
                match err {
                    CallError::Custom(err) => {
                        assert_eq!(err.code(), RUNTIME_ERROR);
                        assert_eq!(err.message(), "Runtime error");
                        assert_eq!(err.data().unwrap().get(), err_str_tst)
                    }
                    _ => panic!("Wrong error type"),
                }
            }
            _ => panic!("Wrong error type"),
        }
    }
}