use std::time::Duration;

mod args;
use args::prelude::*;

mod config;
pub use config::prelude::*;

async fn health_check(config: &Config) -> anyhow::Result<()> {
    log::info!("Staring health checking");

    let client = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(10)) // TODO: Make configurable
        .build()?;

    loop {
        let health_request = config.request.make_request(&client)?;
        let response = client
            .execute(health_request)
            .await
            .and_then(|res| res.error_for_status());
        log::debug!("Health check response: {:?}", response);

        if response.is_err() {
            log::error!(
                "Health check failed, running command, {}",
                response.err().unwrap(),
            );
            let command_result = config.command.execute();

            if command_result.is_err() {
                log::error!("Failed to run command, {:?}", command_result.err().unwrap());
            }
        }

        // TODO: Make configurable
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = Config::from_file(args.config)?;

    env_logger::init();

    return health_check(&config).await;
}
