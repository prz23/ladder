#![cfg_attr(not(feature = "std"), no_std)]

use srml_support::{StorageValue, dispatch::Result, decl_module, decl_storage, decl_event,
                   StorageMap, dispatch::Vec, ensure};
use system::ensure_signed;
use srml_support::traits::{Currency, WithdrawReason, ExistenceRequirement};


type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub trait Trait: system::Trait + balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Currency: Currency<Self::AccountId>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Gateway {
        /// The gateway controller
        pub Author get(author) config(): T::AccountId;
        /// The total increase in gateway
        pub TotalIncrease get(total_increase): BalanceOf<T>;
        /// The total decrease in gateway
        pub TotalDecrease get(total_decrease): BalanceOf<T>;
        /// Record entry hash for deduplication
        pub HashOf get(hash_of): map T::Hash => Option<(T::AccountId, BalanceOf<T>)>;
        /// Record all outgoing from the gateway
        pub AccountOf get(account_of): map T::AccountId => Vec<(Vec<u8>, BalanceOf<T>)>;
        /// The value of maximum exit in single action.
        pub MaximumExit get(maximum_exit) config(): BalanceOf<T>;
        /// The value of minimum exit in single action.
        pub MinimumExit get(minimum_exit) config(): BalanceOf<T>;
    }
}

