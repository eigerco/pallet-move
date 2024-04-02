//! Benchmarking setup for pallet-move.

use frame_benchmarking::*;
use frame_support::traits::Currency;
use frame_system::{Config as SysConfig, RawOrigin};
use move_core_types::account_address::AccountAddress;
use move_vm_backend::types::MAX_GAS_AMOUNT;
pub use sp_core::{crypto::Ss58Codec, sr25519::Public};
use sp_runtime::traits::Zero;
use sp_std::{vec, vec::Vec};

#[cfg(test)]
use crate::mock::*;
use crate::{balance::BalanceOf, *};

const LIMIT: u128 = 30_000_000_000_000;
const BOB_ADDR: &str = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty";
const ALICE_ADDR: &str = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
const DAVE_ADDR: &str = "5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy";
const EVE_ADDR: &str = "5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw";

benchmarks! {
    where_clause { where
        T: Config + SysConfig,
        T::AccountId: From<Public>,
    }

    execute {
        let n in 0 .. 3;

        let bob_32: T::AccountId = Public::from_ss58check(BOB_ADDR).unwrap().into();
        let bob_mv = Pallet::<T>::to_move_address(&bob_32).unwrap();
        let _ = T::Currency::deposit_creating(&bob_32, BalanceOf::<T>::from(u128::MAX));
        let alice_32: T::AccountId = Public::from_ss58check(ALICE_ADDR).unwrap().into();
        let alice_mv = Pallet::<T>::to_move_address(&alice_32).unwrap();
        let _ = T::Currency::deposit_creating(&alice_32, BalanceOf::<T>::from(u128::MAX));
        let dave_32: T::AccountId = Public::from_ss58check(DAVE_ADDR).unwrap().into();
        let dave_mv = Pallet::<T>::to_move_address(&dave_32).unwrap();
        let _ = T::Currency::deposit_creating(&dave_32, BalanceOf::<T>::from(u128::MAX));
        let eve_32: T::AccountId = Public::from_ss58check(EVE_ADDR).unwrap().into();
        let eve_mv = Pallet::<T>::to_move_address(&eve_32).unwrap();
        let _ = T::Currency::deposit_creating(&eve_32, BalanceOf::<T>::from(u128::MAX));

        // Prepare car-wash-example move-project on mockup.
        Pallet::<T>::publish_module(
            RawOrigin::Signed(bob_32.clone()).into(),
            car_wash_example_module(),
            MAX_GAS_AMOUNT
        ).unwrap();
        // Prepare multiple-signers move-project on mockup.
        Pallet::<T>::publish_module(
            RawOrigin::Signed(bob_32.clone()).into(),
            multiple_signers_module(),
            MAX_GAS_AMOUNT
        ).unwrap();

        let accounts = [bob_32.clone(), bob_32, alice_32.clone(), alice_32];
        let script_bcs = [
            car_wash_initial_coin_miniting(&bob_mv),
            multiple_signers_init_module(&bob_mv),
            car_wash_register_new_user(&alice_mv),
            multiple_signers_rent_apartment(&alice_mv, &dave_mv, &eve_mv, 1),
        ];
        let gas = [21, 21, 15, 66];

    }: _(RawOrigin::Signed(accounts[n as usize].clone()), script_bcs[n as usize].clone(), gas[n as usize], BalanceOf::<T>::from(LIMIT))

    publish_module {
        let n in 0 .. 3;

        let bob_32: T::AccountId = Public::from_ss58check(BOB_ADDR).unwrap().into();
        let bob_mv = Pallet::<T>::to_move_address(&bob_32).unwrap();

        let module_bcs = [
            move_basics_module(),
            using_stdlib_natives_module(),
            multiple_signers_module(),
            car_wash_example_module(),

        ];
        let gas = [11, 33, 67, 74];

    }: _(RawOrigin::Signed(bob_32), module_bcs[n as usize].clone(), gas[n as usize])

    publish_module_bundle {
        let bob_32: T::AccountId = Public::from_ss58check(BOB_ADDR).unwrap().into();
        let bundle = core::include_bytes!("assets/move-projects/using_stdlib_natives/build/using_stdlib_natives/bundles/using_stdlib_natives.mvb").to_vec();
    }: _(RawOrigin::Signed(bob_32), bundle, 1_500_000)

    update_stdlib_bundle {
        let stdlib = core::include_bytes!("assets/move-projects/testing-substrate-stdlib/build/testing-substrate-stdlib/bundles/testing-substrate-stdlib.mvb").to_vec();
    }: _(RawOrigin::Root, stdlib)
}

#[cfg(test)]
impl_benchmark_test_suite!(
    Pallet,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Test
);

// Move Basics Example
fn move_basics_module() -> Vec<u8> {
    core::include_bytes!(
        "assets/move-projects/move-basics/build/move-basics/bytecode_modules/EmptyBob.mv"
    )
    .to_vec()
}

// Using Stdlib Natives Example
fn using_stdlib_natives_module() -> Vec<u8> {
    core::include_bytes!("assets/move-projects/using_stdlib_natives/build/using_stdlib_natives/bytecode_modules/Vector.mv").to_vec()
}

// Car Wash Example
fn car_wash_example_module() -> Vec<u8> {
    core::include_bytes!(
        "assets/move-projects/car-wash-example/build/car-wash-example/bytecode_modules/CarWash.mv"
    )
    .to_vec()
}

fn car_wash_initial_coin_miniting(addr: &AccountAddress) -> Vec<u8> {
    let script = core::include_bytes!("assets/move-projects/car-wash-example/build/car-wash-example/bytecode_scripts/initial_coin_minting.mv").to_vec();
    script_transaction!(script, no_type_args!(), addr)
}

fn car_wash_register_new_user(addr: &AccountAddress) -> Vec<u8> {
    let script = core::include_bytes!("assets/move-projects/car-wash-example/build/car-wash-example/bytecode_scripts/register_new_user.mv").to_vec();
    script_transaction!(script, no_type_args!(), addr)
}

fn car_wash_buy_coin(addr: &AccountAddress, cnt: u8) -> Vec<u8> {
    let script = core::include_bytes!(
        "assets/move-projects/car-wash-example/build/car-wash-example/bytecode_scripts/buy_coin.mv"
    )
    .to_vec();
    script_transaction!(script, no_type_args!(), addr, &cnt)
}

// Multiple Signers Example
fn multiple_signers_module() -> Vec<u8> {
    core::include_bytes!(
        "assets/move-projects/multiple-signers/build/multiple-signers/bytecode_modules/Dorm.mv"
    )
    .to_vec()
}

fn multiple_signers_init_module(addr: &AccountAddress) -> Vec<u8> {
    let script = core::include_bytes!("assets/move-projects/multiple-signers/build/multiple-signers/bytecode_scripts/init_module.mv").to_vec();
    script_transaction!(script, no_type_args!(), addr)
}

fn multiple_signers_rent_apartment(
    addr1: &AccountAddress,
    addr2: &AccountAddress,
    addr3: &AccountAddress,
    cnt: u8,
) -> Vec<u8> {
    let script = core::include_bytes!("assets/move-projects/multiple-signers/build/multiple-signers/bytecode_scripts/rent_apartment.mv").to_vec();
    script_transaction!(script, no_type_args!(), addr1, addr2, addr3, &cnt)
}
