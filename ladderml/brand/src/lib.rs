//! A cross chain brand system.
//!
//! Label system store all chain of Renting the current platform.
//! Anyone can add label to current system. Once added to permanent storage.
//! 
//! future: 
//! only root key can add or delete brand. 
//! brands should link with bank.
//! once brand registered, vendor auto listen `anchor` contract from ETH or ABOS.
//! 

#![cfg_attr(not(feature = "std"), no_std)]

use srml_support::{StorageValue, dispatch::Result, decl_module, decl_storage, decl_event, StorageMap, dispatch::Vec};
use system::ensure_signed;
use parity_codec::{Decode, Encode};

#[cfg(feature = "std")]
use runtime_io::with_storage;
#[cfg(feature = "std")]
use serde_derive::{Deserialize, Serialize};

//TODO replace vec<u8>, explorer can't parse NameString.
pub type NameString = Vec<u8>;

const MAX_NAME_LEN: usize = 16;

#[derive(PartialEq, Eq, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
pub struct Trademark {
    /// The name of brand.
    pub name: NameString,
    /// The Id of brand.
    pub id: u32,
}

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Brand {
        /// The total of registered brand.
        BrandTotal get(brand_total) : u32 = 0;
        /// The index of name for id.
        IdOf get(id_of) : map Vec<u8> => u32;
        /// The index of id for name.
        NameOf get(name_of) : map u32 => Vec<u8>;
        //BrandList get(brand_list) : Vec<Trademark>; // required by `sr_api_hidden_includes_decl_storage::hidden_include::StorageValue::put`
    }
    add_extra_genesis {
        config(brands) : Vec<(Trademark, T::AccountId)>;

        build(|storage: &mut primitives::StorageOverlay, _: &mut primitives::ChildrenStorageOverlay, config: &GenesisConfig<T>| {
            with_storage(storage, || {
                config.brands.iter().for_each(|(brand, account)| {
                    <Module<T>>::register_brand(T::Origin::from(Some(account.clone()).into()), brand.name.clone()).unwrap();
                })
            })
        })
    }
}

decl_event!(
    pub enum Event<T> where <T as system::Trait>::AccountId {
        NewBrand(AccountId, Vec<u8>, u32),
    }
);

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;

        /// Register a new brand with name. the brand id added auto.
        pub fn register_brand(origin, name: Vec<u8>) -> Result {
            let sender = ensure_signed(origin)?;
            Self::is_valid_name(&name)?;

            if <IdOf<T>>::exists(&name) {
                return Err("brand already existed")
            }

            let id = Self::brand_total() + 1;
            <IdOf<T>>::insert(&name, id);
            <BrandTotal<T>>::put(id);
            <NameOf<T>>::insert(id, &name);

            Self::deposit_event(RawEvent::NewBrand(sender, name, id));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_valid_name(v: &[u8]) -> Result {
        if v.len() > MAX_NAME_LEN || v.len() == 0 {
            Err("trademark length is too long or zero")
        } else {
            for c in v.iter() {
                // allow number (0x30~0x39), capital letter (0x41~0x5A), small letter (0x61~0x7A), - 0x2D
                if (*c >= 0x30 && *c <= 0x39) // number
                    || (*c >= 0x41 && *c <= 0x5A) // capital
                    || (*c >= 0x61 && *c <= 0x7A) // small
                    || (*c == 0x2D) // -
                // ~
                {
                    continue;
                } else {
                    return Err("invalid trademark char for number, capital/small letter or '-'");
                }
            }
            Ok(())
        }
    }
}

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

    impl Trait for Test {
        type Event = ();
    }

    type Brand = Module<Test>;

    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        let mut t = system::GenesisConfig::<Test>::default().build_storage().unwrap().0;
        t.extend(GenesisConfig::<Test>{
            brands: vec![(Trademark{ name: b"ETH".to_vec(), id: 1 }, 1),
                         (Trademark{ name: b"ABOS".to_vec(), id: 2 }, 1)],
        }.build_storage().unwrap().0);
        t.into()
    }

    #[test]
    fn initialize_test() {
        with_externalities(&mut new_test_ext(), || {
            assert_eq!(Brand::brand_total(), 2);
            assert_eq!(Brand::name_of(1), b"ETH".to_vec());
            assert_eq!(Brand::name_of(2), b"ABOS".to_vec());
            assert_eq!(Brand::name_of(3), b"".to_vec());

            assert_eq!(Brand::id_of(b"".to_vec()), 0);
        });
    }

    #[test]
    fn register_test() {
        with_externalities(&mut new_test_ext(), || {
            assert_ok!(Brand::register_brand(Origin::signed(1), b"EOS".to_vec()));
            assert_eq!(Brand::brand_total(), 3);

            assert_err!(Brand::register_brand(Origin::signed(2), b"EOS".to_vec()), "brand already existed");
            assert_eq!(Brand::brand_total(), 3);
        });
    }
}