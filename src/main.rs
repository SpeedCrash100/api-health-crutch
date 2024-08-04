mod args;
use args::prelude::*;

mod config;
pub use config::prelude::*;

mod health_service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = Config::from_file(args.config)?;

    env_logger::init();

    let mut service = health_service::Service::new(config)?;
    return service.run().await;
}
