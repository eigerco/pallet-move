mod mock;

use frame_support::assert_ok;
use mock::*;
use move_core_types::{identifier::Identifier, language_storage::StructTag};
use pallet_move::address;
use sp_runtime::AccountId32;

const EMPTY_ADDR: AccountId32 = AccountId32::new([1u8; 32]);

#[test]
/// Test getting a module.
fn get_module_correct() {
    new_test_ext().execute_with(|| {
        let signed = RuntimeOrigin::signed(EMPTY_ADDR);
        assert_ok!(Balances::force_set_balance(
            RuntimeOrigin::root(),
            EMPTY_ADDR,
            1_000_000_000_000
        ));

        let module = include_bytes!("assets/move/build/move/bytecode_modules/Empty.mv").to_vec();

        let res = MoveModule::publish_module(signed, module.clone(), 0);

        assert_ok!(res);

        let res = MoveModule::get_module(&EMPTY_ADDR, "Empty");

        assert_eq!(res, Ok(Some(module)));
    });
}

#[test]
/// Test getting a module that does not exist.
fn get_module_nonexistent() {
    new_test_ext().execute_with(|| {
        let res = MoveModule::get_module(&EMPTY_ADDR, "Empty");

        assert_eq!(res, Ok(None));
    });
}

#[test]
#[ignore = "failing - to be investigated"]
/// Test getting resource from the module.
fn get_resource_correct() {
    new_test_ext().execute_with(|| {
        let module = include_bytes!("assets/move/build/move/bytecode_modules/Empty.mv").to_vec();
        let resource_bytes = [0u8; 32]; // For now as we need to investigate what the resource looks like

        let res = MoveModule::publish_module(RuntimeOrigin::signed(EMPTY_ADDR), module.clone(), 0);

        assert_ok!(res);

        let address = address::to_move_address(&EMPTY_ADDR);

        let tag = StructTag {
            address,
            module: Identifier::new("Empty").unwrap(),
            name: Identifier::new("EmptyStruct").unwrap(),
            type_params: vec![],
        };

        let res = MoveModule::get_resource(&EMPTY_ADDR, &bcs::to_bytes(&tag).unwrap());

        assert_eq!(res, Ok(Some(resource_bytes.to_vec())));
    });
}

#[test]
/// Test getting resource from the module.
fn get_resource_non_existent() {
    new_test_ext().execute_with(|| {
        let module = include_bytes!("assets/move/build/move/bytecode_modules/Empty.mv").to_vec();

        let res = MoveModule::publish_module(RuntimeOrigin::signed(EMPTY_ADDR), module.clone(), 0);

        assert_ok!(res);

        let address = address::to_move_address(&EMPTY_ADDR);

        let tag = StructTag {
            address,
            module: Identifier::new("Empty").unwrap(),
            name: Identifier::new("NonExistentStruct").unwrap(),
            type_params: vec![],
        };

        let res = MoveModule::get_resource(&EMPTY_ADDR, &bcs::to_bytes(&tag).unwrap());

        assert_eq!(res, Ok(None));
    });
}
