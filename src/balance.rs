use move_core_types::{account_address::AccountAddress, vm_status::StatusCode};
use move_vm_backend::balance::BalanceHandler;

/// Balance adapter for providing basic access to balance cheques within the MoveVM.
#[derive(Clone)]
pub struct BalanceAdapter {
    // TODO(rqnsom)
}

impl BalanceAdapter {
    /// Create a new [`BalanceAdapter`].
    pub fn new() -> Self {
        BalanceAdapter {
            // TODO(rqnsom)
        }
    }
}

impl BalanceHandler for BalanceAdapter {
    type Error = StatusCode;

    fn transfer(
        &self,
        _src: AccountAddress,
        _dst: AccountAddress,
        _cheque_amount: u128,
    ) -> Result<bool, Self::Error> {
        // TODO(rqnsom): This won't break the MoveVM unless the balance module is accessed.
        unimplemented!()
    }

    fn cheque_amount(&self, _account: AccountAddress) -> Result<u128, Self::Error> {
        // TODO(rqnsom): This won't break the MoveVM unless the balance module is accessed.
        unimplemented!()
    }

    fn total_amount(&self, _account: AccountAddress) -> Result<u128, Self::Error> {
        // TODO(rqnsom): This won't break the MoveVM unless the balance module is accessed.
        unimplemented!()
    }
}
