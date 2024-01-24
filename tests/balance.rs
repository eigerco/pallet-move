mod assets;
mod mock;

use frame_support::assert_ok;
use mock::*;
use move_core_types::language_storage::TypeTag;
use move_vm_backend::{balance::BalanceHandler, types::MAX_GAS_AMOUNT};
use pallet_move::{
    balance::{BalanceAdapter, BalanceOf},
    transaction::Transaction,
};

#[test]
fn verify_get_balance() {
    new_test_ext().execute_with(|| {
        let addr_native = BOB_ADDR_NATIVE.clone();
        let addr_move = *BOB_ADDR_MOVE;
        const AMOUNT: u128 = EXISTENTIAL_DEPOSIT + 100;

        // Set Bob's balance to a predefined value
        assert_ok!(Balances::force_set_balance(
            RuntimeOrigin::root(),
            addr_native.clone(),
            AMOUNT,
        ));

        // Check the pallet side first.
        let balance: BalanceAdapter<Test> = BalanceAdapter::new();
        assert_eq!(balance.total_amount(addr_move).unwrap(), AMOUNT);

        // Now check that it works from within the MoveVM.
        let script = assets::read_script_from_project("balance", "verify_preconfigured_balance");

        let account = bcs::to_bytes(&addr_move).unwrap();
        let preconfigured_amount = bcs::to_bytes(&AMOUNT).unwrap();
        let params: Vec<&[u8]> = vec![&account, &preconfigured_amount];
        let type_args: Vec<TypeTag> = vec![];

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
    })
}

#[test]
fn verify_simple_transfer() {
    const AMOUNT: u128 = 100;

    let (alice_addr_32, alice_addr_mv) = addrs_from_ss58(ALICE_ADDR).unwrap();
    let (bob_addr_32, bob_addr_mv) = addrs_from_ss58(BOB_ADDR).unwrap();

    new_test_ext().execute_with(|| {
        // Set Alice's balance to a predefined value
        assert_ok!(Balances::force_set_balance(
            RuntimeOrigin::root(),
            alice_addr_32.clone(),
            10000,
        ));

        // Check initial state of balances of involved users.
        let ini_blnc_alice = Balances::free_balance(&alice_addr_32);
        let ini_blnc_bob = Balances::free_balance(&bob_addr_32);

        // Now check that it works from within the MoveVM.
        let script = assets::read_script_from_project("balance", "single_transfer");

        let src = bcs::to_bytes(&alice_addr_mv).unwrap();
        let dst = bcs::to_bytes(&bob_addr_mv).unwrap();
        let amount = bcs::to_bytes(&AMOUNT).unwrap();
        let params: Vec<&[u8]> = vec![&src, &dst, &amount];
        let transaction = Transaction {
            script_bc: script,
            type_args: Vec::<TypeTag>::new(),
            args: params.iter().map(|x| x.to_vec()).collect(),
        };
        let transaction_bc = bcs::to_bytes(&transaction).unwrap();

        assert_ok!(MoveModule::execute(
            RuntimeOrigin::signed(alice_addr_32.clone()),
            transaction_bc,
            MAX_GAS_AMOUNT,
            1500,
        ));

        let now_blnc_alice = Balances::free_balance(&alice_addr_32);
        let now_blnc_bob = Balances::free_balance(&bob_addr_32);
        assert_eq!(ini_blnc_alice - AMOUNT, now_blnc_alice);
        assert_eq!(ini_blnc_bob + AMOUNT, now_blnc_bob);
    })
}

