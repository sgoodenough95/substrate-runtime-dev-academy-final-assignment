use super::*;

use crate as kitties;
use std::cell::RefCell;
use sp_core::H256;
use frame_support::{
    parameter_types, assert_ok, assert_noop, error::BadOrigin, unsigned::ValidateUnsigned,
};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup}, testing::Header,
	testing::TestXt,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
		NFT: orml_nft::{Module, Storage},
		KittiesModule: kitties::{Module, Call, Storage, Event<T>, Config},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}
impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

impl orml_nft::Config for Test {
    type ClassId = u32;
	type TokenId = u32;
	type ClassData = ();
	type TokenData = Kitty;
}

thread_local! {
    static RANDOM_PAYLOAD: RefCell<H256> = RefCell::new(Default::default());
}

pub struct MockRandom;

impl Randomness<H256> for MockRandom {
    fn random(_subject: &[u8]) -> H256 {
        RANDOM_PAYLOAD.with(|v| *v.borrow())
    }
}

fn set_random(val: H256) {
    RANDOM_PAYLOAD.with(|v| *v.borrow_mut() = val)
}

parameter_types! {
	pub const DefaultDifficulty: u32 = 3;
}

impl Config for Test {
    type Event = Event;
    type Randomness = MockRandom;
    type Currency = Balances;
    type WeightInfo = ();
    type DefaultDifficulty = DefaultDifficulty;
}

/// An extrinsic type used for tests.
pub type Extrinsic = TestXt<Call, ()>;

impl<LocalCall> SendTransactionTypes<LocalCall> for Test
where
	Call: From<LocalCall>,
{
	type OverarchingCall = Call;
	type Extrinsic = Extrinsic;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

    pallet_balances::GenesisConfig::<Test>{
		balances: vec![(200, 500)],
    }.assimilate_storage(&mut t).unwrap();

    crate::GenesisConfig::default().assimilate_storage::<Test>(&mut t).unwrap();

    let mut t: sp_io::TestExternalities = t.into();

    t.execute_with(|| System::set_block_number(1) );
    t
}

fn last_event() -> Event {
    System::events().last().unwrap().event.clone()
}

#[test]
fn can_create() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create(Origin::signed(100)));

        let kitty = Kitty([59, 250, 138, 82, 209, 39, 141, 109, 163, 238, 183, 145, 235, 168, 18, 122]);

        assert_eq!(KittiesModule::kitties(&100, 0), Some(kitty.clone()));
        assert_eq!(NFT::tokens(KittiesModule::class_id(), 0).unwrap().owner, 100);

        assert_eq!(last_event(), Event::kitties(crate::Event::<Test>::KittyCreated(100, 0, kitty)));
    });
}

#[test]
fn gender() {
    assert_eq!(Kitty([0; 16]).gender(), KittyGender::Male);
    assert_eq!(Kitty([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).gender(), KittyGender::Female);
}

#[test]
fn can_breed() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create(Origin::signed(100)));

        set_random(H256::from([2; 32]));

        assert_ok!(KittiesModule::create(Origin::signed(100)));

        assert_noop!(KittiesModule::breed(Origin::signed(100), 0, 11), Error::<Test>::InvalidKittyId);
        assert_noop!(KittiesModule::breed(Origin::signed(100), 0, 0), Error::<Test>::SameGender);
        assert_noop!(KittiesModule::breed(Origin::signed(101), 0, 1), Error::<Test>::InvalidKittyId);

        assert_ok!(KittiesModule::breed(Origin::signed(100), 0, 1));

        let kitty = Kitty([187, 250, 235, 118, 211, 247, 237, 253, 187, 239, 191, 185, 239, 171, 211, 122]);

        assert_eq!(KittiesModule::kitties(&100, 2), Some(kitty.clone()));
        assert_eq!(NFT::tokens(KittiesModule::class_id(), 2).unwrap().owner, 100);

        assert_eq!(last_event(), Event::kitties(crate::Event::<Test>::KittyBred(100u64, 2u32, kitty)));
    });
}

#[test]
fn can_transfer() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create(Origin::signed(100)));
        assert_ok!(KittiesModule::set_price(Origin::signed(100), 0, Some(10)));

        assert_noop!(KittiesModule::transfer(Origin::signed(101), 200, 0), orml_nft::Error::<Test>::NoPermission);

        assert_ok!(KittiesModule::transfer(Origin::signed(100), 200, 0));

        assert_eq!(NFT::tokens(KittiesModule::class_id(), 0).unwrap().owner, 200);
        assert_eq!(KittyPrices::<Test>::contains_key(0), false);

        assert_eq!(last_event(), Event::kitties(RawEvent::KittyTransferred(100, 200, 0)));
    });
}

#[test]
fn handle_self_transfer() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create(Origin::signed(100)));

        System::reset_events();

        assert_noop!(KittiesModule::transfer(Origin::signed(100), 100, 1), orml_nft::Error::<Test>::TokenNotFound);

        assert_ok!(KittiesModule::transfer(Origin::signed(100), 100, 0));

        assert_eq!(NFT::tokens(KittiesModule::class_id(), 0).unwrap().owner, 100);

        // no transfer event because no actual transfer is executed
        assert_eq!(System::events().len(), 0);
    });
}

