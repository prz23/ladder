//! The Substrate runtime. This can be compiled with ``#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc))]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit="256"]

pub mod matrix;
pub mod sigcount;
pub mod bank;

#[cfg(feature = "std")]
use serde_derive::{Serialize, Deserialize};
use parity_codec::{Encode, Decode};
use rstd::prelude::*;
#[cfg(feature = "std")]
use primitives::bytes;
use primitives::{ed25519, OpaqueMetadata};
use support::{construct_runtime};
use primitives::u32_trait::{_2, _4};
use grandpa::fg_primitives::{self, ScheduledChange};
use client::{
	block_builder::api::{self as block_builder_api, InherentData, CheckInherentsResult},
	runtime_api as client_api, impl_runtime_apis, decl_runtime_apis,
};
use runtime_primitives::{ApplyResult, generic, create_runtime_str};
use runtime_primitives::transaction_validity::TransactionValidity;
use runtime_primitives::traits::{ 
	self, BlakeTwo256, Block as BlockT, DigestFor,
    Verify, NumberFor, StaticLookup,
};
use version::RuntimeVersion;
use council::{motions as council_motions, voting as council_voting};
#[cfg(feature = "std")]
use council::seats as council_seats;
#[cfg(any(feature = "std", test))]
use version::NativeVersion;

#[cfg(any(feature = "std", test))]
pub use runtime_primitives::BuildStorage;
pub use consensus::Call as ConsensusCall;
pub use timestamp::Call as TimestampCall;
pub use balances::Call as BalancesCall;
pub use matrix::Call as MatrixCall;
pub use sigcount::Call as SigCall;
pub use bank::Call as BankCall;
pub use runtime_primitives::{Permill, Perbill};
pub use support::StorageValue;
pub use staking::StakerStatus;
pub use system::EventRecord;

/// The type that is used for identifying authorities.
pub type AuthorityId = <AuthoritySignature as Verify>::Signer;

/// The type used by authorities to prove their ID.
pub type AuthoritySignature = ed25519::Signature;

/// Alias to pubkey that identifies an account on the chain.
pub type AccountId = <AccountSignature as Verify>::Signer;

/// The type used by authorities to prove their ID.
pub type AccountSignature = ed25519::Signature;

/// A hash of some data used by the chain.
pub type Hash = primitives::H256;

/// Index of a block number in the chain.
pub type BlockNumber = u64;

/// Index of an account's extrinsic in the chain.
pub type Nonce = u64;

/// Index of a transaction in the chain.
pub type Index = u64;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core datastructures.
pub mod opaque {
	use super::*;

	/// Opaque, encoded, unchecked extrinsic.
	#[derive(PartialEq, Eq, Clone, Default, Encode, Decode)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
	pub struct UncheckedExtrinsic(#[cfg_attr(feature = "std", serde(with="bytes"))] pub Vec<u8>);
	impl traits::Extrinsic for UncheckedExtrinsic {
		fn is_signed(&self) -> Option<bool> {
			None
		}
	}
	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256, generic::DigestItem<Hash, AuthorityId, AuthoritySignature>>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
	/// Opaque session key type.
	pub type SessionKey = AuthorityId;
}

/// Runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("abmatrix"),
	impl_name: create_runtime_str!("abmatrix-node"),
	authoring_version: 10,
	spec_version: 16,
	impl_version: 16,
	apis: RUNTIME_API_VERSIONS,
};

/// Native version.
#[cfg(any(feature = "std", test))]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

impl system::Trait for Runtime {
	type Origin = Origin;
	type Index = Index;
	type BlockNumber = BlockNumber;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type Digest = generic::Digest<Log>;
	type AccountId = AccountId;
	type Lookup = Indices;
	type Header = generic::Header<BlockNumber, BlakeTwo256, Log>;
	type Event = Event;
	type Log = Log;
}

impl aura::Trait for Runtime {
	type HandleReport = aura::StakingSlasher<Runtime>;
}

impl indices::Trait for Runtime {
	type AccountIndex = AccountIndex;
	type IsDeadAccount = Balances;
	type ResolveHint = indices::SimpleResolveHint<Self::AccountId, Self::AccountIndex>;
	type Event = Event;
}

impl balances::Trait for Runtime {
	type Balance = Balance;
	type OnFreeBalanceZero = ((Staking, Contract), Session);
	type OnNewAccount = Indices;
	type Event = Event;
}

impl fees::Trait for Runtime {
	type Event = Event;
	type TransferAsset = Balances;
}

impl consensus::Trait for Runtime {
	type Log = Log;
	type SessionKey = AuthorityId;

	// The Aura module handles offline-reports internally
	// rather than using an explicit report system.
	type InherentOfflineReport = ();
}

impl timestamp::Trait for Runtime {
	type Moment = u64;
	type OnTimestampSet = Aura;
}

impl session::Trait for Runtime {
	type ConvertAccountIdToSessionKey = ();
	type OnSessionChange = (Staking, grandpa::SyncedAuthorities<Runtime>);
	type Event = Event;
}

impl staking::Trait for Runtime {
	type Currency = balances::Module<Self>;
	type OnRewardMinted = Treasury;
	type Event = Event;
}

impl democracy::Trait for Runtime {
	type Currency = balances::Module<Self>;
	type Proposal = Call;
	type Event = Event;
}

impl council::Trait for Runtime {
	type Event = Event;
}