#[test]
fn verify_multiple_transfers_different() {
    const AMOUNT: u128 = 100;

    let (alice_addr_32, alice_addr_mv) = addrs_from_ss58(ALICE_ADDR).unwrap();
    let (bob_addr_32, bob_addr_mv) = addrs_from_ss58(BOB_ADDR).unwrap();
    let (dave_addr_32, dave_addr_mv) = addrs_from_ss58(DAVE_ADDR).unwrap();

    new_test_ext().execute_with(|| {
        // Set Alice's balance to a predefined value
        assert_ok!(Balances::force_set_balance(
            RuntimeOrigin::root(),
            alice_addr_32.clone(),
            10000,
        ));

        // Check initial state of balances of involved users.
        let ini_blnc_alice = Balances::free_balance(&alice_addr_32);
        let ini_blnc_bob = Balances::free_balance(&bob_addr_32);
        let ini_blnc_dave = Balances::free_balance(&dave_addr_32);

        // Now check that it works from within the MoveVM.
        let script = assets::read_script_from_project("balance", "double_transfer");

        let src = bcs::to_bytes(&alice_addr_mv).unwrap();
        let dst1 = bcs::to_bytes(&bob_addr_mv).unwrap();
        let dst2 = bcs::to_bytes(&dave_addr_mv).unwrap();
        let amount = bcs::to_bytes(&AMOUNT).unwrap();
        let params: Vec<&[u8]> = vec![&src, &dst1, &amount, &dst2, &amount];
        let transaction = Transaction {
            script_bc: script,
            type_args: Vec::<TypeTag>::new(),
            args: params.iter().map(|x| x.to_vec()).collect(),
        };
        let transaction_bc = bcs::to_bytes(&transaction).unwrap();

        assert_ok!(MoveModule::execute(
            RuntimeOrigin::signed(alice_addr_32.clone()),
            transaction_bc,
            MAX_GAS_AMOUNT,
            1500,
        ));

        let now_blnc_alice = Balances::free_balance(&alice_addr_32);
        let now_blnc_bob = Balances::free_balance(&bob_addr_32);
        let now_blnc_dave = Balances::free_balance(&dave_addr_32);
        assert_eq!(ini_blnc_alice - AMOUNT * 2, now_blnc_alice);
        assert_eq!(ini_blnc_bob + AMOUNT, now_blnc_bob);
        assert_eq!(ini_blnc_dave + AMOUNT, now_blnc_dave);
    })
}

#[test]
fn verify_multiple_transfers_same() {
    const AMOUNT: u128 = 100;

    let (alice_addr_32, alice_addr_mv) = addrs_from_ss58(ALICE_ADDR).unwrap();
    let (bob_addr_32, bob_addr_mv) = addrs_from_ss58(BOB_ADDR).unwrap();

    new_test_ext().execute_with(|| {
        // Set Alice's balance to a predefined value
        assert_ok!(Balances::force_set_balance(
            RuntimeOrigin::root(),
            alice_addr_32.clone(),
            10000,
        ));

        // Check initial state of balances of involved users.
        let ini_blnc_alice = Balances::free_balance(&alice_addr_32);
        let ini_blnc_bob = Balances::free_balance(&bob_addr_32);

        // Now check that it works from within the MoveVM.
        let script = assets::read_script_from_project("balance", "double_transfer");

        let src = bcs::to_bytes(&alice_addr_mv).unwrap();
        let dst = bcs::to_bytes(&bob_addr_mv).unwrap();
        let amount = bcs::to_bytes(&AMOUNT).unwrap();
        let params: Vec<&[u8]> = vec![&src, &dst, &amount, &dst, &amount];
        let transaction = Transaction {
            script_bc: script,
            type_args: Vec::<TypeTag>::new(),
            args: params.iter().map(|x| x.to_vec()).collect(),
        };
        let transaction_bc = bcs::to_bytes(&transaction).unwrap();

        assert_ok!(MoveModule::execute(
            RuntimeOrigin::signed(alice_addr_32.clone()),
            transaction_bc,
            MAX_GAS_AMOUNT,
            1500,
        ));

        let now_blnc_alice = Balances::free_balance(&alice_addr_32);
        let now_blnc_bob = Balances::free_balance(&bob_addr_32);
        assert_eq!(ini_blnc_alice - AMOUNT * 2, now_blnc_alice);
        assert_eq!(ini_blnc_bob + AMOUNT * 2, now_blnc_bob);
    })
}

