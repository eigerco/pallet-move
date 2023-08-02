//! Benchmarking setup for pallet-move
#![cfg(feature = "runtime-benchmarks")]
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

use super::*;
#[allow(unused)]
use crate::Pallet as MovePallet;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn execute() {
        let caller: T::AccountId = whitelisted_caller();
        let module =
            include_bytes!("../tests/assets/move/build/move/bytecode_scripts/empty_scr.mv")
                .to_vec();
        #[extrinsic_call]
        execute(RawOrigin::Signed(caller), module, 100_000);
    }

    #[benchmark]
    fn publish_module() {
        let caller: T::AccountId = whitelisted_caller();
        let module =
            include_bytes!("../tests/assets/move/build/move/bytecode_modules/Empty.mv").to_vec();
        #[extrinsic_call]
        publish_module(RawOrigin::Signed(caller), module, 500_000);
    }

    #[benchmark]
    fn publish_package() {
        let caller: T::AccountId = whitelisted_caller();
        let module =
            include_bytes!("../tests/assets/move/build/move/bytecode_modules/Empty.mv").to_vec();
        #[extrinsic_call]
        publish_package(RawOrigin::Signed(caller), module, 1_500_000);
    }

    impl_benchmark_test_suite!(MovePallet, crate::mock::new_test_ext(), crate::mock::Test);
}
