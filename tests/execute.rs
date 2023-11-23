mod mock;
use codec::Decode;
use frame_support::{assert_ok, traits::OffchainWorker};
use mock::*;
use move_core_types::{
    account_address::AccountAddress, language_storage::TypeTag, value::MoveValue,
};
use move_vm_backend::deposit::{CHECK_BALANCE_OF_SCRIPT_BYTES, DEPOSIT_SCRIPT_BYTES};
use pallet_move::{
    transaction::Transaction, Event, ModulesToPublish, ScriptsToExecute, SessionTransferToken,
};
use sp_core::blake2_128;

const MOVE: &str = "0x90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22";
const NOT_A_MODULE: &str = "garbage data";

fn get_account<T: pallet_move::Config>() -> T::AccountId {
    T::AccountId::decode(&mut [1u8; 32].to_vec().as_ref()).unwrap()
}

#[test]
/// Test execution of a script.
fn execute_script_empty() {
    new_test_ext().execute_with(|| {
        let module =
            include_bytes!("assets/move/build/move/bytecode_scripts/empty_scr.mv").to_vec();

        let type_args: Vec<TypeTag> = vec![];
        let params: Vec<&[u8]> = vec![];

        let transaction = Transaction {
            script_bc: module.clone(),
            type_args,
            args: params.iter().map(|x| x.to_vec()).collect(),
        };

        let transaction_bc = bcs::to_bytes(&transaction).unwrap();

        let res = MoveModule::execute(
            RuntimeOrigin::signed(get_account::<Test>()),
            transaction_bc,
            true,
            0,
        );

        assert_ok!(res);

        let module =
            include_bytes!("assets/move/build/move/bytecode_scripts/empty_loop.mv").to_vec();

        let type_args: Vec<TypeTag> = vec![];
        let params: Vec<&[u8]> = vec![];

        let transaction = Transaction {
            script_bc: module.clone(),
            type_args,
            args: params.iter().map(|x| x.to_vec()).collect(),
        };

        let transaction_bc = bcs::to_bytes(&transaction).unwrap();

        let res = MoveModule::execute(
            RuntimeOrigin::signed(get_account::<Test>()),
            transaction_bc,
            true,
            0,
        );

        assert_ok!(res);
    });
}

#[test]
/// Test execution of a script with parametrized function.
fn execute_script_params() {
    new_test_ext().execute_with(|| {
        let module =
            include_bytes!("assets/move/build/move/bytecode_scripts/empty_loop_param.mv").to_vec();

        let iter_count = bcs::to_bytes(&10u64).unwrap();
        let type_args: Vec<TypeTag> = vec![];
        let params: Vec<&[u8]> = vec![&iter_count];

        let transaction = Transaction {
            script_bc: module.clone(),
            type_args,
            args: params.iter().map(|x| x.to_vec()).collect(),
        };

        let transaction_bc = bcs::to_bytes(&transaction).unwrap();

        let res = MoveModule::execute(
            RuntimeOrigin::signed(get_account::<Test>()),
            transaction_bc,
            true,
            0,
        );

        assert_ok!(res);
    });
}