#[test]
fn verify_balance_limit_too_low() {
    const AMOUNT: BalanceOf<Test> = 100;

    let (alice_addr_32, alice_addr_mv) = addrs_from_ss58(ALICE_ADDR).unwrap();
    let (_, bob_addr_mv) = addrs_from_ss58(BOB_ADDR).unwrap();

    new_test_ext().execute_with(|| {
        // Set Alice's balance to a predefined value
        assert_ok!(Balances::force_set_balance(
            RuntimeOrigin::root(),
            alice_addr_32.clone(),
            10000,
        ));

        // Now check that it works from within the MoveVM.
        let script = assets::read_script_from_project("balance", "single_transfer");

        let src = bcs::to_bytes(&alice_addr_mv).unwrap();
        let dst = bcs::to_bytes(&bob_addr_mv).unwrap();
        let amount = bcs::to_bytes(&AMOUNT).unwrap();
        let params: Vec<&[u8]> = vec![&src, &dst, &amount];
        let transaction = Transaction {
            script_bc: script,
            type_args: Vec::<TypeTag>::new(),
            args: params.iter().map(|x| x.to_vec()).collect(),
        };
        let transaction_bc = bcs::to_bytes(&transaction).unwrap();

        assert!(MoveModule::execute(
            RuntimeOrigin::signed(alice_addr_32.clone()),
            transaction_bc,
            MAX_GAS_AMOUNT,
            AMOUNT - 1,
        )
        .is_err());
    })
}

#[test]
fn verify_insufficient_balance() {
    const AMOUNT: BalanceOf<Test> = 100;

    let (alice_addr_32, alice_addr_mv) = addrs_from_ss58(ALICE_ADDR).unwrap();
    let (_, bob_addr_mv) = addrs_from_ss58(BOB_ADDR).unwrap();

    new_test_ext().execute_with(|| {
        let script = assets::read_script_from_project("balance", "single_transfer");

        let src = bcs::to_bytes(&bob_addr_mv).unwrap();
        let dst = bcs::to_bytes(&alice_addr_mv).unwrap();
        let amount = bcs::to_bytes(&AMOUNT).unwrap();
        let params: Vec<&[u8]> = vec![&src, &dst, &amount];
        let transaction = Transaction {
            script_bc: script,
            type_args: Vec::<TypeTag>::new(),
            args: params.iter().map(|x| x.to_vec()).collect(),
        };
        let transaction_bc = bcs::to_bytes(&transaction).unwrap();

        assert!(MoveModule::execute(
            RuntimeOrigin::signed(alice_addr_32.clone()),
            transaction_bc,
            MAX_GAS_AMOUNT,
            AMOUNT,
        )
        .is_err());
    })
}

#[test]
fn verify_move_script_fails_after_successful_transfer() {
    const BALANCE: BalanceOf<Test> = 1000;
    const AMOUNT: BalanceOf<Test> = 100;

    let (alice_addr_32, alice_addr_mv) = addrs_from_ss58(ALICE_ADDR).unwrap();
    let (bob_addr_32, bob_addr_mv) = addrs_from_ss58(BOB_ADDR).unwrap();

    new_test_ext().execute_with(|| {
        // Set Alice's and Bob's balances to a predefined value.
        assert_ok!(Balances::force_set_balance(
            RuntimeOrigin::root(),
            alice_addr_32.clone(),
            BALANCE,
        ));
        assert_ok!(Balances::force_set_balance(
            RuntimeOrigin::root(),
            bob_addr_32.clone(),
            BALANCE,
        ));

        // Execute script with a successful transfer but which fails after transfer.
        let script = assets::read_script_from_project("balance", "fail_at_the_end");

        let src = bcs::to_bytes(&alice_addr_mv).unwrap();
        let dst = bcs::to_bytes(&bob_addr_mv).unwrap();
        let amount = bcs::to_bytes(&AMOUNT).unwrap();
        let params: Vec<&[u8]> = vec![&src, &dst, &amount];
        let transaction = Transaction {
            script_bc: script,
            type_args: Vec::<TypeTag>::new(),
            args: params.iter().map(|x| x.to_vec()).collect(),
        };
        let transaction_bc = bcs::to_bytes(&transaction).unwrap();

        // Expect error because script will fail at the end.
        assert!(MoveModule::execute(
            RuntimeOrigin::signed(alice_addr_32.clone()),
            transaction_bc,
            MAX_GAS_AMOUNT,
            AMOUNT,
        )
        .is_err());

        // Verify balances have not been modified and transfer was not applied.
        let now_blnc_alice = Balances::free_balance(&alice_addr_32);
        let now_blnc_bob = Balances::free_balance(&bob_addr_32);
        assert_eq!(now_blnc_alice, BALANCE);
        assert_eq!(now_blnc_bob, BALANCE);
    })
}

