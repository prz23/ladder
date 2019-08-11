use cli::{AugmentClap, GetLogFilter};
use structopt::{clap::App, StructOpt};
use std::path::PathBuf;

#[derive(Debug, StructOpt, Clone, Default)]
pub struct VendorCmd {
    /// Config file
    #[structopt(long = "config", parse(from_os_str))]
    pub config: Option<PathBuf>,
    /// Enable listener mode
    #[structopt(long = "listener")]
    pub listener: bool,
    /// Enable sender mode
    #[structopt(long = "enableexchange")]
    pub enableexchange: bool,

}

impl GetLogFilter for VendorCmd {
    fn get_log_filter(&self) -> Option<String> {
        None
    }
}

impl AugmentClap for VendorCmd {
    fn augment_clap<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        app
    }
}
