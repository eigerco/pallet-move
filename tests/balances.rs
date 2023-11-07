mod mock;

use frame_support::{assert_err, assert_ok};
use mock::*;
use sp_runtime::{traits::BadOrigin, AccountId32, ArithmeticError};

const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
const BOB: AccountId32 = AccountId32::new([2u8; 32]);

#[test]
// Test transfering with existing balance
fn transfer_move_valid_amounts() {
    new_test_ext().execute_with(|| {
        const INITIAL: u128 = 1_000_000;
        const SENDING: u128 = INITIAL / 10;
        let signed_alice = RuntimeOrigin::signed(ALICE);
        // Set Alice balance to predefined vaule
        assert_ok!(Balances::force_set_balance(
            RuntimeOrigin::root(),
            ALICE,
            INITIAL
        ));
        let bob_move = MoveModule::native_to_move(&BOB).unwrap();
        // make sure Bob's balance is 0
        assert_eq!(0, MoveModule::get_balance(BOB));
        // Send 'SENDING' to Bob
        assert_ok!(MoveModule::transfer(
            signed_alice,
            bob_move.into_bytes(),
            SENDING
        ));
        // verify
        assert_eq!(SENDING, MoveModule::get_balance(BOB));
        // Alice balance reduced
        assert_eq!(MoveModule::get_balance(ALICE), INITIAL - SENDING);
        // Move balances match
        assert_eq!(SENDING, MoveModule::get_move_balance(&bob_move).unwrap());
        assert_eq!(
            MoveModule::get_move_balance(&MoveModule::native_to_move(&ALICE).unwrap()).unwrap(),
            INITIAL - SENDING
        );
    })
}

#[test]
// Invalid cases should fail
fn invalid_balances_transfer_move_fails() {
    new_test_ext().execute_with(|| {
        let bob_move = MoveModule::native_to_move(&BOB).unwrap();
        // root not accepted
        assert_err!(
            MoveModule::transfer(RuntimeOrigin::root(), bob_move.into_bytes(), u128::MAX),
            BadOrigin
        );
        // signed but not enough balance
        let signed_alice = RuntimeOrigin::signed(ALICE);
        assert_err!(
            MoveModule::transfer(signed_alice.clone(), bob_move.into_bytes(), 100000),
            ArithmeticError::Underflow
        );
        // non-zero balance but sending more than free
        const INITIAL: u128 = 1_000_000;
        const SENDING: u128 = INITIAL * 10;
        // Set Alice balance to predefined vaule
        assert_ok!(Balances::force_set_balance(
            RuntimeOrigin::root(),
            ALICE,
            INITIAL
        ));
        assert_err!(
            MoveModule::transfer(signed_alice, bob_move.into_bytes(), SENDING),
            ArithmeticError::Underflow
        );
    })
}
