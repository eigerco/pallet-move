use core::marker::PhantomData;

use frame_support::{sp_runtime::SaturatedConversion, traits::Currency};
use move_core_types::{account_address::AccountAddress, vm_status::StatusCode};
use move_vm_backend::balance::BalanceHandler;

use crate::{Config, Error, Pallet};

/// Converts Error to StatusCode.
///
/// This is a rare direction for the error conversion, but it's needed for the balance handler
/// module which is injected into the MoveVM. If anything goes wrong in the [`BalanceHandler`]
/// implementation, we should be able to map the internal Substrate error into [`StatusCode`].
impl<T: Config> From<Error<T>> for StatusCode {
    fn from(err: Error<T>) -> Self {
        match err {
            Error::InsufficientBalance => Self::INSUFFICIENT_BALANCE,
            Error::InvalidAccountSize => Self::UNABLE_TO_DESERIALIZE_ACCOUNT,
            // If we ever see this one, we should update the function here.
            //_ => Self::INTERNAL_TYPE_ERROR,
            // TODO(eiger): Use the above commented code before the final release.
            _ => unreachable!("WARNING: update the conversion function"),
        }
    }
}

/// Balance adapter for providing basic access to balance cheques within the MoveVM.
#[derive(Clone)]
pub struct BalanceAdapter<T: Config> {
    _pd_config: PhantomData<T>,
}

impl<T: Config> BalanceAdapter<T> {
    /// Create a new [`BalanceAdapter`].
    pub fn new() -> Self {
        BalanceAdapter {
            _pd_config: PhantomData,
        }
    }
}

impl<T: Config> Default for BalanceAdapter<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Config> BalanceHandler for BalanceAdapter<T> {
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

    fn total_amount(&self, account: AccountAddress) -> Result<u128, Self::Error> {
        let native_account: T::AccountId =
            Pallet::<T>::to_native_account(&account).map_err(Into::<Self::Error>::into)?;

        let amount = T::Currency::free_balance(&native_account).saturated_into::<u128>();
        Ok(amount)
    }
}
