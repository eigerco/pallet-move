//! Integration tests related to extrinsic call `update_stdlib`.

mod assets;
mod mock;

use frame_support::{
    assert_err, assert_ok, dispatch::DispatchErrorWithPostInfo, pallet_prelude::*,
};
use mock::*;
use move_stdlib::move_stdlib_bundle;
use move_vm_backend::types::MAX_GAS_AMOUNT;
use move_vm_backend_common::types::ScriptTransaction;

fn mock_move_stdlib() -> Vec<u8> {
    assets::read_bundle_from_project("testing-move-stdlib", "testing-move-stdlib")
}

fn mock_substrate_stdlib() -> Vec<u8> {
    assets::read_bundle_from_project("testing-substrate-stdlib", "testing-substrate-stdlib")
}

fn verify_module_error_with_msg(
    res: DispatchResultWithPostInfo,
    e_msg: &str,
) -> Result<bool, String> {
    if let Err(DispatchErrorWithPostInfo {
        error: DispatchError::Module(moderr),
        ..
    }) = res
    {
        if let Some(msg) = moderr.message {
            return Ok(msg == e_msg);
        }
    }
    Err(format!("{res:?} does not match '{e_msg}'"))
}

macro_rules! script_transaction {
    ($bytecode:expr, $type_args:expr, $($args:expr),*) => {
        {
            let transaction = ScriptTransaction {
                bytecode: $bytecode,
                type_args: $type_args,
                args: vec![$(bcs::to_bytes($args).unwrap()),*],
            };
            bcs::to_bytes(&transaction).unwrap()
        }
    }
}

macro_rules! no_type_args {
    () => {
        vec![]
    };
}

#[test]
fn regular_user_update_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_err!(
            MoveModule::update_stdlib_bundle(
                RuntimeOrigin::signed(CAFE_ADDR_NATIVE.clone()),
                mock_move_stdlib(),
            ),
            DispatchError::BadOrigin,
        );
        assert_err!(
            MoveModule::update_stdlib_bundle(
                RuntimeOrigin::signed(CAFE_ADDR_NATIVE.clone()),
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
    let (bob_addr_32, bob_addr_mv) = addrs_from_ss58(BOB_ADDR).unwrap();

    ExtBuilder::default().build().execute_with(|| {
        // Publish some module to fitting interface.
        let car_wash_module = assets::read_module_from_project("car-wash-example", "CarWash");
        assert_ok!(MoveModule::publish_module(
            RuntimeOrigin::signed(bob_addr_32.clone()),
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
        let transaction_bc = script_transaction!(script, no_type_args!(), &bob_addr_mv);
        assert_ok!(MoveModule::execute(
            RuntimeOrigin::signed(bob_addr_32.clone()),
            transaction_bc,
            MAX_GAS_AMOUNT,
            0,
        ));
    });
}
