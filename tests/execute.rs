mod assets;
mod mock;

use frame_support::assert_ok;
use mock::*;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
};
use move_vm_backend::types::MAX_GAS_AMOUNT;
use pallet_move::transaction::Transaction;
use sp_core::{crypto::Ss58Codec, sr25519::Public};
use sp_runtime::AccountId32;

fn addr32_from_ss58(ss58addr: &str) -> AccountId32 {
    let (pk, _) = Public::from_ss58check_with_version(ss58addr).unwrap();
    pk.into()
}

fn addr32_to_move(addr32: &AccountId32) -> Result<AccountAddress, pallet_move::Error<Test>> {
    MoveModule::to_move_address(&addr32)
}

fn move_script_to_transaction(
    project: &str,
    script: &str,
    type_generic_args: Vec<TypeTag>,
    param: Vec<u8>,
) -> Vec<u8> {
    let params: Vec<&[u8]> = vec![&param];

    let script = assets::read_script_from_project(project, script);
    let transaction = Transaction {
        script_bc: script,
        type_args: type_generic_args,
        args: params.iter().map(|x| x.to_vec()).collect(),
    };
    bcs::to_bytes(&transaction).unwrap()
}

fn get_vm_resource(
    module_owner: &AccountId32,
    module_id: &str,
    key_id: &str,
    address: &AccountAddress,
) -> Result<Option<Vec<u8>>, Vec<u8>> {
    let tag = StructTag {
        address: address.clone(),
        module: Identifier::new(module_id).unwrap(),
        name: Identifier::new(key_id).unwrap(),
        type_params: vec![],
    };
    let bytes = bcs::to_bytes(&tag).unwrap();
    MoveModule::get_resource(module_owner, &bytes)
}

fn script_transaction(script: &str, mv_addr: &AccountAddress) -> Vec<u8> {
    let param = bcs::to_bytes(mv_addr).unwrap();
    move_script_to_transaction("get-resource", script, Vec::<TypeTag>::new(), param)
}

#[test]
/// Test execution of a script.
fn execute_script_empty() {
    new_test_ext().execute_with(|| {
        let addr_native = CAFE_ADDR_NATIVE.clone();

        let script = assets::read_script_from_project("move-basics", "empty_scr");

        let type_args: Vec<TypeTag> = vec![];
        let params: Vec<&[u8]> = vec![];

        let transaction = Transaction {
            script_bc: script,
            type_args,
            args: params.iter().map(|x| x.to_vec()).collect(),
        };

        let transaction_bc = bcs::to_bytes(&transaction).unwrap();

        let res = MoveModule::execute(
            RuntimeOrigin::signed(addr_native.clone()),
            transaction_bc,
            MAX_GAS_AMOUNT,
        );

        assert_ok!(res);

        let script = assets::read_script_from_project("move-basics", "empty_loop");

        let type_args: Vec<TypeTag> = vec![];
        let params: Vec<&[u8]> = vec![];

        let transaction = Transaction {
            script_bc: script,
            type_args,
            args: params.iter().map(|x| x.to_vec()).collect(),
        };

        let transaction_bc = bcs::to_bytes(&transaction).unwrap();

        let res = MoveModule::execute(
            RuntimeOrigin::signed(addr_native),
            transaction_bc,
            MAX_GAS_AMOUNT,
        );

        assert_ok!(res);
    });
}

#[test]
/// Test execution of a script with parametrized function.
fn execute_script_params() {
    new_test_ext().execute_with(|| {
        let addr_native = CAFE_ADDR_NATIVE.clone();

        let script = assets::read_script_from_project("move-basics", "empty_loop_param");

        let iter_count = bcs::to_bytes(&10u64).unwrap();
        let type_args: Vec<TypeTag> = vec![];
        let params: Vec<&[u8]> = vec![&iter_count];

        let transaction = Transaction {
            script_bc: script,
            type_args,
            args: params.iter().map(|x| x.to_vec()).collect(),
        };

        let transaction_bc = bcs::to_bytes(&transaction).unwrap();

        let res = MoveModule::execute(
            RuntimeOrigin::signed(addr_native),
            transaction_bc,
            MAX_GAS_AMOUNT,
        );

        assert_ok!(res);
    });
}

