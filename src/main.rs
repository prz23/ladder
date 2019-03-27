#![warn(missing_docs)]
#![warn(unused_extern_crates)]

extern crate structopt;
extern crate tokio;
extern crate futures;
extern crate serde_json;
#[macro_use]
extern crate error_chain;
extern crate substrate_cli;
extern crate substrate_primitives as primitives;
extern crate substrate_consensus_aura as consensus;
extern crate substrate_inherents as inherents;
extern crate substrate_client as client;
#[macro_use]
extern crate substrate_network as network;
#[macro_use]
extern crate substrate_executor;
extern crate substrate_telemetry;
extern crate substrate_transaction_pool as transaction_pool;
extern crate substrate_basic_authorship as basic_authorship;
extern crate substrate_finality_grandpa as grandpa;
#[macro_use]
extern crate substrate_service;
extern crate substrate_keystore;
extern crate node_runtime;
extern crate vendor;
extern crate signer;

mod chain_spec;
mod service;
mod cli;
mod params;

pub use substrate_cli::{VersionInfo, IntoExit, error};

fn run() -> cli::error::Result<()> {
    let version = VersionInfo {
        name: "ABMatrix Node",
        author: "ABMatrix",
        commit: env!("VERGEN_SHA_SHORT"),
        version: env!("CARGO_PKG_VERSION"),
        description: "A cross chain node of ABMatrix",
        support_url: "https://github.com/paritytech/substrate/issues/new",
        executable_name: "abmatrix",
    };
    cli::run(::std::env::args(), cli::Exit, version)
}

quick_main!(run);
