use crate::{mock::*, no_type_args, script_transaction, Error, Event, MultisigStorage};

use frame_support::{
    assert_err, assert_ok,
    pallet_prelude::DispatchResultWithPostInfo,
    traits::{tokens::WithdrawReasons, Currency},
};
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
    let (bob_addr_32, bob_addr_mv) = addrs_from_ss58(BOB_ADDR).unwrap();

    ExtBuilder::default()
        .with_balances(vec![(bob_addr_32.clone(), EXISTENTIAL_DEPOSIT)])
        .build()
        .execute_with(|| {
            // Roll to first block in case of block based event checkings and processes.
            roll_to(1);

            // eight_normal_signers(_s1: signer, _s2: signer, _s3: &signer, _s4: signer, _s5: &signer,
            // _s6: signer, _s7: &signer, _s8: &signer, _extra: u32)
            let script = assets::read_script_from_project("signer-scripts", "eight_normal_signers");
            let type_args: Vec<TypeTag> = vec![];

            let mut pg = ParamGenerator::new();
            let s1 = pg.address(&bob_addr_mv);
            let extra = pg.rand::<u32>();
            let params: Vec<&[u8]> = vec![&s1, &s1, &s1, &s1, &s1, &s1, &s1, &s1, &extra];

            assert_ok!(execute_script(
                bob_addr_32.clone(),
                script.clone(),
                params.clone(),
                type_args.clone()
            ));
            assert_eq!(
                last_event(),
                RuntimeEvent::MoveModule(Event::<Test>::ExecuteCalled {
                    who: vec![bob_addr_32]
                })
            );
        })
}

