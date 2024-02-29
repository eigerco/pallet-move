use crate::mock::*;
use crate::Error;

use frame_support::{assert_err, assert_ok, pallet_prelude::DispatchResultWithPostInfo};
use move_core_types::{language_storage::TypeTag, u256::U256};
use move_vm_backend::types::MAX_GAS_AMOUNT;
use rand::{distributions::Standard, prelude::Distribution, rngs::ThreadRng, Rng};
use serde::Serialize;

fn execute_script(
    who: AccountId32,
    script: Vec<u8>,
    params: Vec<&[u8]>,
    type_args: Vec<TypeTag>,
) -> DispatchResultWithPostInfo {
    let transaction = bcs::to_bytes(&ScriptTransaction {
        bytecode: script,
        type_args,
        args: params.iter().map(|x| x.to_vec()).collect(),
    })
    .unwrap();

    MoveModule::execute(
        RuntimeOrigin::signed(who.clone()),
        transaction,
        MAX_GAS_AMOUNT,
        EMPTY_CHEQUE,
    )
}

// Quick BCS parameter deserializer.
struct ParamGenerator {
    rng: ThreadRng,
}

impl ParamGenerator {
    fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
        }
    }

    fn rand<T: Serialize>(&mut self) -> Vec<u8>
    where
        Standard: Distribution<T>,
    {
        bcs::to_bytes(&self.rng.gen::<T>()).unwrap()
    }

    fn rand_vec_with_len<T: Serialize>(&mut self, len: usize) -> Vec<u8>
    where
        Standard: Distribution<T>,
    {
        let mut v = vec![];
        for _ in 0..len {
            v.push(self.rng.gen::<T>());
        }
        bcs::to_bytes(&v).unwrap()
    }

    // Deliberately not using associated function for code readiblity.
    fn address_from(&self, hex: &str) -> Vec<u8> {
        bcs::to_bytes(&AccountAddress::from_hex_literal(hex).unwrap()).unwrap()
    }

    // Deliberately not using associated function for code readiblity.
    fn address(&self, addr: &AccountAddress) -> Vec<u8> {
        bcs::to_bytes(&addr).unwrap()
    }

    // Deliberately not using associated function for code readiblity.
    fn address_vec(&self, addr: Vec<&AccountAddress>) -> Vec<u8> {
        bcs::to_bytes(&addr).unwrap()
    }

    // Deliberately not using associated function for code readiblity.
    fn u256_const(&self) -> Vec<u8> {
        let num = U256::one();
        bcs::to_bytes(&num).unwrap()
    }

    fn const_vec_u256_with_len(&mut self, len: usize) -> Vec<u8> {
        let mut v = vec![];

        for _ in 0..len {
            if len % 2 == 0 {
                v.push(U256::zero());
            } else {
                v.push(U256::one());
            }
        }
        bcs::to_bytes(&v).unwrap()
    }
}

/// Script without any parameters executes correctly by anyone.
#[test]
fn general_script_no_params_works() {
    ExtBuilder::default().build().execute_with(|| {
        let (bob_addr_32, _) = addrs_from_ss58(BOB_ADDR).unwrap();

        // no_param_at_all()
        let script = assets::read_script_from_project("signer-scripts", "no_param_at_all");
        let type_args: Vec<TypeTag> = vec![];
        let params: Vec<&[u8]> = vec![];
        assert_ok!(execute_script(bob_addr_32, script, params, type_args));
    })
}

/// Script with many non-signers parameters executes correctly by anyone.
#[test]
fn general_script_no_signers_param_at_all_works() {
    ExtBuilder::default().build().execute_with(|| {
        let mut pg = ParamGenerator::new();
        let (bob_addr_32, _) = addrs_from_ss58(BOB_ADDR).unwrap();

        // no_signers_param_at_all(iterations: u64, _a: u32, _b: u8, _c: u256, _d: address, _e: vector<u32>, _f: bool)
        let script = assets::read_script_from_project("signer-scripts", "no_signers_param_at_all");
        let type_args: Vec<TypeTag> = vec![];

        let iter = pg.rand::<u64>();
        let a = pg.rand::<u32>();
        let b = pg.rand::<u8>();
        let c = pg.u256_const();
        let d = pg.address_from("0xAE");
        let e = pg.rand_vec_with_len::<u32>(4);
        let f = pg.rand::<bool>();
        let params: Vec<&[u8]> = vec![&iter, &a, &b, &c, &d, &e, &f];

        assert_ok!(execute_script(bob_addr_32, script, params, type_args));
    })
}

