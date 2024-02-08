mod assets;
mod mock;

use frame_support::assert_ok;
use mock::*;
use move_core_types::{account_address::AccountAddress, language_storage::TypeTag};
use move_vm_backend::types::MAX_GAS_AMOUNT;
use move_vm_backend_common::types::ScriptTransaction;

const PROJECT: &str = "car-wash-example";
const COIN_PRICE: u128 = 1_000_000_000_000;

fn script_bytecode(name: &str, acc: AccountAddress) -> Vec<u8> {
    let script = assets::read_script_from_project(PROJECT, name);
    let account = bcs::to_bytes(&acc).unwrap();
    let params: Vec<&[u8]> = vec![&account];
    let transaction = ScriptTransaction {
        bytecode: script,
        type_args: Vec::<TypeTag>::new(),
        args: params.iter().map(|x| x.to_vec()).collect(),
    };
    bcs::to_bytes(&transaction).unwrap()
}

/// Test the regular, ideal flow of our example project.
#[test]
fn verify_normal_use_case() {
    let (alice_addr_32, alice_addr_mv) = addrs_from_ss58(ALICE_ADDR).unwrap();
    let (bob_addr_32, bob_addr_mv) = addrs_from_ss58(BOB_ADDR).unwrap();

    new_test_ext().execute_with(|| {
        // Set Alice's and Bob's balance to a predefined value
        assert_ok!(Balances::force_set_balance(
            RuntimeOrigin::root(),
            alice_addr_32.clone(),
            5_000_000_000_000,
        ));
        assert_ok!(Balances::force_set_balance(
            RuntimeOrigin::root(),
            alice_addr_32.clone(),
            10_000_000_000_000,
        ));

        // Check initial state of balances of involved users.
        let ini_blnc_alice = Balances::free_balance(&alice_addr_32);
        let ini_blnc_bob = Balances::free_balance(&bob_addr_32);

        // Let's publish Bob's module CarWash.
        let module_bc = assets::read_module_from_project(PROJECT, "CarWash");
        assert_ok!(MoveModule::publish_module(
            RuntimeOrigin::signed(bob_addr_32.clone()),
            module_bc,
            MAX_GAS_AMOUNT,
        ));

        // Now Bob initialises his module.
        let script_bc = script_bytecode("initial_coin_minting", bob_addr_mv);
        assert_ok!(MoveModule::execute(
            RuntimeOrigin::signed(bob_addr_32.clone()),
            script_bc,
            MAX_GAS_AMOUNT,
            0,
        ));

        // Now Alice comes over to wash her car for the first time...
        let script_bc = script_bytecode("register_new_user", alice_addr_mv);
        assert_ok!(MoveModule::execute(
            RuntimeOrigin::signed(alice_addr_32.clone()),
            script_bc,
            MAX_GAS_AMOUNT,
            0,
        ));

        let script = assets::read_script_from_project(PROJECT, "buy_coin");
        let account = bcs::to_bytes(&alice_addr_mv).unwrap();
        let coin_count = bcs::to_bytes(&1u8).unwrap();
        let params: Vec<&[u8]> = vec![&account, &coin_count];
        let transaction = ScriptTransaction {
            bytecode: script,
            type_args: Vec::<TypeTag>::new(),
            args: params.iter().map(|x| x.to_vec()).collect(),
        };
        let script_bc = bcs::to_bytes(&transaction).unwrap();
        assert_ok!(MoveModule::execute(
            RuntimeOrigin::signed(alice_addr_32.clone()),
            script_bc,
            MAX_GAS_AMOUNT,
            COIN_PRICE,
        ));

        let script_bc = script_bytecode("wash_car", alice_addr_mv);
        assert_ok!(MoveModule::execute(
            RuntimeOrigin::signed(alice_addr_32.clone()),
            script_bc,
            MAX_GAS_AMOUNT,
            0,
        ));

        let now_blnc_alice = Balances::free_balance(&alice_addr_32);
        let now_blnc_bob = Balances::free_balance(&bob_addr_32);
        assert_eq!(ini_blnc_alice - COIN_PRICE, now_blnc_alice);
        assert_eq!(ini_blnc_bob + COIN_PRICE, now_blnc_bob);
    })
}
