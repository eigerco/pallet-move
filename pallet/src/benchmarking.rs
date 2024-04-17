//! Benchmarking setup for pallet-move.

use frame_benchmarking::*;
use frame_system::{Config as SysConfig, RawOrigin};
use move_core_types::account_address::AccountAddress;
use move_vm_backend::types::MAX_GAS_AMOUNT;
pub use sp_core::{crypto::Ss58Codec, sr25519::Public};
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
        let n in 0 .. 7;

        let (bob_32, bob_mv) = account_address::<T>(BOB_ADDR);
        let (alice_32, alice_mv) = account_address::<T>(ALICE_ADDR);
        let (dave_32, dave_mv) = account_address::<T>(DAVE_ADDR);
        let (eve_32, eve_mv) = account_address::<T>(EVE_ADDR);

        // Our benchmark plan (each is a test scenario with different parameters).
        let script_bcs = [
            car_wash_initial_coin_miniting(&bob_mv),
            multiple_signers_init_module(&bob_mv),
            car_wash_register_new_user(&alice_mv),
            car_wash_buy_coin(&alice_mv, 1),
            car_wash_wash_car(&alice_mv),
            multiple_signers_rent_apartment(&alice_mv, &dave_mv, &eve_mv, 1),
            multiple_signers_rent_apartment(&alice_mv, &dave_mv, &eve_mv, 1),
            multiple_signers_rent_apartment(&alice_mv, &dave_mv, &eve_mv, 1),
        ];
        // Sequence of account-IDs who will execute each extrinsic call.
        let accounts = [
            bob_32.clone(),
            bob_32.clone(),
            alice_32.clone(),
            alice_32.clone(),
            alice_32.clone(),
            alice_32.clone(),
            dave_32,
            eve_32,
        ];
        // Needed gas amounts for each script, estimated by smove.
        let gas = [21, 21, 15, 31, 18, 66, 66, 66];

        // Now we have to prepare each script execution with a proper setup.
        // Publish both modules always.
        Pallet::<T>::publish_module(
            RawOrigin::Signed(bob_32.clone()).into(),
            car_wash_example_module(),
            MAX_GAS_AMOUNT
        ).unwrap();
        Pallet::<T>::publish_module(
            RawOrigin::Signed(bob_32.clone()).into(),
            multiple_signers_module(),
            MAX_GAS_AMOUNT
        ).unwrap();

        // Now prepare individual situations for proper script sequences.
        if n > 1 && n < 5 {
            Pallet::<T>::execute(
                RawOrigin::Signed(bob_32.clone()).into(),
                car_wash_initial_coin_miniting(&bob_mv),
                MAX_GAS_AMOUNT,
                LIMIT.into()
            ).unwrap();
            if n > 2 {
                Pallet::<T>::execute(
                    RawOrigin::Signed(alice_32.clone()).into(),
                    car_wash_register_new_user(&alice_mv),
                    MAX_GAS_AMOUNT,
                    LIMIT.into()
                ).unwrap();
            }
            if n > 3 {
                Pallet::<T>::execute(
                    RawOrigin::Signed(alice_32.clone()).into(),
                    car_wash_buy_coin(&alice_mv, 1),
                    MAX_GAS_AMOUNT,
                    LIMIT.into()
                ).unwrap();
            }
        }
        if n > 4 {
            Pallet::<T>::execute(
                RawOrigin::Signed(bob_32.clone()).into(),
                multiple_signers_init_module(&bob_mv),
                MAX_GAS_AMOUNT,
                LIMIT.into()
            ).unwrap();
        }

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
    crate::mock::ExtBuilder::default()
        .with_balances(vec![
            (
                crate::benchmarking::account::<crate::mock::Test>(crate::benchmarking::BOB_ADDR),
                u128::MAX
            ),
            (
                crate::benchmarking::account::<crate::mock::Test>(crate::benchmarking::ALICE_ADDR),
                u128::MAX
            ),
            (
                crate::benchmarking::account::<crate::mock::Test>(crate::benchmarking::DAVE_ADDR),
                u128::MAX
            ),
            (
                crate::benchmarking::account::<crate::mock::Test>(crate::benchmarking::EVE_ADDR),
                u128::MAX
            ),
        ])
        .build(),
    crate::mock::Test
);

#[cfg(test)]
fn account<T: SysConfig + Config>(name: &str) -> T::AccountId
where
    T::AccountId: From<Public>,
{
    Public::from_ss58check(name).unwrap().into()
}

fn account_address<T: SysConfig + Config>(name: &str) -> (T::AccountId, AccountAddress)
where
    T::AccountId: From<Public>,
{
    let account: T::AccountId = Public::from_ss58check(name).unwrap().into();
    let address = Pallet::<T>::to_move_address(&account).unwrap();
    (account, address)
}

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

fn car_wash_wash_car(addr: &AccountAddress) -> Vec<u8> {
    let script = core::include_bytes!(
        "assets/move-projects/car-wash-example/build/car-wash-example/bytecode_scripts/wash_car.mv"
    )
    .to_vec();
    script_transaction!(script, no_type_args!(), addr)
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
