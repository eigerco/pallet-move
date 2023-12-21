mod assets;
mod mock;

use frame_support::assert_ok;
use mock::*;
use move_core_types::{identifier::Identifier, language_storage::StructTag};

#[test]
/// Test getting a module.
fn get_module_correct() {
    new_test_ext().execute_with(|| {
        let module_name = "Empty";
        let module = assets::read_module_from_project("move-basics", module_name);
        let addr_native = CAFE_ADDR_NATIVE.clone();

        let res = MoveModule::publish_module(
            RuntimeOrigin::signed(addr_native.clone()),
            module.clone(),
            INFINITE_GAS,
        );

        assert_ok!(res);

        let res = MoveModule::get_module(&addr_native, module_name);

        assert_eq!(res, Ok(Some(module)));
    });
}

#[test]
/// Test getting a module that does not exist.
fn get_module_nonexistent() {
    new_test_ext().execute_with(|| {
        let res = MoveModule::get_module(&CAFE_ADDR_NATIVE, "Empty");

        assert_eq!(res, Ok(None));
    });
}

#[test]
#[ignore = "failing - to be investigated"]
/// Test getting resource from the module.
fn get_resource_correct() {
    new_test_ext().execute_with(|| {
        let addr = *CAFE_ADDR_MOVE;
        let addr_native = MoveModule::to_native_account(&addr).unwrap();

        let module = assets::read_module_from_project("move-basics", "Empty");
        let resource_bytes = [0u8; 32]; // For now as we need to investigate what the resource looks like

        let res = MoveModule::publish_module(
            RuntimeOrigin::signed(addr_native.clone()),
            module.clone(),
            0,
        );

        assert_ok!(res);

        let tag = StructTag {
            address: addr,
            module: Identifier::new("Empty").unwrap(),
            name: Identifier::new("EmptyStruct").unwrap(),
            type_params: vec![],
        };

        let res = MoveModule::get_resource(&addr_native, &bcs::to_bytes(&tag).unwrap());

        assert_eq!(res, Ok(Some(resource_bytes.to_vec())));
    });
}

#[test]
/// Test getting resource from the module.
fn get_resource_non_existent() {
    new_test_ext().execute_with(|| {
        let addr = *CAFE_ADDR_MOVE;
        let addr_native = MoveModule::to_native_account(&addr).unwrap();

        let module = assets::read_module_from_project("move-basics", "Empty");

        let res = MoveModule::publish_module(
            RuntimeOrigin::signed(addr_native.clone()),
            module,
            INFINITE_GAS,
        );

        assert_ok!(res);

        let tag = StructTag {
            address: addr,
            module: Identifier::new("Empty").unwrap(),
            name: Identifier::new("NonExistentStruct").unwrap(),
            type_params: vec![],
        };

        let res = MoveModule::get_resource(&addr_native, &bcs::to_bytes(&tag).unwrap());

        assert_eq!(res, Ok(None));
    });
}
