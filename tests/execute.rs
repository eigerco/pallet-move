mod mock;
use frame_support::assert_ok;
use mock::*;
use move_core_types::language_storage::TypeTag;
use pallet_move::transaction::Transaction;

const INFINITE_GAS: u64 = u64::MAX;

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
            RuntimeOrigin::signed(0xFECA000000000000),
            transaction_bc,
            INFINITE_GAS,
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
            RuntimeOrigin::signed(0xFECA000000000000),
            transaction_bc,
            INFINITE_GAS,
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
            RuntimeOrigin::signed(0xFECA000000000000),
            transaction_bc,
            INFINITE_GAS,
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
            RuntimeOrigin::signed(0xFECA000000000000),
            transaction_bc,
            INFINITE_GAS,
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
