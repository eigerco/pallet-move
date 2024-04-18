//! Benchmarking setup for pallet-move.

use frame_benchmarking::benchmarks;
#[cfg(test)]
use frame_benchmarking::impl_benchmark_test_suite;
use frame_system::{Config as SysConfig, RawOrigin};
use move_vm_backend::types::MAX_GAS_AMOUNT;
use sp_core::crypto::Ss58Codec;

use crate::{balance::BalanceOf, mock_utils::*, *};

const LIMIT: u128 = 60_000_000_000_000;

benchmarks! {
    where_clause { where
        T: Config + SysConfig,
        T::AccountId: Ss58Codec,
    }

    execute {
        let n in 0 .. 7;

        let (bob_32, bob_mv) = account_n_address::<T>(BOB_ADDR);
        let (alice_32, alice_mv) = account_n_address::<T>(ALICE_ADDR);
        let (dave_32, dave_mv) = account_n_address::<T>(DAVE_ADDR);
        let (eve_32, eve_mv) = account_n_address::<T>(EVE_ADDR);

        // Our benchmark plan (each is a test scenario with different parameters).
        let script_bcs = [
            car_wash_initial_coin_miniting().to_vec(),
            multiple_signers_init_module().to_vec(),
            car_wash_register_new_user().to_vec(),
            car_wash_buy_coin().to_vec(),
            car_wash_wash_car().to_vec(),
            multiple_signers_rent_apartment().to_vec(),
            multiple_signers_rent_apartment().to_vec(),
            multiple_signers_rent_apartment().to_vec(),
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
            car_wash_example_module().to_vec(),
            MAX_GAS_AMOUNT
        ).unwrap();
        Pallet::<T>::publish_module(
            RawOrigin::Signed(bob_32.clone()).into(),
            multiple_signers_module().to_vec(),
            MAX_GAS_AMOUNT
        ).unwrap();

        // Now prepare individual situations for proper script sequences.
        if n > 1 && n < 5 {
            Pallet::<T>::execute(
                RawOrigin::Signed(bob_32.clone()).into(),
                car_wash_initial_coin_miniting().to_vec(),
                MAX_GAS_AMOUNT,
                LIMIT.into()
            ).unwrap();
            if n > 2 {
                Pallet::<T>::execute(
                    RawOrigin::Signed(alice_32.clone()).into(),
                    car_wash_register_new_user().to_vec(),
                    MAX_GAS_AMOUNT,
                    LIMIT.into()
                ).unwrap();
            }
            if n > 3 {
                Pallet::<T>::execute(
                    RawOrigin::Signed(alice_32.clone()).into(),
                    car_wash_buy_coin().to_vec(),
                    MAX_GAS_AMOUNT,
                    LIMIT.into()
                ).unwrap();
            }
        }
        if n > 4 {
            Pallet::<T>::execute(
                RawOrigin::Signed(bob_32.clone()).into(),
                multiple_signers_init_module().to_vec(),
                MAX_GAS_AMOUNT,
                LIMIT.into()
            ).unwrap();
        }

    }: _(RawOrigin::Signed(accounts[n as usize].clone()), script_bcs[n as usize].clone(), gas[n as usize], BalanceOf::<T>::from(LIMIT))

    publish_module {
        let n in 0 .. 3;

        let (bob_32, bob_mv) = account_n_address::<T>(BOB_ADDR);

        let module_bcs = [
            move_basics_module().to_vec(),
            using_stdlib_natives_module().to_vec(),
            multiple_signers_module().to_vec(),
            car_wash_example_module().to_vec(),

        ];
        let gas = [11, 33, 67, 74];

    }: _(RawOrigin::Signed(bob_32), module_bcs[n as usize].clone(), gas[n as usize])

    publish_module_bundle {
        let bob_32 = account::<T>(BOB_ADDR);
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
                crate::mock_utils::account::<crate::mock::Test>(crate::mock_utils::BOB_ADDR),
                u128::MAX
            ),
            (
                crate::mock_utils::account::<crate::mock::Test>(crate::mock_utils::ALICE_ADDR),
                u128::MAX
            ),
            (
                crate::mock_utils::account::<crate::mock::Test>(crate::mock_utils::DAVE_ADDR),
                u128::MAX
            ),
            (
                crate::mock_utils::account::<crate::mock::Test>(crate::mock_utils::EVE_ADDR),
                u128::MAX
            ),
        ])
        .build(),
    crate::mock::Test
);

use benchmark_only::*;

mod benchmark_only {
    // Move Basics Example
    pub fn move_basics_module() -> &'static [u8] {
        core::include_bytes!(
            "assets/move-projects/move-basics/build/move-basics/bytecode_modules/EmptyBob.mv"
        )
    }

    // Using Stdlib Natives Example
    pub fn using_stdlib_natives_module() -> &'static [u8] {
        core::include_bytes!("assets/move-projects/using_stdlib_natives/build/using_stdlib_natives/bytecode_modules/Vector.mv")
    }

    // Car Wash Example
    pub fn car_wash_example_module() -> &'static [u8] {
        core::include_bytes!(
            "assets/move-projects/car-wash-example/build/car-wash-example/bytecode_modules/CarWash.mv"
        )
    }

    pub fn car_wash_initial_coin_miniting() -> &'static [u8] {
        core::include_bytes!("assets/move-projects/car-wash-example/build/car-wash-example/script_transactions/initial_coin_minting.mvt")
    }

    pub fn car_wash_register_new_user() -> &'static [u8] {
        core::include_bytes!("assets/move-projects/car-wash-example/build/car-wash-example/script_transactions/register_new_user.mvt")
    }

    pub fn car_wash_buy_coin() -> &'static [u8] {
        core::include_bytes!(
            "assets/move-projects/car-wash-example/build/car-wash-example/script_transactions/buy_coin.mvt"
        )
    }

    pub fn car_wash_wash_car() -> &'static [u8] {
        core::include_bytes!(
            "assets/move-projects/car-wash-example/build/car-wash-example/script_transactions/wash_car.mvt"
        )
    }

    // Multiple Signers Example
    pub fn multiple_signers_module() -> &'static [u8] {
        core::include_bytes!(
            "assets/move-projects/multiple-signers/build/multiple-signers/bytecode_modules/Dorm.mv"
        )
    }

    pub fn multiple_signers_init_module() -> &'static [u8] {
        core::include_bytes!("assets/move-projects/multiple-signers/build/multiple-signers/script_transactions/init_module.mvt")
    }

    pub fn multiple_signers_rent_apartment() -> &'static [u8] {
        core::include_bytes!("assets/move-projects/multiple-signers/build/multiple-signers/script_transactions/rent_apartment.mvt")
    }
}