#[test]
/// Test execution of a script with generic function.
fn execute_script_generic() {
    new_test_ext().execute_with(|| {
        let addr_native = CAFE_ADDR_NATIVE.clone();

        let script = assets::read_script_from_project("move-basics", "generic_1");

        let param = bcs::to_bytes(&100u64).unwrap();
        let type_args: Vec<TypeTag> = vec![TypeTag::U64];
        let params: Vec<&[u8]> = vec![&param];

        let transaction = Transaction {
            script_bc: script,
            type_args,
            args: params.iter().map(|x| x.to_vec()).collect(),
        };

        let transaction_bc = bcs::to_bytes(&transaction).unwrap();

        let res = MoveModule::execute(
            RuntimeOrigin::signed(addr_native),
            transaction_bc,
            MAX_GAS_AMOUNT,
        );

        assert_ok!(res);
    });
}

#[test]
/// Test execution of a script with correct parameters which stores something inside the storage.
fn execute_script_storage_correct() {
    pub const ALICE_ADDR: &str = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";

    let bob_addr_32 = addr32_from_ss58(BOB_ADDR);
    let bob_addr_mv = addr32_to_move(&bob_addr_32).unwrap();
    let alice_addr_32 = addr32_from_ss58(ALICE_ADDR);
    let alice_addr_mv = addr32_to_move(&bob_addr_32).unwrap();


    new_test_ext().execute_with(|| {
        // Bob publishes the move-module 'Counter', test preparation.
        let module = assets::read_module_from_project("get-resource", "Counter");
        assert_ok!(MoveModule::publish_module(
            RuntimeOrigin::signed(bob_addr_32.clone()),
            module,
            MAX_GAS_AMOUNT,
        ));

        // Check, that there are no counters available.
        assert_eq!(
            get_vm_resource(&bob_addr_32, "Counter", "Counter", &alice_addr_mv).unwrap(),
            None
        );
        assert_eq!(
            get_vm_resource(&bob_addr_32, "Counter", "Counter", &bob_addr_mv).unwrap(),
            None
        );

        // Alice and Bob execute a script to create a counter by using move-module 'Counter'.
        let transaction_bc = script_transaction("create_counter", &alice_addr_mv);
        assert_ok!(MoveModule::execute(
            RuntimeOrigin::signed(alice_addr_32.clone()),
            transaction_bc,
            MAX_GAS_AMOUNT,
        ));

        // TODO: Should be possible to execute this also
        // let transaction_bc = gen_script_trans("create_counter", &bob_addr_mv);
        // assert_ok!(MoveModule::execute(
        //     RuntimeOrigin::signed(bob_addr_32.clone()),
        //     transaction_bc,
        //     MAX_GAS_AMOUNT,
        // ));

        // Verify counter has been created.
        let counter = get_vm_resource(&bob_addr_32, "Counter", "Counter", &alice_addr_mv)
            .unwrap()
            .expect("Couldn't find Alice's counter");
        assert_eq!(counter, vec![0, 0, 0, 0, 0, 0, 0, 0]);

        // TODO: Should be possible to execute this also
        // let counter = get_vm_resource(&bob_addr_32, "Counter", "Counter", &bob_addr_mv)
        //     .unwrap()
        //     .expect("Couldn't find Bob's counter");
        // assert_eq!(counter, vec![0, 0, 0, 0, 0, 0, 0, 0]);

        // Execute script that counts that created counter.
        let transaction_bc = script_transaction("count", &alice_addr_mv);
        assert_ok!(MoveModule::execute(
            RuntimeOrigin::signed(alice_addr_32.clone()),
            transaction_bc,
            MAX_GAS_AMOUNT,
        ));

        // Verify counter has been increased by 1.
        let counter = get_vm_resource(&bob_addr_32, "Counter", "Counter", &alice_addr_mv)
            .unwrap()
            .expect("Could not find Alice's counter");
        assert_eq!(counter, vec![1, 0, 0, 0, 0, 0, 0, 0]);

        // TODO: Should be possible to execute this also
        // let counter = get_vm_resource(&bob_addr_32, "Counter", "Counter", &bob_addr_mv)
        //     .unwrap()
        //     .expect("Couldn't find Bob's counter");
        // assert_eq!(counter, vec![0, 0, 0, 0, 0, 0, 0, 0]);
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