/// Script with many signers parameters executes correctly when all signers are signed by one account.
#[test]
fn general_script_eight_normal_signers_works() {
    let user1_native = int_to_addr32(1);
    let user1_move = addr32_to_move(&user1_native).unwrap();
    let user2_native = int_to_addr32(2);
    let user2_move = addr32_to_move(&user2_native).unwrap();
    let user3_native = int_to_addr32(3);
    let user3_move = addr32_to_move(&user3_native).unwrap();
    let user4_native = int_to_addr32(4);
    let user4_move = addr32_to_move(&user4_native).unwrap();
    let user5_native = int_to_addr32(5);
    let user5_move = addr32_to_move(&user5_native).unwrap();
    let user6_native = int_to_addr32(6);
    let user6_move = addr32_to_move(&user6_native).unwrap();
    let user7_native = int_to_addr32(7);
    let user7_move = addr32_to_move(&user7_native).unwrap();
    let user8_native = int_to_addr32(8);
    let user8_move = addr32_to_move(&user8_native).unwrap();

    ExtBuilder::default()
        .with_balances(vec![
            (user1_native.clone(), EXISTENTIAL_DEPOSIT),
            (user2_native.clone(), EXISTENTIAL_DEPOSIT),
            (user3_native.clone(), EXISTENTIAL_DEPOSIT),
            (user4_native.clone(), EXISTENTIAL_DEPOSIT),
            (user5_native.clone(), EXISTENTIAL_DEPOSIT),
            (user6_native.clone(), EXISTENTIAL_DEPOSIT),
            (user7_native.clone(), EXISTENTIAL_DEPOSIT),
            (user8_native.clone(), EXISTENTIAL_DEPOSIT),
        ])
        .build()
        .execute_with(|| {
            // eight_normal_signers(_s1: signer, _s2: signer, _s3: &signer, _s4: signer, _s5: &signer,
            // _s6: signer, _s7: &signer, _s8: &signer, _extra: u32)
            let script = assets::read_script_from_project("signer-scripts", "eight_normal_signers");
            let type_args: Vec<TypeTag> = vec![];

            let mut pg = ParamGenerator::new();
            let s1 = pg.address(&user1_move);
            let s2 = pg.address(&user2_move);
            let s3 = pg.address(&user3_move);
            let s4 = pg.address(&user4_move);
            let s5 = pg.address(&user5_move);
            let s6 = pg.address(&user6_move);
            let s7 = pg.address(&user7_move);
            let s8 = pg.address(&user8_move);
            let extra = pg.rand::<u32>();
            let params: Vec<&[u8]> = vec![&s1, &s2, &s3, &s4, &s5, &s6, &s7, &s8, &extra];

            assert_ok!(execute_script(
                user1_native,
                script.clone(),
                params.clone(),
                type_args.clone()
            ));
            assert_ok!(execute_script(
                user2_native,
                script.clone(),
                params.clone(),
                type_args.clone()
            ));
            assert_ok!(execute_script(
                user3_native,
                script.clone(),
                params.clone(),
                type_args.clone()
            ));
            assert_ok!(execute_script(
                user4_native,
                script.clone(),
                params.clone(),
                type_args.clone()
            ));
            assert_ok!(execute_script(
                user5_native,
                script.clone(),
                params.clone(),
                type_args.clone()
            ));
            assert_ok!(execute_script(
                user6_native,
                script.clone(),
                params.clone(),
                type_args.clone()
            ));
            assert_ok!(execute_script(
                user7_native,
                script.clone(),
                params.clone(),
                type_args.clone()
            ));
            assert_ok!(execute_script(
                user8_native,
                script.clone(),
                params.clone(),
                type_args.clone()
            ));
        })
}

/// Script with many signers parameters fails if all signers don't provide an actual signature.
#[test]
fn general_script_eight_normal_signers_where_eve_tries_to_forge_signers_fails() {
    ExtBuilder::default().build().execute_with(|| {
        let mut pg = ParamGenerator::new();
        // Eve is basically Bob here, but since Bob is pretending to be bad, we'll rename him.
        let (eve_addr_32, eve_addr_mv) = addrs_from_ss58(BOB_ADDR).unwrap();
        let (_, alice_addr_mv) = addrs_from_ss58(ALICE_ADDR).unwrap();

        // eight_normal_signers(_s1: signer, _s2: signer, _s3: &signer, _s4: signer, _s5: &signer,
        // _s6: signer, _s7: &signer, _s8: &signer, _extra: u32)
        let script = assets::read_script_from_project("signer-scripts", "eight_normal_signers");
        let type_args: Vec<TypeTag> = vec![];

        let alice = pg.address(&alice_addr_mv);
        let eve = pg.address(&eve_addr_mv);
        let extra = pg.rand::<u32>();
        let params: Vec<&[u8]> = vec![&eve, &eve, &alice, &eve, &eve, &eve, &eve, &eve, &extra];

        assert_ok!(execute_script(
            eve_addr_32.clone(),
            script.clone(),
            params.clone(),
            type_args.clone()
        ));
        let result = execute_script(eve_addr_32, script, params, type_args);
        assert_err!(result, Error::<Test>::UserHasAlreadySigned);
    })
}