#[test]
fn can_set_price() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create(Origin::signed(100)));

        assert_noop!(KittiesModule::set_price(Origin::signed(200), 0, Some(10)), Error::<Test>::NotOwner);

        assert_ok!(KittiesModule::set_price(Origin::signed(100), 0, Some(10)));

        assert_eq!(last_event(), Event::kitties(RawEvent::KittyPriceUpdated(100, 0, Some(10))));

        assert_eq!(KittiesModule::kitty_prices(0), Some(10));

        assert_ok!(KittiesModule::set_price(Origin::signed(100), 0, None));
        assert_eq!(KittyPrices::<Test>::contains_key(0), false);

        assert_eq!(last_event(), Event::kitties(RawEvent::KittyPriceUpdated(100, 0, None)));
    });
}

#[test]
fn can_buy() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create(Origin::signed(100)));

        assert_noop!(KittiesModule::buy(Origin::signed(100), 100, 0, 10), Error::<Test>::BuyFromSelf);
        assert_noop!(KittiesModule::buy(Origin::signed(200), 100, 1, 10), Error::<Test>::NotForSale);
        assert_noop!(KittiesModule::buy(Origin::signed(200), 100, 0, 10), Error::<Test>::NotForSale);

        assert_ok!(KittiesModule::set_price(Origin::signed(100), 0, Some(600)));

        assert_noop!(KittiesModule::buy(Origin::signed(200), 100, 0, 500), Error::<Test>::PriceTooLow);

        assert_noop!(KittiesModule::buy(Origin::signed(200), 100, 0, 600), pallet_balances::Error::<Test, _>::InsufficientBalance);

        assert_ok!(KittiesModule::set_price(Origin::signed(100), 0, Some(400)));

        assert_ok!(KittiesModule::buy(Origin::signed(200), 100, 0, 500));

        assert_eq!(KittyPrices::<Test>::contains_key(0), false);
        assert_eq!(NFT::tokens(KittiesModule::class_id(), 0).unwrap().owner, 200);
        assert_eq!(Balances::free_balance(100), 400);
        assert_eq!(Balances::free_balance(200), 100);

        assert_eq!(last_event(), Event::kitties(RawEvent::KittySold(100, 200, 0, 400)));
    });
}

#[test]
fn can_auto_breed() {
    new_test_ext().execute_with(|| {
        // nonce and solution are not checked by auto_breed directly

        assert_ok!(KittiesModule::create(Origin::signed(100)));
        assert_ok!(KittiesModule::create(Origin::signed(101)));

        assert_noop!(KittiesModule::auto_breed(Origin::none(), 0, 2, 0, 0), Error::<Test>::InvalidKittyId);
        assert_noop!(KittiesModule::auto_breed(Origin::none(), 0, 0, 0, 0), Error::<Test>::SameGender);
        assert_noop!(KittiesModule::auto_breed(Origin::signed(100), 0, 1, 0, 0), BadOrigin);

        assert_ok!(KittiesModule::auto_breed(Origin::none(), 0, 1, 0, 0));

        let kitty = Kitty([34, 170, 2, 80, 145, 37, 4, 36, 35, 32, 179, 144, 169, 40, 2, 18]);

        assert_eq!(KittiesModule::kitties(&100, 2), Some(kitty.clone()));
        assert_eq!(NFT::tokens(KittiesModule::class_id(), 2).unwrap().owner, 100);

        assert_eq!(last_event(), Event::kitties(RawEvent::KittyBred(100, 2, kitty)));
    });
}

#[test]
fn can_validate_unsigned() {
    new_test_ext().execute_with(|| {
        // only check nonce and solution are valid

        assert_eq!(KittiesModule::validate_unsigned(TransactionSource::InBlock, &crate::Call::auto_breed(0, 1, 0, 0)), InvalidTransaction::BadProof.into());
        assert_eq!(KittiesModule::validate_unsigned(TransactionSource::InBlock, &crate::Call::auto_breed(0, 1, 0, 1)), InvalidTransaction::BadProof.into());
        assert_eq!(KittiesModule::validate_unsigned(TransactionSource::InBlock, &crate::Call::auto_breed(0, 1, 0, 2)), TransactionValidity::Ok(ValidTransaction {
            priority: 0,
            requires: vec![],
            provides: vec![],
            longevity: 64,
            propagate: true,
        }));

        assert_eq!(KittiesModule::auto_breed_nonce(), 1);

        assert_eq!(KittiesModule::validate_unsigned(TransactionSource::InBlock, &crate::Call::auto_breed(0, 1, 0, 2)), InvalidTransaction::BadProof.into());

        assert_eq!(KittiesModule::validate_unsigned(TransactionSource::InBlock, &crate::Call::auto_breed(0, 1, 1, 11)), InvalidTransaction::BadProof.into());
        assert_eq!(KittiesModule::validate_unsigned(TransactionSource::InBlock, &crate::Call::auto_breed(0, 1, 1, 12)), InvalidTransaction::BadProof.into());
        assert_eq!(KittiesModule::validate_unsigned(TransactionSource::InBlock, &crate::Call::auto_breed(0, 1, 1, 13)), TransactionValidity::Ok(ValidTransaction {
            priority: 0,
            requires: vec![],
            provides: vec![],
            longevity: 64,
            propagate: true,
        }));

        assert_eq!(KittiesModule::auto_breed_nonce(), 2);
    });
}
