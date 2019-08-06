// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Substrate chain configurations.

use hex_literal::hex;
use node_primitives::AccountId;
pub use node_runtime::GenesisConfig;
use node_runtime::{
    BalancesConfig, BankConfig, ConsensusConfig, ContractConfig, CouncilSeatsConfig,
    CouncilVotingConfig, DemocracyConfig, GrandpaConfig, IndicesConfig, Perbill, Permill,
    SessionConfig, StakerStatus, StakingConfig, SudoConfig, TimestampConfig, TreasuryConfig,
    Trademark, BrandConfig, SigncheckConfig,ErcConfig,OtcConfig
};
use primitives::{
    crypto::{UncheckedInto, UncheckedFrom},
    ed25519,
    ed25519::Public as AuthorityId,
    sr25519, Pair,
};
use substrate_service;

/// Specialized `ChainSpec`.
pub type ChainSpec = substrate_service::ChainSpec<GenesisConfig>;

/// Helper function to generate AccountId from seed
pub fn get_account_id_from_seed(seed: &str) -> AccountId {
    sr25519::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Helper function to generate AuthorityId from seed
pub fn get_session_key_from_seed(seed: &str) -> AuthorityId {
    ed25519::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

fn str_to_vecu8(string:&str) -> Vec<u8>{
	let vecu8 : Vec<u8> = string.into();
	vecu8
}

pub fn get_nodeinfo(seed: &str) -> Vec<u8> {
	str_to_vecu8(seed)
}

/// Helper function to generate stash, controller and session key from seed
pub fn get_authority_keys_from_seed(seed: &str) -> (AccountId, AccountId, AuthorityId) {
    (
        get_account_id_from_seed(&format!("{}//stash", seed)),
        get_account_id_from_seed(seed),
        get_session_key_from_seed(seed),
    )
}

/// Helper function to generate exchange
pub fn get_exchangelad() -> Vec<(u64,u64)>{
   vec![
	   (1,1000),
	   (2,50000),
   ]
}

/// Helper function to generate brands.
pub fn get_brands(owned: AccountId) -> Vec<(Trademark, AccountId)> {
    vec![
        (Trademark{ name: b"ETH-kovan".to_vec(), id: 1 }, owned.clone()),
        (Trademark{ name: b"ABOS-test".to_vec(), id: 2 }, owned.clone()),
    ]
}

/// Helper function to generate brands.
pub fn get_nodeinformation() -> Vec<(Vec<u8>,Vec<u8>,Vec<u8>)> {
	vec![
		([00u8].to_vec(), [01u8].to_vec(),[02u8].to_vec()),
	]
}

/// Helper function to generate bond that linked authority and ethereum public key.
pub fn join_authorities_eth_bond(
    initial_authorities:& Vec<(AccountId, AccountId, AuthorityId)>, 
    bonds: Vec<Vec<u8>>
) -> Result<Vec<(AccountId,Vec<u8>)>, &str>{
    if initial_authorities.len() != bonds.len() {
        return Err("different data length");
    }

    let author_bonds:Vec<(AccountId,Vec<u8>)> = initial_authorities.iter()
        .zip(bonds.iter())
        .map(|((_stash, _controller, session), public_key)| {
            let mut r = [0u8; 32];
            r.copy_from_slice(session.as_ref());
            let mock_id: AccountId = UncheckedFrom::unchecked_from(r);
            return (mock_id, public_key.clone());
        }).collect();
    return Ok(author_bonds);
}

/// Helper function to create GenesisConfig for testing
pub fn testnet_genesis(
    initial_authorities: Vec<(AccountId, AccountId, AuthorityId)>,
    root_key: AccountId,
    endowed_accounts: Option<Vec<AccountId>>,
    enable_println: bool,
) -> GenesisConfig {
    let endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
        vec![
            get_account_id_from_seed("Alice"),
            get_account_id_from_seed("Bob"),
            get_account_id_from_seed("Charlie"),
            get_account_id_from_seed("Dave"),
            get_account_id_from_seed("Eve"),
            get_account_id_from_seed("Ferdie"),
            get_account_id_from_seed("Alice//stash"),
            get_account_id_from_seed("Bob//stash"),
            get_account_id_from_seed("Charlie//stash"),
            get_account_id_from_seed("Dave//stash"),
            get_account_id_from_seed("Eve//stash"),
            get_account_id_from_seed("Ferdie//stash"),
            hex!("0bdb300d3f861c5f9dad27b4f2d37b613ab59c689f469cbb19b5844b75e02985")
                .unchecked_into(),
        ]
    });

    let ethereum_public_keys: Vec<Vec<u8>> = vec![
        // alice ethereum public key
        hex!("4707a1eb3028f30b0d8646ca22bf8791210de0e8b80bcd1bbd7176c96fbaa2a223d365c7bf10f69dc8f45de2f182cf1f7217d93b69993c0b32445892b0d768b7").to_vec()
    ];

	const STASH: u128 = 1 << 20;
    const ENDOWMENT: u128 = 1 << 20;
	const REWARDYEAR: u128 = 10_000_000_000 * 1_000_000_000;

    let mut contract_config = ContractConfig {
        signed_claim_handicap: 2,
        rent_byte_price: 4,
        rent_deposit_offset: 1000,
        storage_size_offset: 8,
        surcharge_reward: 150,
        tombstone_deposit: 16,
        transaction_base_fee: 1,
        transaction_byte_fee: 0,
        transfer_fee: 0,
        creation_fee: 0,
        contract_fee: 21,
        call_base_fee: 135,
        create_base_fee: 175,
        gas_price: 1,
        max_depth: 1024,
        block_gas_limit: 10_000_000,
        current_schedule: Default::default(),
    };
    // this should only be enabled on development chains
    contract_config.current_schedule.enable_println = enable_println;

    GenesisConfig {
		consensus: Some(ConsensusConfig {
			code: include_bytes!("../../runtime/wasm/target/wasm32-unknown-unknown/release/node_runtime.compact.wasm").to_vec(),
			authorities: initial_authorities.iter().map(|x| x.2.clone()).collect(),
		}),
		system: None,
		indices: Some(IndicesConfig {
			ids: endowed_accounts.clone(),
		}),
		balances: Some(BalancesConfig {
			transaction_base_fee: 1,
			transaction_byte_fee: 0,
			existential_deposit: 500,
			transfer_fee: 0,
			creation_fee: 0,
			balances: endowed_accounts.iter().map(|k| (k.clone(), ENDOWMENT)).collect(),
			vesting: vec![],
		}),
		session: Some(SessionConfig {
			validators: initial_authorities.iter().map(|x| x.1.clone()).collect(),
			session_length: 10,
			keys: initial_authorities.iter().map(|x| (x.1.clone(), x.2.clone())).collect::<Vec<_>>(),
		}),
		staking: Some(StakingConfig {
			current_era: 0,
			minimum_validator_count: 1,
			validator_count: 2,
			sessions_per_era: 5,
			bonding_duration: 12,
			offline_slash: Perbill::zero(),
			session_reward: Perbill::zero(),
			current_session_reward: 0,
			offline_slash_grace: 0,
			stakers: initial_authorities.iter().map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator)).collect(),
			invulnerables: initial_authorities.iter().map(|x| x.1.clone()).collect(),
			nodeinformation: get_nodeinformation(),
			reward_per_year:REWARDYEAR,
		}),
		democracy: Some(DemocracyConfig {
			launch_period: 9,
			voting_period: 18,
			minimum_deposit: 10,
			public_delay: 0,
			max_lock_periods: 6,
		}),
		council_seats: Some(CouncilSeatsConfig {
			active_council: endowed_accounts.iter()
				.filter(|&endowed| initial_authorities.iter().find(|&(_, controller, _)| controller == endowed).is_none())
				.map(|a| (a.clone(), 1000000)).collect(),
			candidacy_bond: 10,
			voter_bond: 2,
			present_slash_per_voter: 1,
			carry_count: 4,
			presentation_duration: 10,
			approval_voting_period: 20,
			term_duration: 1000000,
			desired_seats: (endowed_accounts.len() / 2 - initial_authorities.len()) as u32,
			inactive_grace_period: 1,
		}),
		council_voting: Some(CouncilVotingConfig {
			cooloff_period: 75,
			voting_period: 20,
			enact_delay_period: 0,
		}),
		timestamp: Some(TimestampConfig {
			minimum_period: 2,                    // 2*2=4 second block time.
		}),
		treasury: Some(TreasuryConfig {
			proposal_bond: Permill::from_percent(5),
			proposal_bond_minimum: 1_000_000,
			spend_period: 12 * 60 * 24,
			burn: Permill::from_percent(50),
		}),
		contract: Some(contract_config),
		sudo: Some(SudoConfig {
			key: root_key.clone(),
		}),
		grandpa: Some(GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.2.clone(), 1)).collect(),
		}),
		bank: Some(BankConfig{
			enable_record: true,
			session_length: 10,
			reward_session_value: vec![1000,5000,60000,80000],
			reward_session_factor: vec![1,2,3,4],
			reward_balance_value: vec![1000,5000,60000,80000],
			reward_balance_factor: vec![1,2,3,4],
			total:0 ,
		}),

		signcheck: Some(SigncheckConfig {
			pubkey: join_authorities_eth_bond(&initial_authorities, ethereum_public_keys).unwrap(),
			athorities: initial_authorities.iter().map(|x| (x.2.clone()) ).collect(),
		}),
        brand: Some(BrandConfig {
            brands: get_brands(root_key),
        }),
		erc: Some(ErcConfig{
			acc: initial_authorities[0].1.clone(),
			enable_record:true,
			total:0,
		}),
		otc: Some(OtcConfig {
			athorities: initial_authorities.iter().map(|x| (x.2.clone()) ).collect(),
			exchangelad : get_exchangelad(),
		}),
	}
}

