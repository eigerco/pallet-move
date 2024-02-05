//! Benchmarking setup for pallet-move
#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::v2::*;
use frame_system::{Config as SysConfig, RawOrigin};
use sp_runtime::traits::Zero;
use sp_std::vec;

use super::*;
use crate::{balance::BalanceOf, Config};

#[benchmarks(
    where
        T: Config + SysConfig,
        BalanceOf<T>: From<u128> + Into<u128>,
)]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn execute() {
        let caller: T::AccountId = whitelisted_caller();
        let module =
            include_bytes!("../tests/assets/move-projects/move-basics/build/move-basics/bytecode_scripts/empty_scr.mv")
                .to_vec();
        #[extrinsic_call]
        execute(
            RawOrigin::Signed(caller),
            module,
            100_000,
            BalanceOf::<T>::zero(),
        );
    }

    #[benchmark]
    fn publish_module() {
        let caller: T::AccountId = whitelisted_caller();
        let module = include_bytes!(
            "../tests/assets/move-projects/move-basics/build/move-basics/bytecode_modules/Empty.mv"
        )
        .to_vec();
        #[extrinsic_call]
        publish_module(RawOrigin::Signed(caller), module, 500_000);
    }

    #[benchmark]
    fn publish_module_bundle() {
        let caller: T::AccountId = whitelisted_caller();
        let module = include_bytes!(
            "../tests/assets/move-projects/move-basics/build/move-basics/bytecode_modules/Empty.mv"
        )
        .to_vec();
        #[extrinsic_call]
        publish_module_bundle(RawOrigin::Signed(caller), module, 1_500_000);
    }

    impl_benchmark_test_suite!(MovePallet, crate::mock::new_test_ext(), crate::mock::Test);
}
