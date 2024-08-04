use std::path::PathBuf;

use clap::Parser;

pub mod prelude {
    pub use super::Args;
    pub use clap::Parser;
}

#[derive(Debug, Parser)]
pub struct Args {
    /// The path to the configuration file.
    #[arg(short = 'c', long = "config")]
    pub config: PathBuf,
}
