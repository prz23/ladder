//! Substrate Node Template CLI library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod chain_spec;
mod service;
mod cli;

pub use cli::{VersionInfo, IntoExit, error};

fn run() -> cli::error::Result<()> {
    let version = VersionInfo {
        name: "ABMatrix Node",
        commit: env!("VERGEN_SHA_SHORT"),
        version: env!("CARGO_PKG_VERSION"),
        executable_name: "abmatrix",
        author: "ABMatrix",
        description: "A cross chain node of ABMatrix",
        support_url: "https://github.com/paritytech/substrate/issues/new",
    };
	cli::run(::std::env::args(), cli::Exit, version)
}

error_chain::quick_main!(run);