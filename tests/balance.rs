mod assets;
mod mock;

use frame_support::assert_ok;
use mock::*;
use move_core_types::language_storage::TypeTag;
use move_vm_backend::{balance::BalanceHandler, types::MAX_GAS_AMOUNT};
use pallet_move::{balance::BalanceAdapter, transaction::Transaction};

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
            AMOUNT
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
        );

        assert_ok!(res);
    })
}
