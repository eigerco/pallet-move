mod mock;

use frame_support::assert_ok;
use mock::*;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, StructTag},
};
use pallet_move::address;

// Just for now - as Move module address account is 0xCAFE, we need to sign it the same
// address. But in tests, AccountId is u64, so we need to convert it (0xCAFE -> 0xFECA000000000000 - endian welcome)
const ACC_ADDR: u64 = 0xFECA000000000000;

#[test]
/// Test getting a module.
fn get_module_correct() {
    new_test_ext().execute_with(|| {
        let module = include_bytes!("assets/move/build/move/bytecode_modules/Empty.mv").to_vec();

        let res = MoveModule::publish_module(RuntimeOrigin::signed(ACC_ADDR), module.clone(), 0);

        assert_ok!(res);

        let address = AccountAddress::from_hex_literal("0xCAFE").unwrap(); // Alternative: let address = address::to_move_address(&ACC_ADDR);
        let module_id = ModuleId::new(address, Identifier::new("Empty").unwrap());

        let res = MoveModule::get_module(&bcs::to_bytes(&module_id).unwrap());

        assert_eq!(res, Ok(Some(module)));
    });
}

#[test]
/// Test getting a module that does not exist.
fn get_module_nonexistent() {
    new_test_ext().execute_with(|| {
        let address = AccountAddress::from_hex_literal("0xCAFE").unwrap();
        let module_id = ModuleId::new(address, Identifier::new("Empty").unwrap());

        let res = MoveModule::get_module(&bcs::to_bytes(&module_id).unwrap());

        assert_eq!(res, Ok(None));
    });
}

#[test]
/// Test getting a module providing incorrect (no module name after the address) module id.
fn get_module_error() {
    new_test_ext().execute_with(|| {
        let address = [0; 32];
        let errmsg = "error in get_module: unexpected end of input".as_bytes();

        let res = MoveModule::get_module(&address);

        assert_eq!(res, Err(errmsg.to_vec()));
    });
}

#[test]
#[ignore = "failing - to be investigated"]
/// Test getting resource from the module.
fn get_resource_correct() {
    new_test_ext().execute_with(|| {
        let module = include_bytes!("assets/move/build/move/bytecode_modules/Empty.mv").to_vec();

        let res = MoveModule::publish_module(RuntimeOrigin::signed(ACC_ADDR), module.clone(), 0);

        assert_ok!(res);

        let address = address::to_move_address(&ACC_ADDR);

        let tag = StructTag {
            address,
            module: Identifier::new("Empty").unwrap(),
            name: Identifier::new("EmptyStruct").unwrap(),
            type_params: vec![],
        };

        let res = MoveModule::get_resource(&ACC_ADDR, &bcs::to_bytes(&tag).unwrap());

        assert_eq!(res, Ok(Some(module)));
    });
}

#[test]
/// Test getting resource from the module.
fn get_resource_non_existent() {
    new_test_ext().execute_with(|| {
        let module = include_bytes!("assets/move/build/move/bytecode_modules/Empty.mv").to_vec();

        let res = MoveModule::publish_module(RuntimeOrigin::signed(ACC_ADDR), module.clone(), 0);

        assert_ok!(res);

        let address = address::to_move_address(&ACC_ADDR);

        let tag = StructTag {
            address,
            module: Identifier::new("Empty").unwrap(),
            name: Identifier::new("NonExistentStruct").unwrap(),
            type_params: vec![],
        };

        let res = MoveModule::get_resource(&ACC_ADDR, &bcs::to_bytes(&tag).unwrap());

        assert_eq!(res, Ok(None));
    });
}
