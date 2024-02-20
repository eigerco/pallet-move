//! Integration tests related to extrinsic call `update_stdlib`.

mod assets;
mod mock;

use frame_support::{assert_err, assert_ok, pallet_prelude::DispatchError};
use mock::*;
use move_stdlib::{move_stdlib_bundle, substrate_stdlib_bundle};
use move_vm_backend::types::MAX_GAS_AMOUNT;
use move_vm_backend_common::types::ScriptTransaction;

fn mock_move_stdlib() -> Vec<u8> {
    assets::read_bundle_from_project("testing-move-stdlib", "testing-move-stdlib")
}

fn mock_substrate_stdlib() -> Vec<u8> {
    assets::read_bundle_from_project("testing-substrate-stdlib", "testing-substrate-stdlib")
}

#[test]
fn regular_user_fail() {
    let move_stdlib = move_stdlib_bundle().to_vec();
    let sub_stdlib = substrate_stdlib_bundle().to_vec();

    ExtBuilder::default()
        .with_move_stdlib(Some(mock_move_stdlib()))
        .with_substrate_stdlib(Some(mock_substrate_stdlib()))
        .build()
        .execute_with(|| {
            assert_err!(
                MoveModule::update_stdlib(
                    RuntimeOrigin::signed(CAFE_ADDR_NATIVE.clone()),
                    move_stdlib,
                ),
                DispatchError::BadOrigin,
            );
            assert_err!(
                MoveModule::update_stdlib(
                    RuntimeOrigin::signed(CAFE_ADDR_NATIVE.clone()),
                    sub_stdlib,
                ),
                DispatchError::BadOrigin,
            );
        });
}

#[test]
fn update_stdlib() {
    let (bob_addr_32, bob_addr_mv) = addrs_from_ss58(BOB_ADDR).unwrap();
    let move_stdlib = move_stdlib_bundle().to_vec();
    let sub_stdlib = substrate_stdlib_bundle().to_vec();

    ExtBuilder::default()
        .with_move_stdlib(Some(mock_move_stdlib()))
        .with_substrate_stdlib(Some(mock_substrate_stdlib()))
        .build()
        .execute_with(|| {
            let car_wash_module = assets::read_module_from_project("car-wash-example", "CarWash");

            // Test that it is not working properly.
            assert!(MoveModule::publish_module(
                RuntimeOrigin::signed(bob_addr_32.clone()),
                car_wash_module.clone(),
                MAX_GAS_AMOUNT,
            )
            .is_err());

            // Update both standard-libraries.
            assert_ok!(MoveModule::update_stdlib(
                RuntimeOrigin::root(),
                move_stdlib,
            ));
            assert_ok!(MoveModule::update_stdlib(RuntimeOrigin::root(), sub_stdlib));

            // Test regular functionalities.
            assert_ok!(MoveModule::publish_module(
                RuntimeOrigin::signed(bob_addr_32.clone()),
                car_wash_module,
                MAX_GAS_AMOUNT,
            ));

            let script =
                assets::read_script_from_project("car-wash-example", "initial_coin_minting");
            let account = bcs::to_bytes(&bob_addr_mv).unwrap();
            let params: Vec<&[u8]> = vec![&account];
            let transaction = ScriptTransaction {
                bytecode: script,
                type_args: vec![],
                args: params.iter().map(|x| x.to_vec()).collect(),
            };
            let transaction_bc = bcs::to_bytes(&transaction).unwrap();
            assert_ok!(MoveModule::execute(
                RuntimeOrigin::signed(bob_addr_32.clone()),
                transaction_bc,
                MAX_GAS_AMOUNT,
                0,
            ));
        });
}