impl council::voting::Trait for Runtime {
	type Event = Event;
}

impl council::motions::Trait for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
}

impl treasury::Trait for Runtime {
	type Currency = balances::Module<Self>;
	type ApproveOrigin = council_motions::EnsureMembers<_4>;
	type RejectOrigin = council_motions::EnsureMembers<_2>;
	type Event = Event;
}

impl contract::Trait for Runtime {
	type Call = Call;
	type Event = Event;
	type Gas = u64;
	type DetermineContractAddress = contract::SimpleAddressDeterminator<Runtime>;
	type ComputeDispatchFee = contract::DefaultDispatchFeeComputor<Runtime>;
}

impl sudo::Trait for Runtime {
	type Event = Event;
	type Proposal = Call;
}

impl grandpa::Trait for Runtime {
	type SessionKey = AuthorityId;
	type Log = Log;
	type Event = Event;
}

impl finality_tracker::Trait for Runtime {
	type OnFinalizationStalled = grandpa::SyncedAuthorities<Runtime>;
}

impl matrix::Trait for Runtime {
    type Event = Event;
}

impl bank::Trait for Runtime {
	type Event = Event;
}

impl sigcount::Trait for Runtime {
	type Event = Event;
}

construct_runtime!(
	pub enum Runtime with Log(InternalLog: DigestItem<Hash, AuthorityId, AuthoritySignature>) where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: system::{default, Log(ChangesTrieRoot)},
		Aura: aura::{Module, Inherent(Timestamp)},
		Timestamp: timestamp::{Module, Call, Storage, Config<T>, Inherent},
		Consensus: consensus::{Module, Call, Storage, Config<T>, Log(AuthoritiesChange), Inherent},
		Indices: indices,
		Balances: balances,
		Session: session,
		Staking: staking::{default, OfflineWorker},
		Democracy: democracy,
		Council: council::{Module, Call, Storage, Event<T>},
		CouncilVoting: council_voting,
		CouncilMotions: council_motions::{Module, Call, Storage, Event<T>, Origin},
		CouncilSeats: council_seats::{Config<T>},
		FinalityTracker: finality_tracker::{Module, Call, Inherent},
		Grandpa: grandpa::{Module, Call, Storage, Config<T>, Log(), Event<T>},
		Treasury: treasury,
		Contract: contract::{Module, Call, Storage, Config<T>, Event<T>},
		Sudo: sudo,
		Fees: fees::{Module, Storage, Config<T>, Event<T>},
		Matrix: matrix::{Module, Call, Storage, Event<T>},
		Sigcount: sigcount::{Module, Call, Storage, Event<T>},
		Bank: bank::{Module, Call, Storage, Event<T>},
	}
);

/// The address format for describing accounts.
pub type Address = <Indices as StaticLookup>::Source;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256, Log>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedMortalCompactExtrinsic<Address, Index, Call, AccountSignature>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Index, Call>;
/// Executive: handles dispatch to the various modules.
pub type Executive = executive::Executive<Runtime, Block, system::ChainContext<Runtime>, Fees, AllModules>;

decl_runtime_apis! {
	pub trait VendorApi {
		fn account_nonce(account: AccountId) -> Index;
		fn is_authority(id: &AuthorityId) -> bool;
	}
}

impl_runtime_apis! {
	impl client_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn authorities() -> Vec<AuthorityId> {
			Consensus::authorities()
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialise_block(header: &<Block as BlockT>::Header) {
			Executive::initialise_block(header)
		}
	}

	impl client_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			Runtime::metadata().into()
		}
	}

	impl block_builder_api::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalise_block() -> <Block as BlockT>::Header {
			Executive::finalise_block()
		}

		fn inherent_extrinsics(data: InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(block: Block, data: InherentData) -> CheckInherentsResult {
			data.check_extrinsics(&block)
		}

		fn random_seed() -> <Block as BlockT>::Hash {
			System::random_seed()
		}
	}

	impl client_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
			Executive::validate_transaction(tx)
		}
	}

	impl fg_primitives::GrandpaApi<Block> for Runtime {
		fn grandpa_pending_change(digest: &DigestFor<Block>)
			-> Option<ScheduledChange<NumberFor<Block>>>
		{
			for log in digest.logs.iter().filter_map(|l| match l {
				Log(InternalLog::grandpa(grandpa_signal)) => Some(grandpa_signal),
				_ => None
			}) {
				if let Some(change) = Grandpa::scrape_digest_change(log) {
					return Some(change);
				}
			}
			None
		}

		fn grandpa_forced_change(digest: &DigestFor<Block>)
			-> Option<(NumberFor<Block>, ScheduledChange<NumberFor<Block>>)>
		{
			for log in digest.logs.iter().filter_map(|l| match l {
				Log(InternalLog::grandpa(grandpa_signal)) => Some(grandpa_signal),
				_ => None
			}) {
				if let Some(change) = Grandpa::scrape_digest_forced_change(log) {
					return Some(change);
				}
			}
			None
		}

		fn grandpa_authorities() -> Vec<(AuthorityId, u64)> {
			Grandpa::grandpa_authorities()
		}
	}

	impl consensus_aura::AuraApi<Block> for Runtime {
		fn slot_duration() -> u64 {
			Aura::slot_duration()
		}
	}

	impl self::VendorApi<Block> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(&account)
		}

		fn is_authority(id: &AuthorityId) -> bool {
			Consensus::authorities().contains(id)
		}
	}
}