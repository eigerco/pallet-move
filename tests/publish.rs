mod assets;
mod mock;

use frame_support::assert_ok;
use mock::*;
use move_vm_backend::types::MAX_GAS_AMOUNT;
use pallet_move::GasStrategy;

#[test]
/// Test that the module is published correctly.
fn publish_module_as_user_correct() {
    new_test_ext().execute_with(|| {
        let module = assets::read_module_from_project("move-basics", "Empty");

        let res = MoveModule::publish_module(
            RuntimeOrigin::signed(CAFE_ADDR_NATIVE.clone()),
            module,
            MAX_GAS_AMOUNT,
        );
        assert_ok!(res);

        let module = assets::read_module_from_project("move-basics", "EmptyBob");

        let res = MoveModule::publish_module(
            RuntimeOrigin::signed(BOB_ADDR_NATIVE.clone()),
            module,
            MAX_GAS_AMOUNT,
        );
        assert_ok!(res);
    });
}

#[test]
/// Test that the module is not published if the user is not the owner.
fn publish_module_as_user_wrong_user() {
    new_test_ext().execute_with(|| {
        let module = assets::read_module_from_project("move-basics", "Empty");

        let res = MoveModule::publish_module(
            RuntimeOrigin::signed(BOB_ADDR_NATIVE.clone()),
            module,
            MAX_GAS_AMOUNT,
        );
        assert!(res.is_err());
    });
}

#[test]
/// Test that the module is not published if the user does not have enough gas.
fn publish_module_as_user_insufficient_gas() {
    new_test_ext().execute_with(|| {
        let module = assets::read_module_from_project("move-basics", "Empty");

        let gas_limit = 1;
        let res = MoveModule::publish_module(
            RuntimeOrigin::signed(CAFE_ADDR_NATIVE.clone()),
            module,
            gas_limit,
        );
        assert!(res.is_err());
    });
}

#[test]
/// Test that the module is not published if the bytecode is corrupted.
fn publish_module_as_user_corrupted_bytecode() {
    new_test_ext().execute_with(|| {
        let mut module = assets::read_module_from_project("move-basics", "Empty");

        // This should be enough to corrupt the bytecode.
        for i in 0..(module.len() / 4) {
            module[i] += 1;
        }

        let res = MoveModule::publish_module(
            RuntimeOrigin::signed(CAFE_ADDR_NATIVE.clone()),
            module,
            MAX_GAS_AMOUNT,
        );
        assert!(res.is_err());
    });
}

#[test]
/// Test that the bundle is published correctly.
fn publish_bundle_as_user_correct() {
    new_test_ext().execute_with(|| {
        let bundle =
            assets::read_bundle_from_project("using_stdlib_natives", "using_stdlib_natives");

        let res = MoveModule::publish_module_bundle(
            RuntimeOrigin::signed(BOB_ADDR_NATIVE.clone()),
            bundle,
            MAX_GAS_AMOUNT,
        );

        assert_ok!(res);
    });
}

#[test]
/// Test that the bundle is not published if the user is not the owner.
fn publish_bundle_as_user_wrong_user() {
    new_test_ext().execute_with(|| {
        let bundle =
            assets::read_bundle_from_project("using_stdlib_natives", "using_stdlib_natives");

        let res = MoveModule::publish_module_bundle(
            RuntimeOrigin::signed(CAFE_ADDR_NATIVE.clone()),
            bundle,
            MAX_GAS_AMOUNT,
        );

        assert!(res.is_err());
    });
}

#[test]
/// Test that the bundle is not published if the user does not have enough gas.
fn publish_bundle_as_user_insufficient_gas() {
    new_test_ext().execute_with(|| {
        let bundle =
            assets::read_bundle_from_project("using_stdlib_natives", "using_stdlib_natives");

        let gas_limit = 1;
        let res = MoveModule::publish_module_bundle(
            RuntimeOrigin::signed(BOB_ADDR_NATIVE.clone()),
            bundle,
            gas_limit,
        );

        assert!(res.is_err());
    });
}

