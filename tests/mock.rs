use frame_support::traits::{ConstU128, ConstU16, ConstU32, ConstU64};
use move_core_types::account_address::AccountAddress;
use sp_core::{crypto::Ss58Codec, sr25519::Public, H256};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    AccountId32, BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

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

impl pallet_move::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type Currency = Balances;
}

/// Test Externalities Builder for an easier test setup.
#[allow(dead_code)]
#[derive(Default)]
pub(crate) struct ExtBuilder {
    /// Overwrite default accounts with balances.
    balances: Vec<(AccountId32, Balance)>,
    /// Overwrite default Move-stdlib setup.
    move_stdlib: Option<Vec<u8>>,
    /// Overwrite default Substrate-stdlib.
    substrate_stdlib: Option<Vec<u8>>,
}

#[allow(dead_code)]
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

// Build genesis storage according to the mock runtime.
#[allow(dead_code)]
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    let pallet_move_config = pallet_move::GenesisConfig::<Test> {
        _phantom: core::marker::PhantomData,
        change_default_move_stdlib_bundle_to: None,
        change_default_substrate_stdlib_bundle_to: None,
    };

    pallet_move_config.assimilate_storage(&mut storage).unwrap();

    storage.into()
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        MoveModule: pallet_move,
    }
);

// Common constants accross the tests.
#[allow(dead_code)]
pub const EMPTY_CHEQUE: u128 = 0; // Not all scripts need the `cheque_amount` parameter.
pub const CAFE_ADDR: &str = "0xCAFE";
pub const BOB_ADDR: &str = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty";
pub const ALICE_ADDR: &str = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
pub const DAVE_ADDR: &str = "5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy";
lazy_static::lazy_static! {
    pub static ref CAFE_ADDR_MOVE: AccountAddress = {
        AccountAddress::from_hex_literal(CAFE_ADDR).unwrap()
    };
    pub static ref CAFE_ADDR_NATIVE: AccountId32 = {
        MoveModule::to_native_account(&CAFE_ADDR_MOVE).unwrap()
    };
    pub static ref BOB_ADDR_NATIVE: AccountId32 = {
        let (pk, _) = Public::from_ss58check_with_version(BOB_ADDR).unwrap();
        pk.into()
    };
    pub static ref BOB_ADDR_MOVE: AccountAddress = {
        MoveModule::to_move_address(&BOB_ADDR_NATIVE).unwrap()
    };
    pub static ref ALICE_ADDR_NATIVE: AccountId32 = {
        let (pk, _) = Public::from_ss58check_with_version(ALICE_ADDR).unwrap();
        pk.into()
    };
    pub static ref ALICE_ADDR_MOVE: AccountAddress = {
        MoveModule::to_move_address(&ALICE_ADDR_NATIVE).unwrap()
    };
    pub static ref DAVE_ADDR_NATIVE: AccountId32 = {
        let (pk, _) = Public::from_ss58check_with_version(DAVE_ADDR).unwrap();
        pk.into()
    };
    pub static ref DAVE_ADDR_MOVE: AccountAddress = {
        MoveModule::to_move_address(&DAVE_ADDR_NATIVE).unwrap()
    };
}

#[allow(dead_code)]
pub fn addr32_from_ss58(ss58addr: &str) -> AccountId32 {
    let (pk, _) = Public::from_ss58check_with_version(ss58addr).unwrap();
    pk.into()
}

#[allow(dead_code)]
pub fn addr32_to_move(addr32: &AccountId32) -> Result<AccountAddress, pallet_move::Error<Test>> {
    MoveModule::to_move_address(addr32)
}

#[allow(dead_code)]
pub fn addrs_from_ss58(
    ss58: &str,
) -> Result<(AccountId32, AccountAddress), pallet_move::Error<Test>> {
    let addr_32 = addr32_from_ss58(ss58);
    let addr_mv = addr32_to_move(&addr_32)?;
    Ok((addr_32, addr_mv))
}
