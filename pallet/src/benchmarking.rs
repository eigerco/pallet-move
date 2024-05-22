//! Benchmarking setup for pallet-move.

use frame_benchmarking::v2::*;
use frame_system::{Config as SysConfig, RawOrigin};
use pallet_balances::{Config as BalancesConfig, Pallet as Balances};
use sp_core::crypto::Ss58Codec;

use crate::{mock_utils as utils, *};

const LIMIT: u128 = 60_000_000_000_000;

const MAX_GAS_AMOUNT: u32 = u32::MAX;

type SourceOf<T> = <<T as SysConfig>::Lookup as sp_runtime::traits::StaticLookup>::Source;

#[benchmarks(
    where
        T: Config + SysConfig + BalancesConfig,
        T::AccountId: Ss58Codec,
        T::Balance: From<u128>,
        SourceOf<T>: From<T::AccountId>,
)]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn execute(n: Linear<0, 7>) {
        let bob_32 = utils::account::<T>(utils::BOB_ADDR);
        let alice_32 = utils::account::<T>(utils::ALICE_ADDR);
        let dave_32 = utils::account::<T>(utils::DAVE_ADDR);
        let eve_32 = utils::account::<T>(utils::EVE_ADDR);

        // Our benchmark plan (each is a test scenario with different parameters).
        let script_bcs = [
            car_wash_initial_coin_miniting().to_vec(),
            car_wash_register_new_user().to_vec(),
            car_wash_buy_coin().to_vec(),
            car_wash_wash_car().to_vec(),
            gas_costs_short_cheap_script().to_vec(),
            gas_costs_long_cheap_script().to_vec(),
            multiple_signers_init_module().to_vec(),
            multiple_signers_rent_apartment().to_vec(),
        ];
        // Sequence of account-IDs who will execute each extrinsic call.
        let accounts = [
            bob_32.clone(), // car-wash-example
            alice_32.clone(),
            alice_32.clone(),
            alice_32.clone(),
            alice_32.clone(), // gas-costs
            alice_32.clone(),
            bob_32.clone(), // multiple-signers
            eve_32,
        ];
        // Needed gas amounts for each script, estimated by smove.
        let gas = [343, 197, 795, 425, 1, 8, 308, 1377];
        // Balance limit to be used.
        let regular = [LIMIT; 2];
        let max = [u128::MAX; 2];
        let limit = [regular, regular, max, regular].concat();

        // Now we have to prepare each script execution with a proper setup.
        // Publish both modules always.
        Pallet::<T>::publish_module(
            RawOrigin::Signed(bob_32.clone()).into(),
            car_wash_example_module().to_vec(),
            MAX_GAS_AMOUNT,
        )
        .unwrap();
        Pallet::<T>::publish_module(
            RawOrigin::Signed(bob_32.clone()).into(),
            multiple_signers_module().to_vec(),
            MAX_GAS_AMOUNT,
        )
        .unwrap();

        // Now prepare individual situations for proper script sequences.
        if n > 0 && n < 6 {
            Pallet::<T>::execute(
                RawOrigin::Signed(bob_32.clone()).into(),
                car_wash_initial_coin_miniting().to_vec(),
                MAX_GAS_AMOUNT,
                LIMIT.into(),
            )
            .unwrap();
            if n > 1 {
                Pallet::<T>::execute(
                    RawOrigin::Signed(alice_32.clone()).into(),
                    car_wash_register_new_user().to_vec(),
                    MAX_GAS_AMOUNT,
                    LIMIT.into(),
                )
                .unwrap();
            }
            if n > 2 && n < 4 {
                Pallet::<T>::execute(
                    RawOrigin::Signed(alice_32.clone()).into(),
                    car_wash_buy_coin().to_vec(),
                    MAX_GAS_AMOUNT,
                    LIMIT.into(),
                )
                .unwrap();
            }
            if n > 3 {
                Balances::<T>::force_set_balance(
                    RawOrigin::Root.into(),
                    alice_32.clone().into(),
                    u128::MAX.into(),
                )
                .unwrap();
            }
        }
        if n > 6 {
            Pallet::<T>::execute(
                RawOrigin::Signed(bob_32.clone()).into(),
                multiple_signers_init_module().to_vec(),
                MAX_GAS_AMOUNT,
                LIMIT.into(),
            )
            .unwrap();
            Pallet::<T>::execute(
                RawOrigin::Signed(alice_32.clone()).into(),
                multiple_signers_rent_apartment().to_vec(),
                MAX_GAS_AMOUNT,
                LIMIT.into(),
            )
            .unwrap();
            Pallet::<T>::execute(
                RawOrigin::Signed(dave_32).into(),
                multiple_signers_rent_apartment().to_vec(),
                MAX_GAS_AMOUNT,
                LIMIT.into(),
            )
            .unwrap();
        }

        #[extrinsic_call]
        execute(
            RawOrigin::Signed(accounts[n as usize].clone()),
            script_bcs[n as usize].clone(),
            gas[n as usize],
            limit[n as usize].into(),
        )
    }

    #[benchmark]
    fn publish_module(n: Linear<0, 2>) {
        let bob_32 = utils::account::<T>(utils::BOB_ADDR);

        let module_bcs = [
            multiple_signers_module().to_vec(),
            car_wash_example_module().to_vec(),
            base58_smove_build_module().to_vec(),
        ];
        let gas = [661, 732, 100];

        #[extrinsic_call]
        publish_module(
            RawOrigin::Signed(bob_32),
            module_bcs[n as usize].clone(),
            gas[n as usize],
        );
    }

    #[benchmark]
    fn publish_module_bundle(n: Linear<0, 2>) {
        let bob_32 = utils::account::<T>(utils::BOB_ADDR);

        let bundles = [
            multiple_signers_module_as_bundle().to_vec(),
            car_wash_example_module_as_bundle().to_vec(),
            base58_smove_build_module_as_bundle().to_vec(),
        ];
        let gas = [664, 735, 102];

        #[extrinsic_call]
        publish_module_bundle(
            RawOrigin::Signed(bob_32),
            bundles[n as usize].clone(),
            gas[n as usize],
        );
    }

    #[benchmark]
    fn update_stdlib_bundle() {
        let stdlib = core::include_bytes!("assets/move-projects/testing-substrate-stdlib/build/testing-substrate-stdlib/bundles/testing-substrate-stdlib.mvb").to_vec();

        #[extrinsic_call]
        update_stdlib_bundle(RawOrigin::Root, stdlib);
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
}

