// Address conversion utilities based on the Pontem address solution.
// To properly handle Move VM addresses and Substrate addresses, we need to convert them to each other.

use codec::{Decode, Encode, Error};
use move_core_types::account_address::AccountAddress;

/// Convert Move VM address instance (AccountAddress) to an AccountId.
///
/// Returns an AccountId instance or decoding error.
pub fn address_to_account<AccountId>(address: &AccountAddress) -> Result<AccountId, Error>
where
    AccountId: Decode + Sized,
{
    AccountId::decode(&mut address.as_ref())
}

/// Convert AccountId to Move VM address format.
///
/// Returns an AccountAddress instance - shouldn't fail as any bytes with length 32 could be
/// represented as an AccountAddress.
///
/// ```
/// use pallet_move::address::account_to_address;
/// use sp_core::sr25519::Public;
/// use sp_core::crypto::Ss58Codec;
///
/// let address = account_to_address(& Public::from_ss58check_with_version("gkQ5K6EnLRgZkwozG8GiBAEnJyM6FxzbSaSmVhKJ2w8FcK7ih").unwrap().0);
/// assert_eq!(address.to_string(), "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d");
/// ```
pub fn account_to_address<AccountId>(account: &AccountId) -> AccountAddress
where
    AccountId: Encode,
{
    AccountAddress::new(account_to_bytes(account))
}

/// Convert AccountId to byte format which is compatible with Move VM address.
///
/// In fact, returned value can be just passed to the AccountAddress constructor and it will
/// create a valid Move VM address.
/// This function is ready to handle different than 32 bytes long AccountId instances.
/// For the MoveVM port, we need to have a 32 bytes long address.
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

    result[skip..].copy_from_slice(&bytes);

    result
}

#[cfg(test)]
mod tests {
    use assert_hex::assert_eq_hex;
    use sp_core::{crypto::Ss58Codec, sr25519::Public};

    use super::{account_to_address, address_to_account, AccountAddress};

    const DATASET: &[(&str, &[u8; 32]); 6] = &[
        (
            "gkQ5K6EnLRgZkwozG8GiBAEnJyM6FxzbSaSmVhKJ2w8FcK7ih",
            &[
                0xD4, 0x35, 0x93, 0xC7, 0x15, 0xFD, 0xD3, 0x1C, 0x61, 0x14, 0x1A, 0xBD, 0x04, 0xA9,
                0x9F, 0xD6, 0x82, 0x2C, 0x85, 0x58, 0x85, 0x4C, 0xCD, 0xE3, 0x9A, 0x56, 0x84, 0xE7,
                0xA5, 0x6D, 0xA2, 0x7D,
            ],
        ),
        (
            "gkNW9pAcCHxZrnoVkhLkEQtsLsW5NWTC75cdAdxAMs9LNYCYg",
            &[
                0x8E, 0xAF, 0x04, 0x15, 0x16, 0x87, 0x73, 0x63, 0x26, 0xC9, 0xFE, 0xA1, 0x7E, 0x25,
                0xFC, 0x52, 0x87, 0x61, 0x36, 0x93, 0xC9, 0x12, 0x90, 0x9C, 0xB2, 0x26, 0xAA, 0x47,
                0x94, 0xF2, 0x6A, 0x48,
            ],
        ),
        (
            "gkKH52LJ2UumhVBim1n3mCsSj3ctj3GkV8JLVLdhJakxmEDcq",
            &[
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x01,
            ],
        ),
        (
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
            &[
                0xd4, 0x35, 0x93, 0xc7, 0x15, 0xfd, 0xd3, 0x1c, 0x61, 0x14, 0x1a, 0xbd, 0x04, 0xa9,
                0x9f, 0xd6, 0x82, 0x2c, 0x85, 0x58, 0x85, 0x4c, 0xcd, 0xe3, 0x9a, 0x56, 0x84, 0xe7,
                0xa5, 0x6d, 0xa2, 0x7d,
            ],
        ),
        (
            "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
            &[
                0x8e, 0xaf, 0x4, 0x15, 0x16, 0x87, 0x73, 0x63, 0x26, 0xc9, 0xfe, 0xa1, 0x7e, 0x25,
                0xfc, 0x52, 0x87, 0x61, 0x36, 0x93, 0xc9, 0x12, 0x90, 0x9c, 0xb2, 0x26, 0xaa, 0x47,
                0x94, 0xf2, 0x6a, 0x48,
            ],
        ),
        (
            "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y",
            &[
                0x90, 0xb5, 0xab, 0x20, 0x5c, 0x69, 0x74, 0xc9, 0xea, 0x84, 0x1b, 0xe6, 0x88, 0x86,
                0x46, 0x33, 0xdc, 0x9c, 0xa8, 0xa3, 0x57, 0x84, 0x3e, 0xea, 0xcf, 0x23, 0x14, 0x64,
                0x99, 0x65, 0xfe, 0x22,
            ],
        ),
    ];

    #[test]
    fn account_to_address_check() {
        for pair in DATASET.iter() {
            let pk = Public::from_ss58check_with_version(pair.0).unwrap().0;
            let addr = account_to_address(&pk);
            assert_eq_hex!(pair.1, addr.as_ref());
        }
    }

    #[test]
    fn address_to_account_check() {
        for pair in DATASET.iter() {
            let pk_expected = Public::from_ss58check_with_version(pair.0).unwrap().0;
            let addr = account_to_address(&pk_expected);
            let pk_decoded = address_to_account(&addr).expect("Cannot decode address");
            assert_eq_hex!(pk_expected, pk_decoded);
        }
    }

    #[test]
    fn account_to_bytes_check() {
        for pair in DATASET.iter() {
            let pk = Public::from_ss58check_with_version(pair.0).unwrap().0;
            let bytes = super::account_to_bytes(&pk);
            assert_eq_hex!(pair.1, bytes.as_ref());
            let bytes_expected = AccountAddress::from_bytes(pair.1).unwrap().to_vec();
            assert_eq_hex!(bytes_expected, bytes);
        }
    }
}