#[test]
fn verify_self_transfer() {
    const AMOUNT: BalanceOf<Test> = 100;

    let (alice_addr_32, alice_addr_mv) = addrs_from_ss58(ALICE_ADDR).unwrap();

    new_test_ext().execute_with(|| {
        assert_ok!(Balances::force_set_balance(
            RuntimeOrigin::root(),
            alice_addr_32.clone(),
            AMOUNT * 2,
        ));

        let script = assets::read_script_from_project("balance", "single_transfer");

        let src = bcs::to_bytes(&alice_addr_mv).unwrap();
        let dst = bcs::to_bytes(&alice_addr_mv).unwrap();
        let amount = bcs::to_bytes(&AMOUNT).unwrap();
        let params: Vec<&[u8]> = vec![&src, &dst, &amount];
        let transaction = Transaction {
            script_bc: script,
            type_args: Vec::<TypeTag>::new(),
            args: params.iter().map(|x| x.to_vec()).collect(),
        };
        let transaction_bc = bcs::to_bytes(&transaction).unwrap();

        assert_ok!(MoveModule::execute(
            RuntimeOrigin::signed(alice_addr_32.clone()),
            transaction_bc,
            MAX_GAS_AMOUNT,
            AMOUNT,
        ));

        let now_blnc_alice = Balances::free_balance(&alice_addr_32);
        assert_eq!(now_blnc_alice, AMOUNT * 2);
    })
}

#[test]
fn verify_self_transfer_trying_to_cheat() {
    const AMOUNT: BalanceOf<Test> = 1000;
    const BALANCE: BalanceOf<Test> = 100;

    let (alice_addr_32, alice_addr_mv) = addrs_from_ss58(ALICE_ADDR).unwrap();

    new_test_ext().execute_with(|| {
        assert_ok!(Balances::force_set_balance(
            RuntimeOrigin::root(),
            alice_addr_32.clone(),
            BALANCE,
        ));

        let script = assets::read_script_from_project("balance", "single_transfer");

        let src = bcs::to_bytes(&alice_addr_mv).unwrap();
        let dst = bcs::to_bytes(&alice_addr_mv).unwrap();
        let amount = bcs::to_bytes(&AMOUNT).unwrap();
        let params: Vec<&[u8]> = vec![&src, &dst, &amount];
        let transaction = Transaction {
            script_bc: script,
            type_args: Vec::<TypeTag>::new(),
            args: params.iter().map(|x| x.to_vec()).collect(),
        };
        let transaction_bc = bcs::to_bytes(&transaction).unwrap();

        assert!(MoveModule::execute(
            RuntimeOrigin::signed(alice_addr_32.clone()),
            transaction_bc,
            MAX_GAS_AMOUNT,
            AMOUNT,
        )
        .is_err());

        let now_blnc_alice = Balances::free_balance(&alice_addr_32);
        assert_eq!(now_blnc_alice, BALANCE);
    })
}
