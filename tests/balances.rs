mod mock;

use frame_support::assert_ok;
use mock::*;
use sp_runtime::AccountId32;

const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
const BOB: AccountId32 = AccountId32::new([2u8; 32]);

#[test]
/// Test getting a module.
fn transfer_move() {
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
        let bob_move = MoveModule::native_to_move(BOB).unwrap();
        // make sure Bob's balance is 0
        assert_eq!(0, MoveModule::get_balance(BOB).unwrap());
        // Send 'SENDING' to Bob
        assert_ok!(MoveModule::transfer(
            signed_alice,
            bob_move.into_bytes(),
            SENDING
        ));
        // verify
        assert_eq!(INITIAL - SENDING, MoveModule::get_balance(BOB).unwrap());
    })
}