#[test]
/// Test execution of a script with generic function.
fn execute_script_generic() {
    new_test_ext().execute_with(|| {
        let module =
            include_bytes!("assets/move/build/move/bytecode_scripts/generic_1.mv").to_vec();

        let param = bcs::to_bytes(&100u64).unwrap();
        let type_args: Vec<TypeTag> = vec![TypeTag::U64];
        let params: Vec<&[u8]> = vec![&param];

        let transaction = Transaction {
            script_bc: module.clone(),
            type_args,
            args: params.iter().map(|x| x.to_vec()).collect(),
        };

        let transaction_bc = bcs::to_bytes(&transaction).unwrap();

        let res = MoveModule::execute(
            RuntimeOrigin::signed(get_account::<Test>()),
            transaction_bc,
            true,
            0,
        );

        assert_ok!(res);
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
fn deposit_script_transfer_works() {
    new_test_ext().execute_with(|| {
        let dest = get_account::<Test>();
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
        let args = vec![
            bcs::to_bytes(&MoveValue::Signer(
                MoveModule::native_to_move(&user).unwrap(),
            ))
            .unwrap(),
            bcs::to_bytes(&MoveValue::Address(
                MoveModule::native_to_move(&dest).unwrap(),
            ))
            .unwrap(),
            bcs::to_bytes(&MoveValue::U128(123u128)).unwrap(),
        ];
        let transaction = Transaction {
            script_bc: DEPOSIT_SCRIPT_BYTES.to_vec(),
            args,
            type_args: vec![],
        };
        // transfer script
        let encoded = bcs::to_bytes(&transaction).unwrap();
        let script_id = u128::from_be_bytes(blake2_128(encoded.as_ref()));
        ScriptsToExecute::<Test>::insert(user.clone(), script_id, encoded);
        // Grant one transfer for account to transfer
        SessionTransferToken::<Test>::insert(script_id, user.clone());
        frame_system::Pallet::<Test>::set_block_number(1);
        MoveModule::offchain_worker(1u64);
        // verify
        assert_last_event(
            Event::<Test>::ExecuteScriptResult {
                publisher: user.clone(),
                script: script_id,
                status: pallet_move::ScriptExecutionStatus::Success,
            }
            .into(),
        );
        assert_eq!(
            <Test as pallet_move::Config>::Currency::free_balance(&dest),
            10123u128
        );
        // check with move script throug SubstrateApi
        let args = vec![
            bcs::to_bytes(&MoveValue::Address(
                MoveModule::native_to_move(&dest).unwrap(),
            ))
            .unwrap(),
            bcs::to_bytes(&MoveValue::U128(10123u128)).unwrap(),
        ];
        let get_balance_transaction = bcs::to_bytes(&Transaction {
            script_bc: CHECK_BALANCE_OF_SCRIPT_BYTES.to_vec(),
            args,
            type_args: vec![],
        })
        .unwrap();
        let balance_script_id = u128::from_be_bytes(blake2_128(&get_balance_transaction));
        SessionTransferToken::<Test>::insert(balance_script_id, user.clone());
        ScriptsToExecute::<Test>::insert(user.clone(), balance_script_id, get_balance_transaction);
        frame_system::Pallet::<Test>::set_block_number(2);
        MoveModule::offchain_worker(2u64);
        // verify
        assert_last_event(
            Event::<Test>::ExecuteScriptResult {
                publisher: user.clone(),
                script: balance_script_id,
                status: pallet_move::ScriptExecutionStatus::Success,
            }
            .into(),
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

#[test]
fn deposit_script_should_fail_test() {
    new_test_ext().execute_with(|| {
        // setup, but partial - no pre-requirements from pallet side fulfilled yet
        let dest = get_account::<Test>();
        let user =
            MoveModule::move_to_native(&AccountAddress::from_hex_literal(MOVE).unwrap()).unwrap();
        let args = vec![
            bcs::to_bytes(&MoveValue::Signer(
                MoveModule::native_to_move(&user).unwrap(),
            ))
            .unwrap(),
            bcs::to_bytes(&MoveValue::Address(
                MoveModule::native_to_move(&dest).unwrap(),
            ))
            .unwrap(),
            bcs::to_bytes(&MoveValue::U128(123u128)).unwrap(),
        ];
        let transaction = Transaction {
            script_bc: DEPOSIT_SCRIPT_BYTES.to_vec(),
            args,
            type_args: vec![],
        };
        // transfer script
        let encoded = bcs::to_bytes(&transaction).unwrap();
        let script_id = u128::from_be_bytes(blake2_128(encoded.as_ref()));
        // insert script
        ScriptsToExecute::<Test>::insert(user.clone(), script_id, encoded.clone());
        frame_system::Pallet::<Test>::set_block_number(1);
        MoveModule::offchain_worker(1u64);
        // verify no script AND no token for transfer left
        assert!(SessionTransferToken::<Test>::get(script_id).is_none());
        assert!(ScriptsToExecute::<Test>::get(&user, script_id).is_none());
        // Expected failure #1 - no transfer token
        assert_last_event(
            Event::<Test>::ExecuteScriptResult {
                publisher: user.clone(),
                script: script_id,
                status: pallet_move::ScriptExecutionStatus::Failure(
                    "No session token for account \
                    5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y and \
                    script 1906774120454276922783716490891208755"
                        .into(),
                ),
            }
            .into(),
        );
        // Grant one transfer for account to transfer
        SessionTransferToken::<Test>::insert(script_id, user.clone());
        // insert script
        ScriptsToExecute::<Test>::insert(user.clone(), script_id, encoded.clone());
        frame_system::Pallet::<Test>::set_block_number(2);
        MoveModule::offchain_worker(2u64);
        // verify no script AND no token for transfer left
        assert!(SessionTransferToken::<Test>::get(script_id).is_none());
        assert!(ScriptsToExecute::<Test>::get(&user, script_id).is_none());
        // Expected failure #2 - not enough funds to transfer
        assert_last_event(
            Event::<Test>::ExecuteScriptResult {
                publisher: user.clone(),
                script: script_id,
                status: pallet_move::ScriptExecutionStatus::Failure("InsuficientBalance".into()),
            }
            .into(),
        );
        // set sender balance
        assert_ok!(<Test as pallet_move::Config>::Currency::force_set_balance(
            RuntimeOrigin::root(),
            user.clone(),
            100,
        ));
        // set destination balance
        assert_ok!(<Test as pallet_move::Config>::Currency::force_set_balance(
            RuntimeOrigin::root(),
            dest.clone(),
            10000,
        ));
        // Grant one transfer for account to transfer
        SessionTransferToken::<Test>::insert(script_id, user.clone());
        // insert script
        ScriptsToExecute::<Test>::insert(user.clone(), script_id, encoded);
        frame_system::Pallet::<Test>::set_block_number(3);
        MoveModule::offchain_worker(3u64);
        // verify no script AND no token for transfer left
        assert!(SessionTransferToken::<Test>::get(script_id).is_none());
        assert!(ScriptsToExecute::<Test>::get(&user, script_id).is_none());
        // Expected failure #3 - not enough fund with non-zero balance
        assert_last_event(
            Event::<Test>::ExecuteScriptResult {
                publisher: user.clone(),
                script: script_id,
                status: pallet_move::ScriptExecutionStatus::Failure("InsuficientBalance".into()),
            }
            .into(),
        );
    });
}
