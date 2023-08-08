mod mock;
use mock::*;

#[test]
/// Test that the module is published correctly.
fn publish_module_as_user_correct() {
    new_test_ext().execute_with(|| {
        assert_eq!(1, 0);
    });
}

#[test]
/// Test that the module is not published if the user is not the owner.
fn publish_module_as_user_wrong_user() {
    new_test_ext().execute_with(|| {
        assert_eq!(1, 0);
    });
}

#[test]
/// Test that the module is not published if the user does not have enough gas.
fn publish_module_as_user_insufficient_gas() {
    new_test_ext().execute_with(|| {
        assert_eq!(1, 0);
    });
}

#[test]
/// Test that the module is not published if the bytecode is corrupted.
fn publish_module_as_user_corrupted_bytecode() {
    new_test_ext().execute_with(|| {
        assert_eq!(1, 0);
    });
}

#[test]
/// Test that the package is published correctly.
fn publish_package_as_user_correct() {
    new_test_ext().execute_with(|| {
        assert_eq!(1, 0);
    });
}

#[test]
/// Test that the package is not published if the user is not the owner.
fn publish_package_as_user_wrong_user() {
    new_test_ext().execute_with(|| {
        assert_eq!(1, 0);
    });
}

#[test]
/// Test that the package is not published if the user does not have enough gas.
fn publish_package_as_user_insufficient_gas() {
    new_test_ext().execute_with(|| {
        assert_eq!(1, 0);
    });
}

#[test]
/// Test that the package is not published if the bytecode is corrupted.
fn publish_package_as_user_corrupted_bytecode() {
    new_test_ext().execute_with(|| {
        assert_eq!(1, 0);
    });
}