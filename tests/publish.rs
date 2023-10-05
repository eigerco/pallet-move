mod mock;

use frame_support::assert_ok;
use mock::*;

#[test]
/// Test that the module is published correctly.
fn publish_module_as_user_correct() {
    new_test_ext().execute_with(|| {
        let module = include_bytes!("assets/move/build/move/bytecode_modules/Empty.mv").to_vec();

        let res = MoveModule::publish_module(
            // Just for now - as Move module address account is 0xCAFE, we need to sing it the same
            // address. But in tests, AccountId is u64, so we need to convert it (0xCAFE -> 0xFECA000000000000 - endian welcome)
            RuntimeOrigin::signed(0xFECA000000000000),
            module,
            0,
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
#[ignore = "to be implemented"]
/// Test that the package is published correctly.
fn publish_package_as_user_correct() {
    new_test_ext().execute_with(|| {
        assert_eq!(1, 0);
    });
}

#[test]
#[ignore = "to be implemented"]
/// Test that the package is not published if the user is not the owner.
fn publish_package_as_user_wrong_user() {
    new_test_ext().execute_with(|| {
        assert_eq!(1, 0);
    });
}

#[test]
#[ignore = "to be implemented"]
/// Test that the package is not published if the user does not have enough gas.
fn publish_package_as_user_insufficient_gas() {
    new_test_ext().execute_with(|| {
        assert_eq!(1, 0);
    });
}

#[test]
#[ignore = "to be implemented"]
/// Test that the package is not published if the bytecode is corrupted.
fn publish_package_as_user_corrupted_bytecode() {
    new_test_ext().execute_with(|| {
        assert_eq!(1, 0);
    });
}
