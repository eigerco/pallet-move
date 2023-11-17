mod mock;
use frame_support::{assert_ok, traits::OffchainWorker};
use mock::*;
use move_core_types::account_address::AccountAddress;
use pallet_move::{Event, ModulesToPublish, ScriptsToExecute};
use sp_core::blake2_128;

const MOVE: &str = "0x90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22";
const NOT_A_MODULE: &str = "garbage data";

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
fn offline_client_deposit_module_publish_works() {
    new_test_ext().execute_with(|| {
        frame_system::Pallet::<Test>::set_block_number(1);
        MoveModule::offchain_worker(1u64);
        assert_last_event(Event::<Test>::StdModulePublished.into());
    });
}

#[test]
// TODO: un-panic after transfer PR is merged
fn deposit_script_transfer_works() {
    new_test_ext().execute_with(|| {
        use codec::Decode;
        let dest =
            <Test as frame_system::Config>::AccountId::decode(&mut [1u8; 32].as_ref()).unwrap();
        // set destination balance
        assert_ok!(<Test as pallet_move::Config>::Currency::force_set_balance(
            RuntimeOrigin::root(),
            dest.clone(),
            10000,
        ));
        let user =
            MoveModule::move_to_native(&AccountAddress::from_hex_literal(MOVE).unwrap()).unwrap();
        // set sender balance
        assert_ok!(<Test as pallet_move::Config>::Currency::force_set_balance(
            RuntimeOrigin::root(),
            user.clone(),
            10000,
        ));
        frame_system::Pallet::<Test>::set_block_number(1);
        // transfer script
        use move_vm_backend::deposit::DEPOSIT_SCRIPT_BYTES;
        let script_id = u128::from_be_bytes(blake2_128(DEPOSIT_SCRIPT_BYTES.as_ref()));
        ScriptsToExecute::<Test>::insert(user.clone(), script_id, DEPOSIT_SCRIPT_BYTES.to_vec());
        MoveModule::offchain_worker(1u64);
        assert_last_event(
            Event::<Test>::ExecuteScriptResult {
                publisher: user,
                script: script_id,
                status: pallet_move::ScriptExecutionStatus::Suceess,
            }
            .into(),
        );
        assert_eq!(
            <Test as pallet_move::Config>::Currency::free_balance(&dest),
            10123u128
        );
    });
}

#[test]
fn offline_client_bad_inputs_emmits_correct_error_events() {
    new_test_ext().execute_with(|| {
        let user =
            MoveModule::move_to_native(&AccountAddress::from_hex_literal(MOVE).unwrap()).unwrap();
        let module_id = u128::from_be_bytes(blake2_128(NOT_A_MODULE.as_bytes()));
        ModulesToPublish::<Test>::insert(user.clone(), module_id, NOT_A_MODULE.as_bytes());
        assert!(ModulesToPublish::<Test>::contains_key(
            user.clone(),
            module_id
        ));
        frame_system::Pallet::<Test>::set_block_number(1);
        MoveModule::offchain_worker(1u64);
        assert_last_event(
            Event::<Test>::PublishModuleResult {
                publisher: user.clone(),
                module: module_id,
                status: pallet_move::ModulePublishStatus::Failure(
                    "Error code:BAD_MAGIC: msg: ''".into(),
                ),
            }
            .into(),
        );
        frame_system::Pallet::<Test>::set_block_number(2);
        // make sure it's purged
        assert!(!ModulesToPublish::<Test>::contains_key(user, module_id));
    });
}
