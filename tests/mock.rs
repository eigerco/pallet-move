use frame_support::traits::{ConstU128, ConstU16, ConstU32, ConstU64};
use pallet_move::Config;
use sp_core::H256;
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

pub const EXISTENTIAL_DEPOSIT: u128 = 500;
pub type Balance = u128;

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

impl Config for Test {
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
#[allow(dead_code)]
pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(0));
    ext
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

#[cfg(test)]
#[allow(dead_code)]
/// function to check last event from runtime.
/// for test purposes only!
pub(crate) fn assert_last_event(generic_event: RuntimeEvent) {
    let events = frame_system::Pallet::<Test>::events();
    let system_event: <Test as frame_system::Config>::RuntimeEvent = generic_event;
    // compare to the last event record
    let frame_system::EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}
