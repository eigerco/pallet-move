//! Benchmarking setup for substrate-movevm-pallet
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as MovePallet;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn execute() {
        let caller: T::AccountId = whitelisted_caller();
        #[extrinsic_call]
        execute(RawOrigin::Signed(caller));
    }

    impl_benchmark_test_suite!(MovePallet, crate::mock::new_test_ext(), crate::mock::Test);
}