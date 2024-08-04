use std::time::Duration;

use reqwest::Client;

use crate::Config;

enum State {
    Init,
    Checking,
    Failed,
    GraceWaiting,
}

pub struct Service {
    config: Config,
    client: Client,

    remaining_retries: u32,
}

impl Service {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let client = reqwest::ClientBuilder::new()
            .connect_timeout(Duration::from_millis(config.grace.timeout_ms)) // TODO: Make configurable
            .build()?;

        let remaining_retries = config.grace.retry_count;

        Ok(Self {
            config,
            client,
            remaining_retries,
        })
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let mut state = State::Init;

        loop {
            state = match state {
                State::Init => self.init().await?,
                State::Checking => self.check().await?,
                State::Failed => self.failed().await?,
                State::GraceWaiting => self.grace_waiting().await?,
            }
        }
    }

    async fn init(&mut self) -> anyhow::Result<State> {
        log::info!("Started health check");

        Ok(State::Checking)
    }

    async fn check(&mut self) -> anyhow::Result<State> {
        let health_request = self.config.request.make_request(&self.client)?;
        let response = self
            .client
            .execute(health_request)
            .await
            .and_then(|res| res.error_for_status());

        if let Err(e) = response {
            log::warn!("Health check failed: {:?}", e);

            if self.remaining_retries == 0 {
                self.remaining_retries = self.config.grace.retry_count;
                return Ok(State::Failed);
            }

            self.remaining_retries -= 1;
        }

        tokio::time::sleep(Duration::from_millis(self.config.grace.check_interval_ms)).await;
        Ok(State::Checking)
    }

    async fn failed(&mut self) -> anyhow::Result<State> {
        let command_result = self.config.command.execute();
        if command_result.is_err() {
            log::error!("Failed to run command, {:?}", command_result.err().unwrap());
        }

        Ok(State::GraceWaiting)
    }

    async fn grace_waiting(&mut self) -> anyhow::Result<State> {
        tokio::time::sleep(Duration::from_millis(
            self.config.grace.wait_after_command_ms,
        ))
        .await;

        Ok(State::Checking)
    }
}
