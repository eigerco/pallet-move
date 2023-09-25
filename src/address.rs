//! Address conversion utilities based on the Pontem address solution.
//! To properly handle Move VM addresses and Substrate addresses, we need to convert them to each other.

//TODO(asmie): general TODO - investigate if there is something better than AccountId
// described there: https://paritytech.github.io/polkadot-sdk/master/frame_system/pallet/trait.Config.html#associatedtype.AccountId

use codec::{Decode, Encode, Error};
use move_core_types::account_address::AccountAddress;

/// Convert Move VM address instance (AccountAddress) to an AccountId.
///
/// Returns an AccountId instance or decoding error.
pub fn to_substrate_account<AccountId>(move_address: &AccountAddress) -> Result<AccountId, Error>
where
    AccountId: Decode + Sized,
{
    AccountId::decode(&mut move_address.as_ref())
}

/// Convert AccountId to Move VM address format.
///
/// Returns an AccountAddress instance - shouldn't fail as any bytes with length 32 could be
/// represented as an AccountAddress.
///
/// ```
/// use pallet_move::address::to_move_address;
/// use sp_core::sr25519::Public;
/// use sp_core::crypto::Ss58Codec;
///
/// let address = to_move_address(& Public::from_ss58check_with_version("gkQ5K6EnLRgZkwozG8GiBAEnJyM6FxzbSaSmVhKJ2w8FcK7ih").unwrap().0);
/// assert_eq!(address.to_string(), "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d");
/// ```
pub fn to_move_address<AccountId>(substrate_account: &AccountId) -> AccountAddress
where
    AccountId: Encode,
{
    AccountAddress::new(account_to_bytes(substrate_account))
}

/// Convert AccountId to byte format which is compatible with Move VM address.
///
/// In fact, returned value can be just passed to the AccountAddress constructor and it will
/// create a valid Move VM address.
/// This function is ready to handle different than 32 bytes long AccountId instances.
/// For the MoveVM port, we need to have a 32 bytes long address.
/// Panic on bad account length.
///
/// ```
/// use pallet_move::address::account_to_bytes;
/// use sp_core::sr25519::Public;
/// use sp_core::crypto::Ss58Codec;
///
/// let bytes = account_to_bytes(& Public::from_ss58check_with_version("gkQ5K6EnLRgZkwozG8GiBAEnJyM6FxzbSaSmVhKJ2w8FcK7ih").unwrap().0);
/// assert_eq!(bytes.as_ref(), &[0xD4, 0x35, 0x93, 0xC7, 0x15, 0xFD, 0xD3, 0x1C, 0x61, 0x14, 0x1A, 0xBD, 0x04, 0xA9, 0x9F, 0xD6,
///                 0x82, 0x2C, 0x85, 0x58, 0x85, 0x4C, 0xCD, 0xE3, 0x9A, 0x56, 0x84, 0xE7, 0xA5, 0x6D, 0xA2, 0x7D]);
/// ```
pub fn account_to_bytes<AccountId>(account: &AccountId) -> [u8; AccountAddress::LENGTH]
where
    AccountId: Encode,
{
    const LENGTH: usize = AccountAddress::LENGTH;
    let mut result = [0; LENGTH];
    let bytes = account.encode();

    let skip = LENGTH.saturating_sub(bytes.len());

    assert_eq!(
        LENGTH,
        bytes.len() + skip,
        "Substrate account address can't be larger than Move address"
    );

    result[skip..].copy_from_slice(&bytes);

    result
}

#[cfg(test)]
mod tests {
    use sp_core::{crypto::Ss58Codec, sr25519::Public};

    use super::{account_to_bytes, to_move_address, to_substrate_account, AccountAddress};

    const DATASET: &[(&str, &str); 6] = &[
        (
            "gkQ5K6EnLRgZkwozG8GiBAEnJyM6FxzbSaSmVhKJ2w8FcK7ih",
            "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
        ),
        (
            "gkNW9pAcCHxZrnoVkhLkEQtsLsW5NWTC75cdAdxAMs9LNYCYg",
            "8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48",
        ),
        (
            "gkKH52LJ2UumhVBim1n3mCsSj3ctj3GkV8JLVLdhJakxmEDcq",
            "0000000000000000000000000000000000000000000000000000000000000001",
        ),
        (
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
            "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
        ),
        (
            "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
            "8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48",
        ),
        (
            "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y",
            "90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22",
        ),
    ];

    #[test]
    fn to_move_address_check() {
        for pair in DATASET.iter() {
            let pk = Public::from_ss58check_with_version(pair.0).unwrap().0;
            let addr = to_move_address(&pk);
            assert_eq!(pair.1, addr.to_string());
        }
    }

    #[test]
    fn to_substrate_account_check() {
        for pair in DATASET.iter() {
            let pk_expected = Public::from_ss58check_with_version(pair.0).unwrap().0;
            let addr = to_move_address(&pk_expected);
            let pk_decoded = to_substrate_account(&addr).expect("Cannot decode address");
            assert_eq!(pk_expected, pk_decoded);
        }
    }

    #[test]
    fn account_to_bytes_check() {
        for pair in DATASET.iter() {
            let pk = Public::from_ss58check_with_version(pair.0).unwrap().0;
            let bytes = account_to_bytes(&pk);
            assert_eq!(pair.1, hex::encode(bytes));
            let bytes_expected = AccountAddress::from_hex_literal(&format!("0x{}", pair.1))
                .unwrap()
                .to_vec();
            assert_eq!(bytes_expected, bytes);
        }
    }

    #[test]
    #[should_panic]
    fn account_to_bytes_panic() {
        let test_vec = vec![0; 64];
        let _bytes = account_to_bytes(&test_vec);
    }
}
