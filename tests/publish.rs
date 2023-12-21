mod assets;
mod mock;

use frame_support::assert_ok;
use mock::*;

#[test]
/// Test that the module is published correctly.
fn publish_module_as_user_correct() {
    new_test_ext().execute_with(|| {
        let module = assets::read_module_from_project("move-basics", "Empty");

        let res = MoveModule::publish_module(
            RuntimeOrigin::signed(CAFE_ADDR_NATIVE.clone()),
            module,
            INFINITE_GAS,
        );
        assert_ok!(res);

        let module = assets::read_module_from_project("move-basics", "EmptyBob");

        let res = MoveModule::publish_module(
            RuntimeOrigin::signed(BOB_ADDR_NATIVE.clone()),
            module,
            INFINITE_GAS,
        );
        assert_ok!(res);
    });
}

#[test]
#[ignore = "to be implemented"]
/// Test that the module is not published if the user is not the owner.
fn publish_module_as_user_wrong_user() {
    new_test_ext().execute_with(|| {
        assert_eq!(1, 0);
    });
}

#[test]
#[ignore = "to be implemented"]
/// Test that the module is not published if the user does not have enough gas.
fn publish_module_as_user_insufficient_gas() {
    new_test_ext().execute_with(|| {
        assert_eq!(1, 0);
    });
}

#[test]
#[ignore = "to be implemented"]
/// Test that the module is not published if the bytecode is corrupted.
fn publish_module_as_user_corrupted_bytecode() {
    new_test_ext().execute_with(|| {
        assert_eq!(1, 0);
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
            INFINITE_GAS,
        );

        assert_ok!(res);
    });
}

#[test]
#[ignore = "to be implemented"]
/// Test that the bundle is not published if the user is not the owner.
fn publish_bundle_as_user_wrong_user() {
    new_test_ext().execute_with(|| {
        assert_eq!(1, 0);
    });
}

#[test]
#[ignore = "to be implemented"]
/// Test that the bundle is not published if the user does not have enough gas.
fn publish_bundle_as_user_insufficient_gas() {
    new_test_ext().execute_with(|| {
        assert_eq!(1, 0);
    });
}

#[test]
#[ignore = "to be implemented"]
/// Test that the bundle is not published if the bytecode is corrupted.
fn publish_bundle_as_user_corrupted_bytecode() {
    new_test_ext().execute_with(|| {
        assert_eq!(1, 0);
    });
}
