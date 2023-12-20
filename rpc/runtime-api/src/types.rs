use codec::{Decode, Encode};
use sp_runtime::scale_info::TypeInfo;

/// Gas estimation information.
#[derive(Clone, PartialEq, Debug, Encode, Decode, TypeInfo)]
pub struct MoveApiEstimation {
    /// Gas used.
    pub gas_used: u64,
    /// Status code for the MoveVM execution.
    pub vm_status_code: u64,
}
