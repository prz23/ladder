use structopt::StructOpt;
use cli::CoreParams;

/// Extend params for Node
#[derive(Debug, StructOpt)]
pub struct Params {
	#[structopt(flatten)]
	core: CoreParams
}
