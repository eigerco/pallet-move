use core::marker::PhantomData;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::RuntimeDebug, traits::Get, BoundedBTreeMap, Parameter};
use frame_system::Config as SysConfig;
use scale_info::TypeInfo;
use sp_runtime::traits::MaybeSerializeDeserialize;

use crate::{
    balance::{BalanceAdapter, BalanceOf},
    Config, Error,
};

/// This definition stores the hash value of a script transaction.
pub type CallHash = [u8; 32];

/// A simple signature.
#[derive(Clone, Eq, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum Signature {
    /// Signer has approved the script with the signature.
    Approved,
    /// Signer's approval still missing.
    #[default]
    Missing,
}

#[derive(Clone, Eq, PartialEq)]
pub enum MultisigError {
    MaxSignersExceeded,
    ScriptSignatureFailure,
    UnableToDeserializeAccount,
    UserHasAlreadySigned,
}

impl<T> From<MultisigError> for Error<T> {
    fn from(err: MultisigError) -> Self {
        match err {
            MultisigError::MaxSignersExceeded => Error::<T>::MaxSignersExceeded,
            MultisigError::ScriptSignatureFailure => Error::<T>::ScriptSignatureFailure,
            MultisigError::UnableToDeserializeAccount => Error::<T>::UnableToDeserializeAccount,
            MultisigError::UserHasAlreadySigned => Error::<T>::UserHasAlreadySigned,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct SignerData {
    /// State of the user's signature.
    pub signature: Signature,
    /// Individual cheque-limit.
    pub cheque_limit: u128,
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(Size))]
pub struct Multisig<AccountId, Size>(BoundedBTreeMap<AccountId, SignerData, Size>)
where
    AccountId: Parameter + Ord + MaybeSerializeDeserialize,
    Size: Get<u32>;

impl<AccountId, Size> Multisig<AccountId, Size>
where
    AccountId: Parameter + Ord + MaybeSerializeDeserialize,
    Size: Get<u32>,
{
    /// Creates a new [`Multisig`] with all blank signatures for the provided script.
    pub fn new(signers: Vec<AccountId>) -> Result<Self, MultisigError> {
        if signers.len() > (Size::get() as usize) {
            return Err(MultisigError::MaxSignersExceeded);
        }

        let mut multisig_info = Multisig::<AccountId, Size>::default();
        for account in signers.iter() {
            multisig_info
                .try_insert(account.clone(), SignerData::default())
                .map_err(|_| MultisigError::UnableToDeserializeAccount)?;
        }

        Ok(multisig_info)
    }
}

impl<AccountId, Size> core::ops::Deref for Multisig<AccountId, Size>
where
    AccountId: Parameter + Ord + MaybeSerializeDeserialize,
    Size: Get<u32>,
{
    type Target = BoundedBTreeMap<AccountId, SignerData, Size>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<AccountId, Size> core::ops::DerefMut for Multisig<AccountId, Size>
where
    AccountId: Parameter + Ord + MaybeSerializeDeserialize,
    Size: Get<u32>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Because in substrate_move::AccountAddress Default impl is missing.
impl<AccountId, Size> Default for Multisig<AccountId, Size>
where
    AccountId: Parameter + Ord + MaybeSerializeDeserialize,
    Size: Get<u32>,
{
    fn default() -> Self {
        Multisig(BoundedBTreeMap::<AccountId, SignerData, Size>::new())
    }
}

/// Script signature handler tracks required signatures for the provided script.
#[derive(Clone, Eq, PartialEq)]
pub(crate) struct ScriptSignatureHandler<T>
where
    T: Config + SysConfig,
    T::AccountId: Parameter + Ord + MaybeSerializeDeserialize,
    BalanceOf<T>: From<u128> + Into<u128>,
{
    _pd_config: PhantomData<T>,
    /// All required multisig_info.
    multisig_info: Multisig<T::AccountId, T::MaxScriptSigners>,
}

impl<T> ScriptSignatureHandler<T>
where
    T: Config + SysConfig,
    T::AccountId: Parameter + Ord + MaybeSerializeDeserialize,
    BalanceOf<T>: From<u128> + Into<u128>,
{
    /// Creates a new [`ScriptSignatureHandler`] with all blank signatures for the provided script.
    pub(crate) fn new(accounts: Vec<T::AccountId>) -> Result<Self, Error<T>> {
        Ok(Self {
            _pd_config: PhantomData,
            multisig_info: Multisig::<T::AccountId, T::MaxScriptSigners>::new(accounts)
                .map_err(Into::<Error<T>>::into)?,
        })
    }

    /// Provide a signature to this script.
    ///
    /// In case the script doesn't require the signature, or it doesn't require a signature from
    /// the current signer, the signature shall be ignored, and no error will be thrown for simplicity reasons.
    ///
    /// In case the signer is not the one who should sign the script, the signature shall be stored
    /// until all necessary signatures are collected.
    pub(crate) fn sign_script(
        &mut self,
        account: &T::AccountId,
        cheque_limit: u128,
    ) -> Result<(), Error<T>> {
        if let Some(ms_data) = self.multisig_info.get_mut(account) {
            if matches!(ms_data.signature, Signature::Approved) {
                Err(Error::<T>::UserHasAlreadySigned)
            } else {
                ms_data.signature = Signature::Approved;
                ms_data.cheque_limit = cheque_limit;
                Ok(())
            }
        } else {
            Err(Error::<T>::ScriptSignatureFailure)
        }
    }

    /// Check whether the script has been approved by all required signers.
    pub(crate) fn all_signers_approved(&self) -> bool {
        self.multisig_info
            .values()
            .all(|signer| signer.signature == Signature::Approved)
    }

    /// Creates a [`BalanceAdapter`] from the internal stored cheque-limits.
    /// Function returns an error if not all signers have signed.
    pub(crate) fn write_cheques(&self) -> Result<BalanceAdapter<T>, Error<T>> {
        if !self.all_signers_approved() {
            return Err(Error::<T>::ScriptSignatureFailure);
        }

        let mut balances = BalanceAdapter::<T>::new();
        for (account, ms_data) in self.multisig_info.iter() {
            balances
                .write_cheque(account, &BalanceOf::<T>::from(ms_data.cheque_limit))
                .map_err(|_| Error::<T>::InsufficientBalance)?;
        }

        Ok(balances)
    }

    /// Consumes [`ScriptSignatureHandler`] and returns innner `Multisig`.
    pub(crate) fn into_inner(self) -> Multisig<T::AccountId, T::MaxScriptSigners> {
        self.multisig_info
    }

    /// Consumes [`ScriptSignatureHandler`] and returns accounts of all signers as vector.
    pub(crate) fn into_signer_accounts(self) -> Result<Vec<T::AccountId>, Error<T>> {
        let accounts: Vec<T::AccountId> = self.multisig_info.keys().cloned().collect();
        Ok(accounts)
    }
}

impl<T> From<Multisig<T::AccountId, T::MaxScriptSigners>> for ScriptSignatureHandler<T>
where
    T: Config + SysConfig,
    BalanceOf<T>: From<u128> + Into<u128>,
{
    fn from(multisig_info: Multisig<T::AccountId, T::MaxScriptSigners>) -> Self {
        Self {
            _pd_config: PhantomData,
            multisig_info,
        }
    }
}
