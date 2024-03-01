//! Benchmarking setup for pallet-move.
#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::*;
use frame_system::{Config as SysConfig, RawOrigin};
use sp_runtime::{traits::Zero, AccountId32};
use sp_std::vec;

use crate::{balance::BalanceOf, mock::*, *};

benchmarks! {
    where_clause { where
        T: Config + SysConfig,
        T::AccountId: From<AccountId32>,
        BalanceOf<T>: From<u128> + Into<u128>,
    }

    execute {
        let caller: T::AccountId = addr32_from_ss58(BOB_ADDR).unwrap().into();
        let script = assets::read_script_from_project("move-basics", "empty_scr");
        let transaction_bc = script_transaction!(script, no_type_args!());
    }: _(RawOrigin::Signed(caller), transaction_bc, 100_000, BalanceOf::<T>::zero())
    verify {}

    publish_module {
        let caller: T::AccountId = addr32_from_ss58(BOB_ADDR).unwrap().into();
        let module = assets::read_module_from_project("using_stdlib_natives", "Vector");
    }: _(RawOrigin::Signed(caller), module, 500_000)
    verify {}

    publish_module_bundle {
        let caller: T::AccountId = addr32_from_ss58(BOB_ADDR).unwrap().into();
        let bundle = assets::read_bundle_from_project("using_stdlib_natives", "using_stdlib_natives");
    }: _(RawOrigin::Signed(caller), bundle, 1_500_000)
    verify {}
}

impl_benchmark_test_suite!(
    Pallet,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Test
);
