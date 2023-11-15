use core::convert::TryFrom;

//TODO: either leave it as is when moving to move-vm-backend (then remove this comment) or
// change it to crate error type
use anyhow::Error;
use frame_support::dispatch::Vec;
use move_core_types::language_storage::TypeTag;
use serde::{Deserialize, Serialize};

/// Transaction representation used in execute call.
#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    /// Script bytecode.
    pub script_bc: Vec<u8>,
    /// Script args.
    pub args: Vec<Vec<u8>>,
    /// Script type arguments.
    pub type_args: Vec<TypeTag>,
}

impl TryFrom<&[u8]> for Transaction {
    type Error = Error;
    fn try_from(blob: &[u8]) -> Result<Self, Self::Error> {
        bcs::from_bytes(blob).map_err(Error::msg)
    }
}
