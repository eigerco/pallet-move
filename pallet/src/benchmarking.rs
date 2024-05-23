//! Benchmarking setup for pallet-move.

use frame_benchmarking::v2::*;
use frame_system::{Config as SysConfig, RawOrigin};
use sp_core::crypto::Ss58Codec;

use crate::{mock_utils as utils, *};

type SourceOf<T> = <<T as SysConfig>::Lookup as sp_runtime::traits::StaticLookup>::Source;

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
    /// Substrate weights, we created Move scripts with known gas costs and increasing steps of 20.
    /// Twenty-five scripts with rising gas costs of about 20 for each iteration step were used as
    /// input for this benchmark. Therefore, the original output was divided by 20 afterwards.
    #[benchmark]
    fn execute(n: Linear<0, 24>) {
        let alice_32 = utils::account::<T>(utils::ALICE_ADDR);

        // Our benchmark plan (each is a test scenario with different parameters).
        let script_bcs = [
            cal_gas_cost_0().to_vec(),
            cal_gas_cost_1().to_vec(),
            cal_gas_cost_2().to_vec(),
            cal_gas_cost_3().to_vec(),
            cal_gas_cost_4().to_vec(),
            cal_gas_cost_5().to_vec(),
            cal_gas_cost_6().to_vec(),
            cal_gas_cost_7().to_vec(),
            cal_gas_cost_8().to_vec(),
            cal_gas_cost_9().to_vec(),
            cal_gas_cost_10().to_vec(),
            cal_gas_cost_11().to_vec(),
            cal_gas_cost_12().to_vec(),
            cal_gas_cost_13().to_vec(),
            cal_gas_cost_14().to_vec(),
            cal_gas_cost_15().to_vec(),
            cal_gas_cost_16().to_vec(),
            cal_gas_cost_17().to_vec(),
            cal_gas_cost_18().to_vec(),
            cal_gas_cost_19().to_vec(),
            cal_gas_cost_20().to_vec(),
            cal_gas_cost_21().to_vec(),
            cal_gas_cost_22().to_vec(),
            cal_gas_cost_23().to_vec(),
            cal_gas_cost_24().to_vec(),
        ];

        #[extrinsic_call]
        execute(
            RawOrigin::Signed(alice_32),
            script_bcs[n as usize].clone(),
            (n + 1) * 20,
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

    impl_gas_costs_cal_fns!(cal_gas_cost_0);
    impl_gas_costs_cal_fns!(cal_gas_cost_1);
    impl_gas_costs_cal_fns!(cal_gas_cost_2);
    impl_gas_costs_cal_fns!(cal_gas_cost_3);
    impl_gas_costs_cal_fns!(cal_gas_cost_4);
    impl_gas_costs_cal_fns!(cal_gas_cost_5);
    impl_gas_costs_cal_fns!(cal_gas_cost_6);
    impl_gas_costs_cal_fns!(cal_gas_cost_7);
    impl_gas_costs_cal_fns!(cal_gas_cost_8);
    impl_gas_costs_cal_fns!(cal_gas_cost_9);
    impl_gas_costs_cal_fns!(cal_gas_cost_10);
    impl_gas_costs_cal_fns!(cal_gas_cost_11);
    impl_gas_costs_cal_fns!(cal_gas_cost_12);
    impl_gas_costs_cal_fns!(cal_gas_cost_13);
    impl_gas_costs_cal_fns!(cal_gas_cost_14);
    impl_gas_costs_cal_fns!(cal_gas_cost_15);
    impl_gas_costs_cal_fns!(cal_gas_cost_16);
    impl_gas_costs_cal_fns!(cal_gas_cost_17);
    impl_gas_costs_cal_fns!(cal_gas_cost_18);
    impl_gas_costs_cal_fns!(cal_gas_cost_19);
    impl_gas_costs_cal_fns!(cal_gas_cost_20);
    impl_gas_costs_cal_fns!(cal_gas_cost_21);
    impl_gas_costs_cal_fns!(cal_gas_cost_22);
    impl_gas_costs_cal_fns!(cal_gas_cost_23);
    impl_gas_costs_cal_fns!(cal_gas_cost_24);
}
