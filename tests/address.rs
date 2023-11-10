//! Address conversion utilities based on the Pontem address solution.
//! To properly handle Move VM addresses and Substrate addresses, we need to convert them to each other.
pub(crate) mod mock;

#[cfg(test)]
mod account_convert_tests {
    use move_core_types::account_address::AccountAddress;
    use sp_core::{crypto::Ss58Codec, sr25519::Public};

    use super::mock::*;

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
        new_test_ext().execute_with(|| {
            for pair in DATASET.iter() {
                let pk = Public::from_ss58check_with_version(pair.0).unwrap().0;
                let addr = MoveModule::native_to_move(&pk.into()).unwrap();
                assert_eq!(pair.1, addr.to_string());
            }
        })
    }

    #[test]
    fn to_substrate_account_check() {
        new_test_ext().execute_with(|| {
            for pair in DATASET.iter() {
                let pk_expected = Public::from_ss58check_with_version(pair.0)
                    .unwrap()
                    .0
                    .into();
                let addr = MoveModule::native_to_move(&pk_expected).unwrap();
                let pk_decoded = MoveModule::move_to_native(&addr).expect("Cannot decode address");
                assert_eq!(pk_expected, pk_decoded);
            }
        })
    }

    #[test]
    fn account_to_bytes_check() {
        for pair in DATASET.iter() {
            let pk = Public::from_ss58check_with_version(pair.0).unwrap().0;
            let bytes = MoveModule::native_to_move(&pk.into()).unwrap().into_bytes();
            assert_eq!(pair.1, hex::encode(bytes));
            let bytes_expected = AccountAddress::from_hex_literal(&format!("0x{}", pair.1))
                .unwrap()
                .to_vec();
            assert_eq!(bytes_expected, bytes);
        }
    }
}