fn development_config_genesis() -> GenesisConfig {
    testnet_genesis(
        vec![get_authority_keys_from_seed("Alice")],
        get_account_id_from_seed("Alice"),
        None,
        true,
    )
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Development",
        "dev",
        development_config_genesis,
        vec![],
        None,
        None,
        None,
        None,
    )
}

fn local_testnet_genesis() -> GenesisConfig {
    testnet_genesis(
        vec![
            get_authority_keys_from_seed("Alice"),
            get_authority_keys_from_seed("Bob"),
        ],
        get_account_id_from_seed("Alice"),
        None,
        false,
    )
}

/// Local testnet config (multivalidator Alice + Bob)
pub fn local_testnet_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Local Testnet",
        "local_testnet",
        local_testnet_genesis,
        vec![],
        None,
        None,
        None,
        None,
    )
}

fn ladder_testnet_genesis() -> GenesisConfig {
    // stash , control, session
    let initial_authorities: Vec<(AccountId, AccountId, AuthorityId)> = vec![
        (
            hex!["ae626c4207a4fcd0360862172347e5078ee8c249649f7a0a1b30e8375ba35d0f"]
                .unchecked_into(), // 5G1MRi1tTL3yT7sLxote4X71mrrknmD1V4wqV46cXfmvSXb6
            hex!["003911b2203ab1d9f8d0c800b3918c9a08b96a760a3848e69cf0b766498fee4a"]
                .unchecked_into(), // 5C4zowsXUqg6qA3ggBjUQjcSnWc3Di95PRthTKhcJ4fkKx65
            hex!["f223b164b22ca82b3c5b2d83b7598c47281eebe28ced1427791846f2c7cfaeb8"]
                .unchecked_into(), // 5DGQfV2rtWo1iCoT7t5yapBz9ww6CrcTJ48TCUMLqmfjhcJd
        ),
        (
            hex!["541339a3f3406f14912f2c493d04d8863c1eaeaff06e1a03d7de82ee7d89aa7f"]
                .unchecked_into(), // 5DxwbCqB2sE72BG8oiP3seCAzLR8t9rAZaKYbQnd7Yj4srmW
            hex!["30891634febca7e0aef901470aed018c1237f7b64a994e6ad2a9e2e4eed8476a"]
                .unchecked_into(), // 5DALsvrp2uhBMwgaBKGKRXgcqa1nTE5cHHNyXsi8JU7BqRAC
            hex!["35294d534d5163927fb4e622ff6fb9dc98c819fb4356a99902dc16f8f3c13176"]
                .unchecked_into(), // 5DGQfV2rtWo1iCoT7t5yapBz9ww6CrcTJ48TCUMLqmfjhcJd
        ),
        (
            hex!["e2fb12d87bbb9eb57949b3eb2dd01cc24823194828fb36933f584971dd3e084a"]
                .unchecked_into(), // 5HCKGEpkLBvq4qwW3ULCirPWBNJycG7Jg36bLRLvG1J722us
            hex!["7a7358d81391a4f9048efbb13ebf8be3de7b4b5c34dddd835609088647d27a4a"]
                .unchecked_into(), // 5EqFxtEx9QQN3BJx7bHi9YmLxDzYmTj6ChTtdqhuk9WbrUUD
            hex!["7878bb590eb1a9fde93690038580c84b966c04e838c2005bcf428772eed396c9"]
                .unchecked_into(), // 5EnfU7B4zEpwd6jvDQxna349JTYDhcoWDgTvRMu61D8wwtur
        ),
        (
            hex!["96c72cbd7e76353d9051020c9ca8d41fa3b56770ceabf91e3e2610ffcbc60476"]
                .unchecked_into(), // 5FUQCy3HjgTuutFyzZUe1AtQFaU39yH5HU1coEiMgWhtpqFx
            hex!["1a4ad302298e0f647aae3d31b78083655e1447295bbf4559769e6a4e1f5adf75"]
                .unchecked_into(), // 5CfBL54BxiJiEbiovTXHfRGr75kjmL4mhZfhJk1D9wYYyDQt
            hex!["b15c0fb0a6a68b8c62c9c704c5671fb94235e81a9718e7a0217b7bfdca8d1c17"]
                .unchecked_into(), // 5G5FgS2DvWpFTW4pjNL4ugzUYMwEPRuoiWWVX1sE3U49T293
        ),
    ];

    let ethereum_public_keys: Vec<Vec<u8>> = vec![
        hex!("b59fe985fd3c00de38aa1094d7a8a34914771d025a8038238502d07e8b4048ac9afb68d22c77376921a056cf41f6f659f4f70f9b07d2887ffe3b9362d57070b6").to_vec(),
        hex!("cd3463a8be2cc3d3d589d5dc3e91a0f3f49fdafc201d0a9da0c684b285acf588c1945c8bae64112408b6d5e5084da11de530891df665bd666b0a2df98157ca0d").to_vec(),
        hex!("1c8e0106db1a5e3ca1228366bb3b6f7a9dee067d8d40bedcbc5d11eec30eb65e69b5f44241927108e71d14c54f386142360c9b57485904b3bb181c01ff8ca5f1").to_vec(),
        hex!("84f934cd44c0a044cc808dc11ce42623560b92961c2d4fc58d160006a7893bf47cf66a726388749a5b2cc46f85a21e41affca5156094003ced2669c605d6d251").to_vec(),
    ];

    // root account.
    let endowed_accounts: Vec<AccountId> = vec![
        hex!["58149eabec2e986b0dec740f243bbb836f6f6dc48a656e7c036471f1f6e06f6d"].unchecked_into(), //5E4CCLsJ3P1UBXgRdzFEQivMMJEqfg3VBj1tpvx8dsJa2FxQ
    ];
    const MILLICENTS: u128 = 1_000;
    const CENTS: u128 = 1_000 * MILLICENTS; // assume this is worth about a cent.
    const DOLLARS: u128 = 100 * CENTS;

    const SECS_PER_BLOCK: u64 = 8;
    const MINUTES: u64 = 60 / SECS_PER_BLOCK;
    const HOURS: u64 = MINUTES * 60;
    const DAYS: u64 = HOURS * 24;

    const ENDOWMENT: u128 = 10_000_000 * DOLLARS;
    const STASH: u128 = 100 * DOLLARS;
	const REWARDYEAR: u128 = 10_000_000_000 * 1_000_000_000;

    GenesisConfig {
		consensus: Some(ConsensusConfig {
			code: include_bytes!("../../runtime/wasm/target/wasm32-unknown-unknown/release/node_runtime.compact.wasm").to_vec(),    // FIXME change once we have #1252
			authorities: initial_authorities.iter().map(|x| x.2.clone()).collect(),
		}),
		system: None,
		balances: Some(BalancesConfig {
			transaction_base_fee: 1 * CENTS,
			transaction_byte_fee: 10 * MILLICENTS,
			balances: endowed_accounts.iter().cloned()
				.map(|k| (k, ENDOWMENT))
				.chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH)))
				.chain(initial_authorities.iter().map(|x| (AccountId::unchecked_from(x.2.clone().0), STASH))) // FIX oracle no need fee
				.collect(),
			existential_deposit: 1 * DOLLARS,
			transfer_fee: 1 * CENTS,
			creation_fee: 1 * CENTS,
			vesting: vec![],
		}),
		indices: Some(IndicesConfig {
			ids: endowed_accounts.iter().cloned()
				.chain(initial_authorities.iter().map(|x| x.0.clone()))
				.chain(initial_authorities.iter().map(|x| x.1.clone()))
				.collect::<Vec<_>>(),
		}),
		session: Some(SessionConfig {
			validators: initial_authorities.iter().map(|x| x.1.clone()).collect(),
			session_length: 5 * MINUTES,
			keys: initial_authorities.iter().map(|x| (x.1.clone(), x.2.clone())).collect::<Vec<_>>(),
		}),
		staking: Some(StakingConfig {
			current_era: 0,
			offline_slash: Perbill::from_billionths(1_000_000),
			session_reward: Perbill::from_billionths(2_065),
			current_session_reward: 0,
			validator_count: 7,
			sessions_per_era: 12,
			bonding_duration: 12,
			offline_slash_grace: 4,
			minimum_validator_count: 4,
			stakers: initial_authorities.iter().map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator)).collect(),
			invulnerables: initial_authorities.iter().map(|x| x.1.clone()).collect(),
			nodeinformation: get_nodeinformation(),
			reward_per_year: REWARDYEAR
		}),
		democracy: Some(DemocracyConfig {
			launch_period: 10 * MINUTES,    // 1 day per public referendum
			voting_period: 10 * MINUTES,    // 3 days to discuss & vote on an active referendum
			minimum_deposit: 50 * DOLLARS,    // 12000 as the minimum deposit for a referendum
			public_delay: 10 * MINUTES,
			max_lock_periods: 6,
		}),
		council_seats: Some(CouncilSeatsConfig {
			active_council: vec![],
			candidacy_bond: 10 * DOLLARS,
			voter_bond: 1 * DOLLARS,
			present_slash_per_voter: 1 * CENTS,
			carry_count: 6,
			presentation_duration: 1 * DAYS,
			approval_voting_period: 2 * DAYS,
			term_duration: 28 * DAYS,
			desired_seats: 0,
			inactive_grace_period: 1,    // one additional vote should go by before an inactive voter can be reaped.
		}),
		council_voting: Some(CouncilVotingConfig {
			cooloff_period: 4 * DAYS,
			voting_period: 1 * DAYS,
			enact_delay_period: 0,
		}),
		timestamp: Some(TimestampConfig {
			minimum_period: SECS_PER_BLOCK / 2, // due to the nature of aura the slots are 2*period
		}),
		treasury: Some(TreasuryConfig {
			proposal_bond: Permill::from_percent(5),
			proposal_bond_minimum: 1 * DOLLARS,
			spend_period: 1 * DAYS,
			burn: Permill::from_percent(50),
		}),
		contract: Some(ContractConfig {
			signed_claim_handicap: 2,
			rent_byte_price: 4,
			rent_deposit_offset: 1000,
			storage_size_offset: 8,
			surcharge_reward: 150,
			tombstone_deposit: 16,
			transaction_base_fee: 1 * CENTS,
			transaction_byte_fee: 10 * MILLICENTS,
			transfer_fee: 1 * CENTS,
			creation_fee: 1 * CENTS,
			contract_fee: 1 * CENTS,
			call_base_fee: 1000,
			create_base_fee: 1000,
			gas_price: 1 * MILLICENTS,
			max_depth: 1024,
			block_gas_limit: 10_000_000,
			current_schedule: Default::default(),
		}),
		sudo: Some(SudoConfig {
			key: endowed_accounts[0].clone(),
		}),
		grandpa: Some(GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.2.clone(), 1)).collect(),
		}),
		bank: Some(BankConfig{
			enable_record: true,
			session_length: 10,
			reward_session_value: vec![1000,5000,60000,80000],
			reward_session_factor: vec![1,2,3,4],
			reward_balance_value: vec![1000,5000,60000,80000],
			reward_balance_factor: vec![1,2,3,4],
			total:0 ,
		}),

		signcheck: Some(SigncheckConfig {
			pubkey: join_authorities_eth_bond(&initial_authorities, ethereum_public_keys).unwrap(),
			athorities: initial_authorities.iter().map(|x| (x.2.clone()) ).collect(),
		}),
        brand: Some(BrandConfig {
            brands: get_brands(endowed_accounts[0].clone()),
        }),
		erc: Some(ErcConfig{
			acc: initial_authorities[0].1.clone(),
			enable_record:true,
			total:0,
		}),
		otc: Some(OtcConfig {
			athorities: initial_authorities.iter().map(|x| (x.2.clone()) ).collect(),
			exchangelad : get_exchangelad(),
		}),
	}
}

/// ladder testnet config
pub fn ladder_testnet_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Ladder Testnet v0.5.0",
        "Ladder Testnet",
        ladder_testnet_genesis,
        vec![],
        None,
        None,
        None,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::Factory;
    use service_test;

    fn local_testnet_genesis_instant() -> GenesisConfig {
        let mut genesis = local_testnet_genesis();
        genesis.timestamp = Some(TimestampConfig { minimum_period: 1 });
        genesis
    }

    /// Local testnet config (multivalidator Alice + Bob)
    pub fn integration_test_config() -> ChainSpec {
        ChainSpec::from_genesis(
            "Integration Test",
            "test",
            local_testnet_genesis_instant,
            vec![],
            None,
            None,
            None,
            None,
        )
    }

    #[test]
    fn test_connectivity() {
        service_test::connectivity::<Factory>(integration_test_config());
    }
}
