//! Integration tests related to extrinsic call `update_stdlib`.

use crate::{assets, mock::*, no_type_args, script_transaction};

use frame_support::{assert_err, assert_ok, pallet_prelude::*};
use move_stdlib::move_stdlib_bundle;
use move_vm_backend::types::MAX_GAS_AMOUNT;

fn mock_move_stdlib() -> Vec<u8> {
    assets::read_bundle_from_project("testing-move-stdlib", "testing-move-stdlib")
}

fn mock_substrate_stdlib() -> Vec<u8> {
    assets::read_bundle_from_project("testing-substrate-stdlib", "testing-substrate-stdlib")
}

#[test]
fn regular_user_update_fail() {
    let bob_addr_native = addr32_from_ss58(BOB_ADDR).unwrap();

    ExtBuilder::default().build().execute_with(|| {
        assert_err!(
            MoveModule::update_stdlib_bundle(
                RuntimeOrigin::signed(bob_addr_native.clone()),
                mock_move_stdlib(),
            ),
            DispatchError::BadOrigin,
        );
        assert_err!(
            MoveModule::update_stdlib_bundle(
                RuntimeOrigin::signed(bob_addr_native),
                mock_substrate_stdlib(),
            ),
            DispatchError::BadOrigin,
        );
    });
}

#[test]
fn change_interface_add_param_fail() {
    ExtBuilder::default().build().execute_with(|| {
        // The default ExtBuilder will include stdlib.
        let res = MoveModule::update_stdlib_bundle(RuntimeOrigin::root(), mock_move_stdlib());
        assert!(verify_module_error_with_msg(res, "BackwardIncompatibleModuleUpdate").unwrap());
    });
}

#[test]
fn change_stdlib_api_remove_param_fail() {
    ExtBuilder::default()
        .with_move_stdlib(Some(mock_move_stdlib()))
        .with_substrate_stdlib(Some(mock_substrate_stdlib()))
        .build()
        .execute_with(|| {
            let res = MoveModule::update_stdlib_bundle(
                RuntimeOrigin::root(),
                move_stdlib_bundle().to_vec(),
            );
            assert!(verify_module_error_with_msg(res, "BackwardIncompatibleModuleUpdate").unwrap());
        });
}

#[test]
fn add_new_methods_or_update_methods_works() {
    let (bob_addr_native, bob_addr_move) = addrs_from_ss58(BOB_ADDR).unwrap();

    ExtBuilder::default().build().execute_with(|| {
        // Publish some module to fitting interface.
        let car_wash_module = assets::read_module_from_project("car-wash-example", "CarWash");
        assert_ok!(MoveModule::publish_module(
            RuntimeOrigin::signed(bob_addr_native.clone()),
            car_wash_module.clone(),
            MAX_GAS_AMOUNT,
        ));

        // Update substrate-library with extended and modified functionality.
        assert_ok!(MoveModule::update_stdlib_bundle(
            RuntimeOrigin::root(),
            mock_substrate_stdlib(),
        ));

        // Test module is still working in its bounds.
        let script = assets::read_script_from_project("car-wash-example", "initial_coin_minting");
        let transaction_bc = script_transaction!(script, no_type_args!(), &bob_addr_move);
        assert_ok!(MoveModule::execute(
            RuntimeOrigin::signed(bob_addr_native),
            transaction_bc,
            MAX_GAS_AMOUNT,
            0,
        ));
    });
}
