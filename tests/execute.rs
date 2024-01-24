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
use sp_runtime::AccountId32;

fn get_vm_resource(
    module_owner: &AccountAddress,
    module_id: &str,
    key_id: &str,
    address: &AccountId32,
) -> Result<Option<Vec<u8>>, anyhow::Error> {
    let tag = StructTag {
        address: *module_owner,
        module: Identifier::new(module_id)?,
        name: Identifier::new(key_id)?,
        type_params: vec![],
    };
    let bytes = bcs::to_bytes(&tag)?;
    let o_vec = MoveModule::get_resource(address, &bytes)
        .map_err(|e| anyhow::anyhow!("MoveModule::get_resource {:?}", e))?;
    Ok(o_vec)
}

fn transaction_bc_for_create_counter_script(
    project: &str,
    script: &str,
    mv_addr: &AccountAddress,
) -> Result<Vec<u8>, anyhow::Error> {
    let param = bcs::to_bytes(mv_addr)?;
    let params: Vec<&[u8]> = vec![&param];
    let script = assets::read_script_from_project(project, script);
    let transaction = Transaction {
        script_bc: script,
        type_args: Vec::<TypeTag>::new(),
        args: params.iter().map(|x| x.to_vec()).collect(),
    };
    Ok(bcs::to_bytes(&transaction)?)
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
            0,
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
            0,
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
            0,
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
            0,
        );

        assert_ok!(res);
    });
}

#[test]
/// Test execution of a script with correct parameters which stores something inside the storage.
fn execute_script_storage_correct() {
    pub const ALICE_ADDR: &str = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";

    let (bob_addr_32, bob_addr_mv) = addrs_from_ss58(BOB_ADDR).unwrap();
    let (alice_addr_32, alice_addr_mv) = addrs_from_ss58(ALICE_ADDR).unwrap();

    new_test_ext().execute_with(|| {
        // Bob publishes the move-module 'Counter', test preparation.
        let module = assets::read_module_from_project("get-resource", "Counter");
        assert_ok!(MoveModule::publish_module(
            RuntimeOrigin::signed(bob_addr_32.clone()),
            module,
            MAX_GAS_AMOUNT,
        ));

        // Check that there are no counters available.
        assert!(
            get_vm_resource(&bob_addr_mv, "Counter", "Counter", &alice_addr_32)
                .unwrap()
                .is_none()
        );
        assert!(
            get_vm_resource(&bob_addr_mv, "Counter", "Counter", &bob_addr_32)
                .unwrap()
                .is_none()
        );

        // Alice and Bob execute a script to create a counter using the move-module 'Counter'.
        let transaction_bc = transaction_bc_for_create_counter_script(
            "get-resource",
            "create_counter",
            &alice_addr_mv,
        )
        .unwrap();
        assert_ok!(MoveModule::execute(
            RuntimeOrigin::signed(alice_addr_32.clone()),
            transaction_bc,
            MAX_GAS_AMOUNT,
            0,
        ));

        let transaction_bc = transaction_bc_for_create_counter_script(
            "get-resource",
            "create_counter",
            &bob_addr_mv,
        )
        .unwrap();
        assert_ok!(MoveModule::execute(
            RuntimeOrigin::signed(bob_addr_32.clone()),
            transaction_bc,
            MAX_GAS_AMOUNT,
            0,
        ));

        // Verify counter has been created.
        let counter = get_vm_resource(&bob_addr_mv, "Counter", "Counter", &alice_addr_32)
            .unwrap()
            .expect("couldn't find Alice's counter");
        assert_eq!(counter, vec![0, 0, 0, 0, 0, 0, 0, 0]);

        let counter = get_vm_resource(&bob_addr_mv, "Counter", "Counter", &bob_addr_32)
            .unwrap()
            .expect("couldn't find Bob's counter");
        assert_eq!(counter, vec![0, 0, 0, 0, 0, 0, 0, 0]);

        // Execute script that counts that created counter, but only for Alice.
        let transaction_bc =
            transaction_bc_for_create_counter_script("get-resource", "count", &alice_addr_mv)
                .unwrap();
        assert_ok!(MoveModule::execute(
            RuntimeOrigin::signed(alice_addr_32.clone()),
            transaction_bc,
            MAX_GAS_AMOUNT,
            0,
        ));

        // Verify counter has been increased by 1.
        let counter = get_vm_resource(&bob_addr_mv, "Counter", "Counter", &alice_addr_32)
            .unwrap()
            .expect("couldn't find Alice's counter");
        assert_eq!(counter, vec![1, 0, 0, 0, 0, 0, 0, 0]);
        // Verify counter has still the same value.
        let counter = get_vm_resource(&bob_addr_mv, "Counter", "Counter", &bob_addr_32)
            .unwrap()
            .expect("couldn't find Bob's counter");
        assert_eq!(counter, vec![0, 0, 0, 0, 0, 0, 0, 0]);
    });
}

#[test]
/// Test execution of a script with correct parameters but with insufficient gas.
fn execute_script_insufficient_gas() {
    let (bob_addr_32, bob_addr_mv) = addrs_from_ss58(BOB_ADDR).unwrap();

    new_test_ext().execute_with(|| {
        // Bob publishes the move-module 'Counter', test preparation.
        let module = assets::read_module_from_project("get-resource", "Counter");
        assert_ok!(MoveModule::publish_module(
            RuntimeOrigin::signed(bob_addr_32.clone()),
            module,
            MAX_GAS_AMOUNT,
        ));

        // Bob wants to execute a script which shall trigger that module but with too little gas.
        let transaction_bc = transaction_bc_for_create_counter_script(
            "get-resource",
            "create_counter",
            &bob_addr_mv,
        )
        .unwrap();
        assert!(MoveModule::execute(
            RuntimeOrigin::signed(bob_addr_32.clone()),
            transaction_bc,
            10,
            0,
        )
        .is_err());
    });
}

#[test]
/// Test execution of a script with corrupted bytecode.
fn execute_script_corrupted_bytecode() {
    let (bob_addr_32, bob_addr_mv) = addrs_from_ss58(BOB_ADDR).unwrap();

    new_test_ext().execute_with(|| {
        // Bob publishes the move-module 'Counter', test preparation.
        let module = assets::read_module_from_project("get-resource", "Counter");
        assert_ok!(MoveModule::publish_module(
            RuntimeOrigin::signed(bob_addr_32.clone()),
            module,
            MAX_GAS_AMOUNT,
        ));

        // Bob executes a corrupted script.
        let mut transaction_bc = transaction_bc_for_create_counter_script(
            "get-resource",
            "create_counter",
            &bob_addr_mv,
        )
        .unwrap();
        transaction_bc[10] += 1;
        assert!(MoveModule::execute(
            RuntimeOrigin::signed(bob_addr_32.clone()),
            transaction_bc,
            MAX_GAS_AMOUNT,
            0,
        )
        .is_err());
    });
}
