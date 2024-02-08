//! Benchmarking setup for pallet-move

use frame_benchmarking::v2::*;
use frame_system::{Config as SysConfig, RawOrigin};
use sp_runtime::traits::Zero;
use sp_std::vec;

use crate::{balance::BalanceOf, Config, *};

#[benchmarks(
    where
        T: Config + SysConfig,
        T::AccountId: From<AccountId32>,
        BalanceOf<T>: From<u128> + Into<u128>,
)]
mod benchmarks {
    use sp_core::{crypto::Ss58Codec, sr25519::Public};
    use sp_runtime::AccountId32;

    use super::*;

    const BOB_ADDR: &str = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty";

    fn addr32_from_ss58(ss58addr: &str) -> AccountId32 {
        let (pk, _) = Public::from_ss58check_with_version(ss58addr).unwrap();
        pk.into()
    }

    #[benchmark]
    fn execute() {
        let caller: T::AccountId = addr32_from_ss58(BOB_ADDR).into();
        let module =
            include_bytes!("../tests/assets/move-projects/move-basics/build/move-basics/bytecode_scripts/empty_scr.mv")
                .to_vec();
        let transaction = ScriptTransaction {
            bytecode: module,
            type_args: vec![],
            args: vec![],
        };
        let transaction_bc = bcs::to_bytes(&transaction).unwrap();
        #[extrinsic_call]
        execute(
            RawOrigin::Signed(caller),
            transaction_bc,
            100_000,
            BalanceOf::<T>::zero(),
        );
    }

    #[benchmark]
    fn publish_module() {
        let caller: T::AccountId = addr32_from_ss58(BOB_ADDR).into();
        let module = include_bytes!(
            "../tests/assets/move-projects/using_stdlib_natives/build/using_stdlib_natives/bytecode_modules/Vector.mv"
        )
        .to_vec();
        #[extrinsic_call]
        publish_module(RawOrigin::Signed(caller), module, 500_000);
    }

    #[benchmark]
    fn publish_module_bundle() {
        let caller: T::AccountId = addr32_from_ss58(BOB_ADDR).into();
        let module = include_bytes!(
            "../tests/assets/move-projects/using_stdlib_natives/build/using_stdlib_natives/bundles/using_stdlib_natives.mvb"
        )
        .to_vec();
        #[extrinsic_call]
        publish_module_bundle(RawOrigin::Signed(caller), module, 1_500_000);
    }

    impl_benchmark_test_suite!(MovePallet, crate::mock::new_test_ext(), crate::mock::Test);
}