/// Script with many signers parameters fails if all signers don't provide an actual signature.
#[test]
fn eve_cant_execute_multisig_script_without_other_signers_works() {
    ExtBuilder::default().build().execute_with(|| {
        let mut pg = ParamGenerator::new();
        // Eve is basically Bob here, but since Bob is pretending to be bad, we'll rename him.
        let (eve_addr_32, eve_addr_mv) = addrs_from_ss58(BOB_ADDR).unwrap();
        let (alice_addr_32, alice_addr_mv) = addrs_from_ss58(ALICE_ADDR).unwrap();

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
        let result = execute_script(
            eve_addr_32,
            script.clone(),
            params.clone(),
            type_args.clone(),
        );
        assert_err!(result, Error::<Test>::UserHasAlreadySigned);
        assert_ok!(execute_script(
            alice_addr_32.clone(),
            script.clone(),
            params.clone(),
            type_args.clone()
        ));
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

#[test]
fn multiple_signers_in_multisig_script_works() {
    const BALANCE: Balance = 80_000_000_000_000;
    const CHANGE: Balance = 20_000_000_000_000;
    let (bob_addr_32, bob_addr_mv) = addrs_from_ss58(BOB_ADDR).unwrap();
    let (alice_addr_32, alice_addr_mv) = addrs_from_ss58(ALICE_ADDR).unwrap();
    let (dave_addr_32, dave_addr_mv) = addrs_from_ss58(DAVE_ADDR).unwrap();
    let (eve_addr_32, eve_addr_mv) = addrs_from_ss58(EVE_ADDR).unwrap();

    ExtBuilder::default()
        .with_balances(vec![
            (bob_addr_32.clone(), BALANCE),
            (alice_addr_32.clone(), BALANCE),
            (dave_addr_32.clone(), BALANCE),
            (eve_addr_32.clone(), BALANCE),
        ])
        .build()
        .execute_with(|| {
            // Roll to first block in case of block based event checkings and processes.
            roll_to(1);

            // Initialisation & Setup by developer Bob.
            let module = assets::read_module_from_project("multiple-signers", "Dorm");
            assert_ok!(MoveModule::publish_module(
                RuntimeOrigin::signed(bob_addr_32.clone()),
                module,
                MAX_GAS_AMOUNT
            ));
            assert_eq!(
                last_event(),
                RuntimeEvent::MoveModule(Event::<Test>::ModulePublished {
                    who: bob_addr_32.clone()
                })
            );

            let script = assets::read_script_from_project("multiple-signers", "init_module");
            let transaction_bc = script_transaction!(script, no_type_args!(), &bob_addr_mv);
            assert_ok!(MoveModule::execute(
                RuntimeOrigin::signed(bob_addr_32.clone()),
                transaction_bc.clone(),
                MAX_GAS_AMOUNT,
                BALANCE,
            ));
            assert_eq!(
                last_event(),
                RuntimeEvent::MoveModule(Event::<Test>::ExecuteCalled {
                    who: vec![bob_addr_32.clone()]
                })
            );

            // Now our three tenants want to rent the 3-room apartment.
            let script = assets::read_script_from_project("multiple-signers", "rent_apartment");
            let transaction_bc = script_transaction!(
                script,
                no_type_args!(),
                &alice_addr_mv,
                &dave_addr_mv,
                &eve_addr_mv,
                &2u8
            );
            let call_hash = MoveModule::transaction_bc_call_hash(&transaction_bc[..]);

            // Verify that no lock has been set so far and that the Multisig request entry cannot
            // be found in storage.
            assert!(MultisigStorage::<Test>::try_get(call_hash).is_err());
            assert_ok!(Balances::ensure_can_withdraw(
                &alice_addr_32,
                BALANCE,
                WithdrawReasons::TRANSFER,
                0
            ));
            assert_ok!(Balances::ensure_can_withdraw(
                &dave_addr_32,
                BALANCE,
                WithdrawReasons::TRANSFER,
                0
            ));
            assert_ok!(Balances::ensure_can_withdraw(
                &eve_addr_32,
                BALANCE,
                WithdrawReasons::TRANSFER,
                0
            ));

            assert_ok!(MoveModule::execute(
                RuntimeOrigin::signed(alice_addr_32.clone()),
                transaction_bc.clone(),
                MAX_GAS_AMOUNT,
                BALANCE,
            ));
            assert_eq!(
                last_event(),
                RuntimeEvent::MoveModule(Event::<Test>::SignedMultisigScript {
                    who: alice_addr_32.clone()
                })
            );
            assert!(MultisigStorage::<Test>::try_get(call_hash).is_ok());
            assert!(Balances::ensure_can_withdraw(
                &alice_addr_32,
                BALANCE,
                WithdrawReasons::TRANSFER,
                0
            )
            .is_err());
            assert_ok!(Balances::ensure_can_withdraw(
                &dave_addr_32,
                BALANCE,
                WithdrawReasons::TRANSFER,
                0
            ));
            assert_ok!(Balances::ensure_can_withdraw(
                &eve_addr_32,
                BALANCE,
                WithdrawReasons::TRANSFER,
                0
            ));

            assert_ok!(MoveModule::execute(
                RuntimeOrigin::signed(dave_addr_32.clone()),
                transaction_bc.clone(),
                MAX_GAS_AMOUNT,
                BALANCE,
            ));
            assert_eq!(
                last_event(),
                RuntimeEvent::MoveModule(Event::<Test>::SignedMultisigScript {
                    who: dave_addr_32.clone()
                })
            );
            // Now this candidate should also not be able to transfer the locked tokens.
            assert!(Balances::ensure_can_withdraw(
                &alice_addr_32,
                BALANCE,
                WithdrawReasons::TRANSFER,
                0
            )
            .is_err());
            assert!(Balances::ensure_can_withdraw(
                &dave_addr_32,
                BALANCE,
                WithdrawReasons::TRANSFER,
                0
            )
            .is_err());
            assert_ok!(Balances::ensure_can_withdraw(
                &eve_addr_32,
                BALANCE,
                WithdrawReasons::TRANSFER,
                0
            ));

            assert_ok!(MoveModule::execute(
                RuntimeOrigin::signed(eve_addr_32.clone()),
                transaction_bc.clone(),
                MAX_GAS_AMOUNT,
                BALANCE,
            ));
            assert_eq!(
                last_event(),
                RuntimeEvent::MoveModule(Event::<Test>::ExecuteCalled {
                    who: vec![
                        dave_addr_32.clone(),
                        alice_addr_32.clone(),
                        eve_addr_32.clone()
                    ]
                })
            );
            assert_ok!(Balances::ensure_can_withdraw(
                &alice_addr_32,
                CHANGE,
                WithdrawReasons::TRANSFER,
                0
            ));
            assert_ok!(Balances::ensure_can_withdraw(
                &dave_addr_32,
                CHANGE,
                WithdrawReasons::TRANSFER,
                0
            ));
            assert_ok!(Balances::ensure_can_withdraw(
                &eve_addr_32,
                CHANGE,
                WithdrawReasons::TRANSFER,
                0
            ));
            // Now after all signers have signed the multsig script, we can expect that the script
            // will be executed and the multisg storage will remove the pending mutlisig script
            // data, since the script has been executed.
            assert!(MultisigStorage::<Test>::try_get(call_hash).is_err());
        })
}

/// Multi-signer script execution request gets removed after defined period.
#[test]
fn verify_old_multi_signer_requests_getting_removed() {
    const BALANCE: Balance = 80_000_000_000_000;
    let (bob_addr_32, bob_addr_mv) = addrs_from_ss58(BOB_ADDR).unwrap();
    let (alice_addr_32, alice_addr_mv) = addrs_from_ss58(ALICE_ADDR).unwrap();
    let (dave_addr_32, dave_addr_mv) = addrs_from_ss58(DAVE_ADDR).unwrap();
    let (eve_addr_32, eve_addr_mv) = addrs_from_ss58(EVE_ADDR).unwrap();

    ExtBuilder::default()
        .with_balances(vec![
            (bob_addr_32.clone(), BALANCE),
            (alice_addr_32.clone(), BALANCE),
            (dave_addr_32.clone(), BALANCE),
            (eve_addr_32.clone(), BALANCE),
        ])
        .build()
        .execute_with(|| {
            // Roll to first block in case of block based event checkings and processes.
            roll_to(1);

            // Initialisation & Setup by developer Bob.
            let module = assets::read_module_from_project("multiple-signers", "Dorm");
            assert_ok!(MoveModule::publish_module(
                RuntimeOrigin::signed(bob_addr_32.clone()),
                module,
                MAX_GAS_AMOUNT
            ));
            let script = assets::read_script_from_project("multiple-signers", "init_module");
            let transaction_bc = script_transaction!(script, no_type_args!(), &bob_addr_mv);
            assert_ok!(MoveModule::execute(
                RuntimeOrigin::signed(bob_addr_32.clone()),
                transaction_bc.clone(),
                MAX_GAS_AMOUNT,
                BALANCE,
            ));

            // Now only 2 of 3 planned signers will sign the script execution.
            let script = assets::read_script_from_project("multiple-signers", "rent_apartment");
            let transaction_bc = script_transaction!(
                script,
                no_type_args!(),
                &alice_addr_mv,
                &dave_addr_mv,
                &eve_addr_mv,
                &2u8
            );
            assert_ok!(MoveModule::execute(
                RuntimeOrigin::signed(alice_addr_32.clone()),
                transaction_bc.clone(),
                MAX_GAS_AMOUNT,
                BALANCE,
            ));
            assert_ok!(MoveModule::execute(
                RuntimeOrigin::signed(dave_addr_32.clone()),
                transaction_bc.clone(),
                MAX_GAS_AMOUNT,
                BALANCE,
            ));
            // Sloppy or distrustful Eve is missing...

            // Verify expected Multisig data in storage.
            let call_hash = MoveModule::transaction_bc_call_hash(&transaction_bc[..]);
            let _request = MultisigStorage::<Test>::try_get(call_hash).unwrap();

            // Let's roll forward to block number 4 and check our request still exists.
            roll_to(5);
            assert!(MultisigStorage::<Test>::try_get(call_hash).is_ok());

            // One more block forward and it shall be removed!
            roll_to(6);
            assert!(MultisigStorage::<Test>::try_get(call_hash).is_err());
            assert_eq!(
                last_event(),
                RuntimeEvent::MoveModule(Event::<Test>::MultiSignRequestRemoved {
                    call: vec![call_hash],
                })
            );

            // If Eve now tries to sign that multi-signer request, a new request will be created.
            assert_ok!(MoveModule::execute(
                RuntimeOrigin::signed(eve_addr_32.clone()),
                transaction_bc.clone(),
                MAX_GAS_AMOUNT,
                BALANCE,
            ));
            assert_eq!(
                last_event(),
                RuntimeEvent::MoveModule(Event::<Test>::SignedMultisigScript {
                    who: eve_addr_32.clone()
                })
            );
        })
}

#[test]
fn cheque_limit_in_multi_signer_execution_works() {
    const BALANCE: Balance = 80_000_000_000_000;
    let (bob_addr_32, bob_addr_mv) = addrs_from_ss58(BOB_ADDR).unwrap();
    let (alice_addr_32, alice_addr_mv) = addrs_from_ss58(ALICE_ADDR).unwrap();
    let (dave_addr_32, dave_addr_mv) = addrs_from_ss58(DAVE_ADDR).unwrap();
    let (eve_addr_32, eve_addr_mv) = addrs_from_ss58(EVE_ADDR).unwrap();

    ExtBuilder::default()
        .with_balances(vec![
            (bob_addr_32.clone(), BALANCE),
            (alice_addr_32.clone(), BALANCE),
            (dave_addr_32.clone(), BALANCE),
            (eve_addr_32.clone(), BALANCE),
        ])
        .build()
        .execute_with(|| {
            // Roll to first block in case of block based event checkings and processes.
            roll_to(1);

            // Initialisation & Setup by developer Bob.
            let module = assets::read_module_from_project("multiple-signers", "Dorm");
            assert_ok!(MoveModule::publish_module(
                RuntimeOrigin::signed(bob_addr_32.clone()),
                module,
                MAX_GAS_AMOUNT
            ));
            let script = assets::read_script_from_project("multiple-signers", "init_module");
            let transaction_bc = script_transaction!(script, no_type_args!(), &bob_addr_mv);
            assert_ok!(MoveModule::execute(
                RuntimeOrigin::signed(bob_addr_32.clone()),
                transaction_bc.clone(),
                MAX_GAS_AMOUNT,
                BALANCE,
            ));

            // Now only 2 of 3 planned signers will sign the script execution.
            let script = assets::read_script_from_project("multiple-signers", "rent_apartment");
            let transaction_bc = script_transaction!(
                script,
                no_type_args!(),
                &alice_addr_mv,
                &dave_addr_mv,
                &eve_addr_mv,
                &2u8
            );
            assert_ok!(MoveModule::execute(
                RuntimeOrigin::signed(dave_addr_32.clone()),
                transaction_bc.clone(),
                MAX_GAS_AMOUNT,
                BALANCE,
            ));
            assert_ok!(MoveModule::execute(
                RuntimeOrigin::signed(eve_addr_32.clone()),
                transaction_bc.clone(),
                MAX_GAS_AMOUNT,
                BALANCE,
            ));
            let res = MoveModule::execute(
                RuntimeOrigin::signed(alice_addr_32.clone()),
                transaction_bc.clone(),
                MAX_GAS_AMOUNT,
                BALANCE / 2,
            );
            assert!(verify_module_error_with_msg(res, "Aborted").unwrap());
        })
}