/// Script with a signer before all possible vector parameters should execute fine.
#[test]
fn signer_before_all_possible_vectors_works() {
    ExtBuilder::default().build().execute_with(|| {
        let mut pg = ParamGenerator::new();
        let (bob_addr_32, bob_addr_mv) = addrs_from_ss58(BOB_ADDR).unwrap();

        // signer_before_all_possible_vectors(_s: signer, _a: vector<u8>, _b: vector<u16>, _c:
        // vector<u32>, _e: vector<u64>, _f: vector<u128>, _g: vector<u256>, _h: vector<address>,
        // _i: vector<bool>)
        let script = assets::read_script_from_project(
            "signer-scripts",
            "signer_before_all_possible_vectors",
        );
        let type_args: Vec<TypeTag> = vec![];

        let bob = pg.address(&bob_addr_mv);
        let v_u8 = pg.rand_vec_with_len::<u8>(1);
        let v_u16 = pg.rand_vec_with_len::<u16>(2);
        let v_u32 = pg.rand_vec_with_len::<u32>(3);
        let v_u64 = pg.rand_vec_with_len::<u64>(2);
        let v_u128 = pg.rand_vec_with_len::<u128>(2);
        let v_u256 = pg.const_vec_u256_with_len(2);
        let v_addr = pg.address_vec(vec![&bob_addr_mv]);
        let v_bool = pg.rand_vec_with_len::<bool>(5);
        let params: Vec<&[u8]> = vec![
            &bob, &v_u8, &v_u16, &v_u32, &v_u64, &v_u128, &v_u256, &v_addr, &v_bool,
        ];

        assert_ok!(execute_script(bob_addr_32, script, params, type_args));
    })
}

/// Script with a signer after all possible vector parameters should fail.
#[test]
fn signer_after_all_possible_vectors_fails() {
    ExtBuilder::default().build().execute_with(|| {
        let mut pg = ParamGenerator::new();
        let (bob_addr_32, bob_addr_mv) = addrs_from_ss58(BOB_ADDR).unwrap();

        // signer_after_all_possible_vectors(_a: vector<u8>, _b: vector<u16>, _c: vector<u32>, _e:
        // vector<u64>, _f: vector<u128>, _g: vector<u256>, _h: vector<address>, _i: vector<bool>,
        // _s: &signer)
        let script =
            assets::read_script_from_project("signer-scripts", "signer_after_all_possible_vectors");
        let type_args: Vec<TypeTag> = vec![];

        let v_u8 = pg.rand_vec_with_len::<u8>(1);
        let v_u16 = pg.rand_vec_with_len::<u16>(2);
        let v_u32 = pg.rand_vec_with_len::<u32>(3);
        let v_u64 = pg.rand_vec_with_len::<u64>(2);
        let v_u128 = pg.rand_vec_with_len::<u128>(2);
        let v_u256 = pg.const_vec_u256_with_len(2);
        let v_addr = pg.address_vec(vec![&bob_addr_mv, &bob_addr_mv]);
        let v_bool = pg.rand_vec_with_len::<bool>(5);
        let bob = pg.address(&bob_addr_mv);
        let params: Vec<&[u8]> = vec![
            &v_u8, &v_u16, &v_u32, &v_u64, &v_u128, &v_u256, &v_addr, &v_bool, &bob,
        ];

        let result = execute_script(bob_addr_32, script, params, type_args);
        assert_err!(result, Error::<Test>::InvalidMainFunctionSignature);
    })
}

/// Script with a vector that contains a signer should fail.
#[test]
fn script_with_vector_containing_signer_fails() {
    ExtBuilder::default().build().execute_with(|| {
        let pg = ParamGenerator::new();
        let (bob_addr_32, bob_addr_mv) = addrs_from_ss58(BOB_ADDR).unwrap();

        // trying_vector_containing_signer(_v: vector<signer>)
        let script =
            assets::read_script_from_project("signer-scripts", "trying_vector_containing_signer");
        let type_args: Vec<TypeTag> = vec![];

        let v_addr = pg.address_vec(vec![&bob_addr_mv]);
        let params: Vec<&[u8]> = vec![&v_addr];

        let result = execute_script(bob_addr_32, script, params, type_args);
        assert_err!(result, Error::<Test>::InvalidMainFunctionSignature);
    })
}