use benchmark_only::*;

mod benchmark_only {
    // Base58 build example
    pub fn base58_smove_build_module() -> &'static [u8] {
        core::include_bytes!(
            "assets/move-projects/base58_smove_build/build/base58_smove_build/bytecode_modules/BobBase58.mv"
        )
    }
    pub fn base58_smove_build_module_as_bundle() -> &'static [u8] {
        core::include_bytes!(
            "assets/move-projects/base58_smove_build/build/base58_smove_build/bundles/base58_smove_build.mvb"
        )
    }

    // Car Wash Example
    pub fn car_wash_example_module() -> &'static [u8] {
        core::include_bytes!(
            "assets/move-projects/car-wash-example/build/car-wash-example/bytecode_modules/CarWash.mv"
        )
    }
    pub fn car_wash_example_module_as_bundle() -> &'static [u8] {
        core::include_bytes!(
            "assets/move-projects/car-wash-example/build/car-wash-example/bundles/car-wash-example.mvb"
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
    pub fn multiple_signers_module_as_bundle() -> &'static [u8] {
        core::include_bytes!(
            "assets/move-projects/multiple-signers/build/multiple-signers/bundles/multiple-signers.mvb"
        )
    }

    pub fn multiple_signers_init_module() -> &'static [u8] {
        core::include_bytes!("assets/move-projects/multiple-signers/build/multiple-signers/script_transactions/init_module.mvt")
    }

    pub fn multiple_signers_rent_apartment() -> &'static [u8] {
        core::include_bytes!("assets/move-projects/multiple-signers/build/multiple-signers/script_transactions/rent_apartment.mvt")
    }

    pub fn gas_costs_short_cheap_script() -> &'static [u8] {
        core::include_bytes!("assets/move-projects/gas-costs/build/gas-costs/script_transactions/short_cheap_script.mvt")
    }

    pub fn gas_costs_long_cheap_script() -> &'static [u8] {
        core::include_bytes!("assets/move-projects/gas-costs/build/gas-costs/script_transactions/long_cheap_script.mvt")
    }
}
