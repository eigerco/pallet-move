use core::marker::PhantomData;

use hashbrown::HashMap;
use move_core_types::account_address::AccountAddress;

use crate::{balance::BalanceOf, Config, Error, Pallet};

/// A simple signature.
#[derive(PartialEq, Eq)]
enum Signature {
    /// Signer has approved the script with the signature.
    Approved,
    /// Signer's approval still missing.
    Missing,
}

/// Script signature handler tracks required signatures for the provided script.
pub(crate) struct ScriptSignatureHandler<T: Config>
where
    BalanceOf<T>: From<u128> + Into<u128>,
{
    _pd_config: PhantomData<T>,

    /// All required signers.
    signers: HashMap<AccountAddress, Signature>,
}

impl<T: Config> ScriptSignatureHandler<T>
where
    BalanceOf<T>: From<u128> + Into<u128>,
{
    /// Creates a new [`ScriptSignatureHandler`] with all blank signatures for the provided script.
    pub(crate) fn new(script_args: &[Vec<u8>], signer_count: usize) -> Result<Self, Error<T>> {
        if signer_count > script_args.len() {
            return Err(Error::ScriptSignatureFailure);
        }

        let mut signers = HashMap::new();
        for signer in &script_args[..signer_count] {
            let account_address =
                bcs::from_bytes(signer).map_err(|_| Error::UnableToDeserializeAccount)?;
            signers.insert(account_address, Signature::Missing);
        }

        Ok(Self {
            _pd_config: PhantomData,
            signers,
        })
    }

    /// Provide a signature to this script.
    ///
    /// In case the script doesn't require the signature, or it doesn't require a signature from
    /// the current signer, the signature shall be ignored, and no error will be thrown for simplicity reasons.
    ///
    /// In case the signer is not the one who should sign the script, the signature shall be stored
    /// until all necessary signatures are collected.
    pub(crate) fn sign_script(&mut self, account: &T::AccountId) -> Result<(), Error<T>> {
        let account_address = Pallet::<T>::to_move_address(account)?;

        if let Some(signature) = self.signers.get_mut(&account_address) {
            *signature = Signature::Approved;
        }

        Ok(())
    }

    /// Check whether the script has been approved by all required signers.
    pub(crate) fn all_signers_approved(&self) -> bool {
        self.signers
            .values()
            .all(|signature| *signature == Signature::Approved)
    }
}