decl_event!(
    pub enum Event<T> where <T as system::Trait>::Hash,
    Balance = BalanceOf<T>,
    <T as system::Trait>::AccountId,
    {
        Increase(Hash, AccountId, Balance),
        Decrease(AccountId, Vec<u8>, Balance),
    }
);

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;

        /// Map Ethereum funds to current system
		///
		/// Only the authority can successfully call.
        pub fn enter(origin, hash: T::Hash, receiver: T::AccountId, value: BalanceOf<T>) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(sender == Self::author(), "only author can call it");

            if <HashOf<T>>::exists(hash) {
                return Err("repeat entry");
            }
            <HashOf<T>>::insert(hash, (receiver.clone(), value.clone()));

            // modify balance
            T::Currency::deposit_creating(&receiver, value);

            <TotalIncrease<T>>::mutate(|total| {*total = *total + value; });

            Self::deposit_event(RawEvent::Increase(hash, receiver, value));
            Ok(())
        }

        /// Map current system funds to Ethereum
		///
		/// If the funds in the account are locked, the call will fail, for example in a mortgage.
        pub fn out(origin, receiver: Vec<u8>, value: BalanceOf<T>) -> Result {
            let sender = ensure_signed(origin)?;

            // Another meaning, the maximum value is used to control whether to open the channel.
            ensure!(value <= Self::maximum_exit(), "exceeding the maximum output");

            ensure!(value >= Self::minimum_exit(), "below the minimum output");

            T::Currency::withdraw(&sender, value, WithdrawReason::Transfer, ExistenceRequirement::KeepAlive)?;

            <TotalDecrease<T>>::mutate(|total| { *total = *total + value; });
            <AccountOf<T>>::mutate(&sender, |q| q.push((receiver.clone(), value.clone())));

            // dispach event
            Self::deposit_event(RawEvent::Decrease(sender, receiver, value));
            Ok(())
        }

        /// update author
        pub fn update_author(new: T::AccountId) {
			<Author<T>>::put(new);
		}

        /// update the value of maximum exit.
		pub fn update_maximum_exit(new: BalanceOf<T>) {
		    <MaximumExit<T>>::put(new);
		}

        /// update the value of minimum exit.
		pub fn update_minimum_exit(new: BalanceOf<T>) {
		    <MinimumExit<T>>::put(new);
		}
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use srml_support::{impl_outer_origin, assert_ok, assert_err,
                    traits::{LockableCurrency, LockIdentifier, WithdrawReason, WithdrawReasons}};
    use runtime_io::{Blake2Hasher, with_externalities};
    use primitives::{
        BuildStorage, traits::{BlakeTwo256, IdentityLookup, Hash},
        testing::{H256, Digest, DigestItem, Header}
    };

    impl_outer_origin!{
        pub enum Origin for Test {}
    }

    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;

    impl system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Digest = Digest;
        type AccountId = u64;
        type Lookup = IdentityLookup<u64>;
        type Header = Header;
        type Event = ();
        type Log = DigestItem;
    }

    impl balances::Trait for Test {
        type Balance = u64;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
        type TransactionPayment = ();
        type TransferPayment = ();
        type DustRemoval = ();
    }

    impl Trait for Test {
        type Event = ();
        type Currency = balances::Module<Self>;
    }

    type Gateway = Module<Test>;
    type Balances = balances::Module<Test>;

    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        let mut t = system::GenesisConfig::<Test>::default().build_storage().unwrap().0;
        t.extend(GenesisConfig::<Test>{
            author: 1,
            maximum_exit: 1000,
            minimum_exit: 10,
        }.build_storage().unwrap().0);
        t.into()
    }

    const ID_1: LockIdentifier = *b"1       ";

    #[test]
    fn initialize_test() {
        with_externalities(&mut new_test_ext(), || {
            assert_eq!(Gateway::author(), 1);

            // author enter
            assert_ok!(Gateway::enter(Origin::signed(1), BlakeTwo256::hash(&[1]), 2, 1000));
            assert_eq!(Balances::free_balance(&2), 1000);

            // 2 withdraw
            assert_ok!(Gateway::out(Origin::signed(2),[1u8, 2u8].to_vec(), 1000));
            assert_eq!(Balances::free_balance(&2), 0);

            assert_eq!(Gateway::total_increase(), 1000);
            assert_eq!(Gateway::total_decrease(), 1000);

            // author enter
            assert_ok!(Gateway::enter(Origin::signed(1), BlakeTwo256::hash(&[2]), 2, 1001));
            assert_eq!(Balances::free_balance(&2), 1001);

            // withdraw fail, exceeding maximum
            assert_err!(Gateway::out(Origin::signed(2),[1u8, 2u8].to_vec(), 1001), "exceeding the maximum output");
            assert_eq!(Balances::free_balance(&2), 1001);

            // withdraw fail, below minimum
            assert_err!(Gateway::out(Origin::signed(2),[1u8, 2u8].to_vec(), 1), "below the minimum output");
            assert_eq!(Balances::free_balance(&2), 1001);
        });
    }

    #[test]
    fn only_author_can_enter() {
        with_externalities(&mut new_test_ext(), || {
            assert_eq!(Gateway::author(), 1);

            // author enter
            assert_ok!(Gateway::enter(Origin::signed(1), BlakeTwo256::hash(&[1]), 2, 1000));
            assert_eq!(Balances::free_balance(&2), 1000);

            // author enter
            assert_ok!(Gateway::enter(Origin::signed(1), BlakeTwo256::hash(&[2]), 3, 1000));
            assert_eq!(Balances::free_balance(&3), 1000);

            // not author enter
            assert_err!(Gateway::enter(Origin::signed(2), BlakeTwo256::hash(&[3]), 2, 1000), "only author can call it");
            assert_eq!(Balances::free_balance(&2), 1000);

            assert_eq!(Gateway::total_increase(), 2000);
        });
    }

    #[test]
    fn repeat_enter_should_failed() {
        with_externalities(&mut new_test_ext(), || {
            assert_eq!(Gateway::author(), 1);

            // author enter
            assert_ok!(Gateway::enter(Origin::signed(1), BlakeTwo256::hash(&[1]), 2, 1000));
            assert_eq!(Balances::free_balance(&2), 1000);

            // repeat hash
            assert_err!(Gateway::enter(Origin::signed(1), BlakeTwo256::hash(&[1]), 2, 1000), "repeat entry");
            assert_eq!(Balances::free_balance(&2), 1000);

            assert_eq!(Gateway::total_increase(), 1000);
        });
    }

    #[test]
    fn only_avaliable_balance_can_go_out() {
        with_externalities(&mut new_test_ext(), || {
            assert_eq!(Gateway::author(), 1);

            // author enter
            assert_ok!(Gateway::enter(Origin::signed(1), BlakeTwo256::hash(&[1]), 2, 1000));
            assert_eq!(Balances::free_balance(&2), 1000);

            // lock balance
            Balances::set_lock(ID_1, &2, 500, u64::max_value(), WithdrawReasons::all());

            // 2 withdraw failed
            assert_err!(Gateway::out(Origin::signed(2),[1u8, 2u8].to_vec(), 1000), "account liquidity restrictions prevent withdrawal");
            assert_eq!(Balances::free_balance(&2), 1000);

            // 2 withdraw
            assert_ok!(Gateway::out(Origin::signed(2),[1u8, 2u8].to_vec(), 400));
            assert_eq!(Balances::free_balance(&2), 600);


            // unlock balance
            Balances::remove_lock(ID_1, &2);
            assert_ok!(Gateway::out(Origin::signed(2),[1u8, 2u8].to_vec(), 600));

            assert_eq!(Gateway::total_increase(), 1000);
            assert_eq!(Gateway::total_decrease(), 1000);

        });
    }
}