#[test]
/// Test that the bundle is not published if the bytecode is corrupted.
fn publish_bundle_as_user_corrupted_bytecode() {
    new_test_ext().execute_with(|| {
        let mut bundle =
            assets::read_bundle_from_project("using_stdlib_natives", "using_stdlib_natives");

        // This should be enough to corrupt the bytecode.
        for i in 0..(bundle.len() / 4) {
            bundle[i] += 1;
        }

        let res = MoveModule::publish_module_bundle(
            RuntimeOrigin::signed(CAFE_ADDR_NATIVE.clone()),
            bundle,
            MAX_GAS_AMOUNT,
        );
        assert!(res.is_err());
    });
}

#[test]
/// Test that the module is published correctly when the gas is estimated.
fn raw_publish_module_dry_run() {
    new_test_ext().execute_with(|| {
        let module = assets::read_module_from_project("using_stdlib_natives", "Vector");

        let estimation =
            MoveModule::raw_publish_module(&BOB_ADDR_MOVE, module.clone(), GasStrategy::DryRun)
                .expect("failed to publish a module")
                .gas_used;

        let insufficient_gas = estimation - 1;
        let invalid_publish = MoveModule::publish_module(
            RuntimeOrigin::signed(BOB_ADDR_NATIVE.clone()),
            module.clone(),
            insufficient_gas,
        );
        assert!(
            invalid_publish.is_err(),
            "managed to publish a module with insufficient gas"
        );

        // Use the exact amount of gas.
        MoveModule::publish_module(
            RuntimeOrigin::signed(BOB_ADDR_NATIVE.clone()),
            module,
            estimation,
        )
        .expect("failed to publish a module");
    });
}

#[test]
/// Test that the bundle is published correctly when the gas is estimated.
fn raw_publish_bundle_dry_run() {
    new_test_ext().execute_with(|| {
        let bundle =
            assets::read_bundle_from_project("using_stdlib_natives", "using_stdlib_natives");

        let estimation =
            MoveModule::raw_publish_bundle(&BOB_ADDR_MOVE, bundle.clone(), GasStrategy::DryRun)
                .expect("failed to publish a bundle")
                .gas_used;

        let insufficient_gas = estimation - 1;
        let invalid_publish = MoveModule::publish_module(
            RuntimeOrigin::signed(BOB_ADDR_NATIVE.clone()),
            bundle.clone(),
            insufficient_gas,
        );
        assert!(
            invalid_publish.is_err(),
            "managed to publish a bundle with insufficient gas"
        );

        // Use the exact amount of gas.
        MoveModule::publish_module_bundle(
            RuntimeOrigin::signed(BOB_ADDR_NATIVE.clone()),
            bundle,
            estimation,
        )
        .expect("failed to publish a bundle");
    });
}

#[test]
/// Test that the module publishing fails when gas is exceeded.
fn publish_module_will_fail_in_case_the_gas_limit_is_exceeded() {
    new_test_ext().execute_with(|| {
        let module = assets::read_module_from_project("using_stdlib_natives", "Vector");

        // Exceed the maximum gas limit by one.
        let result = MoveModule::publish_module(
            RuntimeOrigin::signed(BOB_ADDR_NATIVE.clone()),
            module,
            MAX_GAS_AMOUNT + 1,
        );
        assert!(
            result.is_err(),
            "managed to publish a module with insufficient gas"
        );
    });
}

#[test]
/// Test that the bundle publishing fails when gas is exceeded.
fn publish_bundle_will_fail_in_case_the_gas_limit_is_exceeded() {
    new_test_ext().execute_with(|| {
        let bundle =
            assets::read_bundle_from_project("using_stdlib_natives", "using_stdlib_natives");

        // Exceed the maximum gas limit by one.
        let result = MoveModule::publish_module_bundle(
            RuntimeOrigin::signed(BOB_ADDR_NATIVE.clone()),
            bundle,
            MAX_GAS_AMOUNT + 1,
        );
        assert!(
            result.is_err(),
            "managed to publish a bundle with insufficient gas"
        );
    });
}
