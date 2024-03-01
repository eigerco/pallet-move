use frame_support::{
    parameter_types,
    traits::{ConstU128, ConstU16, ConstU32, ConstU64},
};
use sp_core::{crypto::Ss58Codec, sr25519::Public, H256};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

use crate as pallet_move;
use crate::Error;

pub use move_core_types::account_address::AccountAddress;
pub use move_vm_backend_common::types::ScriptTransaction;
pub use sp_runtime::AccountId32;

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        MoveModule: pallet_move,
    }
);

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

pub type Balance = u128;
pub const EXISTENTIAL_DEPOSIT: u128 = 100;

impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type RuntimeHoldReason = ();
    type MaxHolds = ();
    type RuntimeFreezeReason = ();
}

parameter_types! {
    pub const MaxRequestLifetime: u32 = 5;
    pub const MaxScriptSigners: u32 = 8;
}

impl pallet_move::Config for Test {
    type Currency = Balances;
    type MaxRequestLifetime = MaxRequestLifetime;
    type MaxScriptSigners = MaxScriptSigners;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

/// Test Externalities Builder for an easier test setup.
#[derive(Default)]
pub(crate) struct ExtBuilder {
    /// Overwrite default accounts with balances.
    balances: Vec<(AccountId32, Balance)>,
    /// Overwrite default Move-stdlib setup.
    move_stdlib: Option<Vec<u8>>,
    /// Overwrite default Substrate-stdlib.
    substrate_stdlib: Option<Vec<u8>>,
}

impl ExtBuilder {
    /// Overwrites default balances on dev-test setup.
    pub(crate) fn with_balances(mut self, balances: Vec<(AccountId32, Balance)>) -> Self {
        self.balances = balances;
        self
    }

    /// Overwrites default Move-stdlib dev-test setup.
    pub(crate) fn with_move_stdlib(mut self, move_stdlib: Option<Vec<u8>>) -> Self {
        self.move_stdlib = move_stdlib;
        self
    }

    /// Overwrites default Substrate-stdlib on dev-test setup.
    pub(crate) fn with_substrate_stdlib(mut self, sub_stdlib: Option<Vec<u8>>) -> Self {
        self.substrate_stdlib = sub_stdlib;
        self
    }

    pub(crate) fn build(self) -> sp_io::TestExternalities {
        let mut ext = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("Frame system builds valid default genesis config");

        pallet_balances::GenesisConfig::<Test> {
            balances: self.balances.clone(),
        }
        .assimilate_storage(&mut ext)
        .expect("Pallet balances storage cannot be assimilated");

        pallet_move::GenesisConfig::<Test> {
            _phantom: core::marker::PhantomData,
            change_default_move_stdlib_bundle_to: self.move_stdlib.clone(),
            change_default_substrate_stdlib_bundle_to: self.substrate_stdlib.clone(),
        }
        .assimilate_storage(&mut ext)
        .expect("Pallet Move storage cannot be assimilated");

        ext.into()
    }
}

// Common constants accross the tests.
pub const EMPTY_CHEQUE: u128 = 0; // Not all scripts need the `cheque_amount` parameter.
pub const CAFE_ADDR: &str = "5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSrvWGWqi1eSv4fmh4G"; // == 0xCAFE
pub const BOB_ADDR: &str = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty";
pub const ALICE_ADDR: &str = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
pub const DAVE_ADDR: &str = "5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy";

pub(crate) fn addr32_from_ss58(ss58addr: &str) -> Result<AccountId32, Error<Test>> {
    let (pk, _) = Public::from_ss58check_with_version(ss58addr)
        .map_err(|_| Error::<Test>::InvalidAccountSize)?;
    let account: AccountId32 = pk.into();
    Ok(account)
}

pub(crate) fn addr32_to_move(addr32: &AccountId32) -> Result<AccountAddress, Error<Test>> {
    MoveModule::to_move_address(addr32)
}

pub(crate) fn addrs_from_ss58(ss58: &str) -> Result<(AccountId32, AccountAddress), Error<Test>> {
    let addr_32 = addr32_from_ss58(ss58)?;
    let addr_mv = addr32_to_move(&addr_32)?;
    Ok((addr_32, addr_mv))
}

pub(crate) fn int_to_addr32(n: u128) -> AccountId32 {
    let mut addr = [0u8; 32];
    let bytes: [u8; 16] = n.to_be_bytes();
    bytes.iter().enumerate().for_each(|(i, b)| addr[i] = *b);
    AccountId32::from(addr)
}

pub mod assets {
    const MOVE_PROJECTS: &str = "src/tests/assets/move-projects";

    /// Reads bytes from a file for the given path.
    /// Can panic if the file doesn't exist.
    fn read_bytes(file_path: &str) -> Vec<u8> {
        std::fs::read(file_path)
            .unwrap_or_else(|e| panic!("Can't read {file_path}: {e} - make sure you run pallet-move/tests/assets/move-projects/smove-build-all.sh"))
    }

    /// Reads a precompiled Move module from our assets directory.
    pub fn read_module_from_project(project: &str, module_name: &str) -> Vec<u8> {
        let path =
            format!("{MOVE_PROJECTS}/{project}/build/{project}/bytecode_modules/{module_name}.mv");
        read_bytes(&path)
    }

    /// Reads a precompiled Move bundle from our assets directory.
    pub fn read_bundle_from_project(project: &str, bundle_name: &str) -> Vec<u8> {
        let path = format!("{MOVE_PROJECTS}/{project}/build/{project}/bundles/{bundle_name}.mvb");
        read_bytes(&path)
    }

    /// Reads a precompiled Move scripts from our assets directory.
    pub fn read_script_from_project(project: &str, script_name: &str) -> Vec<u8> {
        let path =
            format!("{MOVE_PROJECTS}/{project}/build/{project}/bytecode_scripts/{script_name}.mv");
        read_bytes(&path)
    }
}

#[macro_export]
macro_rules! script_transaction {
    ($bytecode:expr, $type_args:expr $(, $args:expr)*) => {
        {
            let transaction = ScriptTransaction {
                bytecode: $bytecode,
                type_args: $type_args,
                args: vec![$(bcs::to_bytes($args).unwrap()),*],
            };
            bcs::to_bytes(&transaction).unwrap()
        }
    }
}

#[macro_export]
macro_rules! no_type_args {
    () => {
        vec![]
    };
}
