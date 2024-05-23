//! Benchmarking setup for pallet-move.

use frame_benchmarking::v2::*;
use frame_system::{Config as SysConfig, RawOrigin};
use sp_core::crypto::Ss58Codec;

use crate::{mock_utils as utils, *};

type SourceOf<T> = <<T as SysConfig>::Lookup as sp_runtime::traits::StaticLookup>::Source;

const MAX_GAS_AMOUNT: u32 = u32::MAX;

macro_rules! impl_gas_costs_cal_fns {
    ($name:tt) => {
        pub fn $name() -> &'static [u8] {
            core::include_bytes!(concat!(
                "assets/move-projects/gas-costs/build/gas-costs/script_transactions/",
                stringify!($name),
                ".mvt"
            ))
        }
    };
}

#[benchmarks(
    where
        T: Config + SysConfig,
        T::AccountId: Ss58Codec,
        SourceOf<T>: From<T::AccountId>,
)]
mod benchmarks {
    use super::*;

    /// Because it is challenging to determine a reliable and fixed relation between gas costs and
    /// Substrate weights, we created Move scripts with known gas costs and increasing steps of 403.
    /// Twenty-five scripts with rising gas costs of about 403 for each iteration step were used as
    /// input for this benchmark.
    #[benchmark]
    fn execute(n: Linear<0, 24>) {
        let alice_32 = utils::account::<T>(utils::ALICE_ADDR);
        let bob_32 = utils::account::<T>(utils::BOB_ADDR);

        // Our benchmark plan (each is a test scenario with different parameters).
        let script_bcs = [
            mint_1().to_vec(),
            mint_2().to_vec(),
            mint_3().to_vec(),
            mint_4().to_vec(),
            mint_5().to_vec(),
            mint_6().to_vec(),
            mint_7().to_vec(),
            mint_8().to_vec(),
            mint_9().to_vec(),
            mint_10().to_vec(),
            mint_11().to_vec(),
            mint_12().to_vec(),
            mint_13().to_vec(),
            mint_14().to_vec(),
            mint_15().to_vec(),
            mint_16().to_vec(),
            mint_17().to_vec(),
            mint_18().to_vec(),
            mint_19().to_vec(),
            mint_20().to_vec(),
            mint_21().to_vec(),
            mint_22().to_vec(),
            mint_23().to_vec(),
            mint_24().to_vec(),
            mint_25().to_vec(),
        ];

        Pallet::<T>::publish_module(
            RawOrigin::Signed(bob_32.clone()).into(),
            publish_basic_coin().to_vec(),
            MAX_GAS_AMOUNT,
        )
        .unwrap();

        Pallet::<T>::execute(
            RawOrigin::Signed(alice_32.clone()).into(),
            publish_basic_balance().to_vec(),
            MAX_GAS_AMOUNT,
            0u128.into(),
        )
        .unwrap();

        #[extrinsic_call]
        execute(
            RawOrigin::Signed(bob_32),
            script_bcs[n as usize].clone(),
            19 + (n + 1) * 403,
            0u128.into(),
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

    // Basic Coin Example
    pub fn publish_basic_coin() -> &'static [u8] {
        core::include_bytes!(
            "assets/move-projects/gas-costs/build/gas-costs/bytecode_modules/dependencies/basic_coin/BasicCoin.mv"
        )
    }

    pub fn publish_basic_balance() -> &'static [u8] {
        core::include_bytes!(
            "assets/move-projects/gas-costs/build/gas-costs/script_transactions/publish_basic_balance.mvt"
        )
    }

    impl_gas_costs_cal_fns!(mint_1);
    impl_gas_costs_cal_fns!(mint_2);
    impl_gas_costs_cal_fns!(mint_3);
    impl_gas_costs_cal_fns!(mint_4);
    impl_gas_costs_cal_fns!(mint_5);
    impl_gas_costs_cal_fns!(mint_6);
    impl_gas_costs_cal_fns!(mint_7);
    impl_gas_costs_cal_fns!(mint_8);
    impl_gas_costs_cal_fns!(mint_9);
    impl_gas_costs_cal_fns!(mint_10);
    impl_gas_costs_cal_fns!(mint_11);
    impl_gas_costs_cal_fns!(mint_12);
    impl_gas_costs_cal_fns!(mint_13);
    impl_gas_costs_cal_fns!(mint_14);
    impl_gas_costs_cal_fns!(mint_15);
    impl_gas_costs_cal_fns!(mint_16);
    impl_gas_costs_cal_fns!(mint_17);
    impl_gas_costs_cal_fns!(mint_18);
    impl_gas_costs_cal_fns!(mint_19);
    impl_gas_costs_cal_fns!(mint_20);
    impl_gas_costs_cal_fns!(mint_21);
    impl_gas_costs_cal_fns!(mint_22);
    impl_gas_costs_cal_fns!(mint_23);
    impl_gas_costs_cal_fns!(mint_24);
    impl_gas_costs_cal_fns!(mint_25);
}
