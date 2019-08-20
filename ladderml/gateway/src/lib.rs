#![cfg_attr(not(feature = "std"), no_std)]

use srml_support::{StorageValue, dispatch::Result, decl_module, decl_storage, decl_event, StorageMap, dispatch::Vec, ensure};
use system::ensure_signed;
use parity_codec::{Decode, Encode};
use srml_support::traits::Currency;
//use system::Module;

#[cfg(feature = "std")]
use runtime_io::with_storage;
#[cfg(feature = "std")]
use serde_derive::{Deserialize, Serialize};

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

#[derive(PartialEq, Eq, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
pub struct EnterInfo<Account, Balance> {
    pub receiver: Account,
    pub value: Balance,
}

pub trait Trait: system::Trait + balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Currency: Currency<Self::AccountId>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Gateway {
        ///
         Author get(author) config(): T::AccountId;
        ///
         TotalIncrease get(total_increase): BalanceOf<T>;
        ///
         TotalDecrease get(total_decrease): BalanceOf<T>;
        ///
         HashOf get(hash_of): map T::Hash => EnterInfo<T::AccountId, BalanceOf<T>>;
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

        pub fn enter(origin, hash: T::Hash, receiver: T::AccountId, value: BalanceOf<T>) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(sender == Self::author(), "Only author can call it");

            if (<HashOf<T>>::exists(hash)) {
                return Err("Repeat entry information");
            }

            // modify balance
            T::Currency::deposit_creating(&receiver, value);

            <TotalIncrease<T>>::mutate(|total| {*total = *total + value;});

            Self::deposit_event(RawEvent::Increase(hash, receiver, value));
            Ok(())
        }

        pub fn out(origin, receiver: Vec<u8>, value: BalanceOf<T>) -> Result {
            let sender = ensure_signed(origin)?;

            // modify balance


            ensure!(T::Currency::can_slash(&sender, value), "balance to low");

            T::Currency::slash(&sender, value);

            <TotalDecrease<T>>::mutate(|total| {
                *total = *total + value;
                *total
            });

            // dispach event
            Self::deposit_event(RawEvent::Decrease(sender, receiver, value));
            Ok(())
        }

//        /// update author
//        pub fn update_author(new: T:AccountId) {
//			<Author<T>>::put(new);
//		}
    }
}

impl<T: Trait> Module<T> {}


#[cfg(test)]
mod tests {
    use super::*;

    use srml_support::{impl_outer_origin, assert_ok, assert_err};
    use runtime_io::{Blake2Hasher, with_externalities};
    use primitives::{
        BuildStorage, traits::{BlakeTwo256, IdentityLookup},
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

    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        let mut t = system::GenesisConfig::<Test>::default().build_storage().unwrap().0;
        t.extend(GenesisConfig::<Test>{
            author: 1,
        }.build_storage().unwrap().0);
        t.into()
    }

    #[test]
    fn initialize_test() {
        with_externalities(&mut new_test_ext(), || {
            assert_eq!(Gateway::author(), 1);
        });
    }

    #[test]
    fn only_author_can_enter() {
        with_externalities(&mut new_test_ext(), || {
            assert_eq!(Gateway::author(), 1);
        });
    }

    #[test]
    fn repeat_enter_should_failed() {
        with_externalities(&mut new_test_ext(), || {
            assert_eq!(Gateway::author(), 1);
        });
    }

    #[test]
    fn only_avaliable_balance_can_go_out() {
        with_externalities(&mut new_test_ext(), || {
            assert_eq!(Gateway::author(), 1);
        });
    }

}