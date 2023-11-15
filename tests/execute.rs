mod mock;
use mock::*;
use move_core_types::account_address::AccountAddress;
use sp_core::offchain::{testing, OffchainWorkerExt, TransactionPoolExt};
use sp_keystore::{testing::MemoryKeystore, Keystore, KeystoreExt};
use sp_runtime::KeyTypeId;

const MOVE: &str = "0x90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22";
const PHRASE: &str = "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
const TEST_ID: KeyTypeId = KeyTypeId([1u8; 4]);

#[test]
#[ignore = "to be implemented"]
/// Test execution of a script with correct parameters.
fn execute_script_correct() {
    new_test_ext().execute_with(|| {
        assert_eq!(1, 0);
    });
}

#[test]
#[ignore = "to be implemented"]
/// Test execution of a script with correct parameters which stores something inside the storage.
fn execute_script_storage_correct() {
    new_test_ext().execute_with(|| {
        assert_eq!(1, 0);
    });
}

#[test]
#[ignore = "to be implemented"]
/// Test execution of a script with correct parameters but as a wrong user.
/// Transaction does not require sudo but call was signed with sudo.
fn execute_script_wrong_user() {
    new_test_ext().execute_with(|| {
        assert_eq!(1, 0);
    });
}

#[test]
#[ignore = "to be implemented"]
/// Test execution of a script with correct parameters but with insufficient gas.
fn execute_script_insufficient_gas() {
    new_test_ext().execute_with(|| {
        assert_eq!(1, 0);
    });
}

#[test]
#[ignore = "to be implemented"]
/// Test execution of a script with corrupted bytecode.
fn execute_script_corrupted_bytecode() {
    new_test_ext().execute_with(|| {
        assert_eq!(1, 0);
    });
}

#[test]
fn round_conversion_native_move_works() {
    new_test_ext().execute_with(|| {
        let native =
            MoveModule::move_to_native(&AccountAddress::from_hex_literal(MOVE).unwrap()).unwrap();
        let move_again = MoveModule::native_to_move(&native).unwrap();
        assert_eq!(MOVE, move_again.to_hex_literal());
    })
}

#[test]
fn offline_client_bad_inputs_emmits_correct_error_events() {
    let (offchain, offchain_state) = testing::TestOffchainExt::new();
    let (pool, pool_state) = testing::TestTransactionPoolExt::new();

    let keystore = MemoryKeystore::new();

    keystore
        .sr25519_generate_new(TEST_ID, Some(&format!("{}/hunter1", PHRASE)))
        .unwrap();

    let public_key = *keystore.sr25519_public_keys(TEST_ID).get(0).unwrap();
    let mut t = sp_io::TestExternalities::default();
    t.register_extension(OffchainWorkerExt::new(offchain));
    t.register_extension(TransactionPoolExt::new(pool));
    t.register_extension(KeystoreExt::new(keystore));
    new_test_ext().execute_with(|| {});
}
