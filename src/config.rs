use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::bail;
use reqwest::Method;
use serde::Deserialize;

pub mod prelude {
    pub use super::{Body, Config, Request};
}

/// A body specification for a request
#[derive(Debug, Default, Deserialize, Clone)]
pub enum Body {
    #[default]
    #[serde(rename = "empty")]
    Empty,
    #[serde(rename = "string")]
    String(String),
    #[serde(rename = "file")]
    File(PathBuf),
}

impl Body {
    /// Convert this body into a `reqwest::Body`
    pub fn as_body(&self) -> anyhow::Result<reqwest::Body> {
        match self.clone() {
            Body::Empty => Ok(reqwest::Body::default()),
            Body::String(s) => Ok(reqwest::Body::from(s)),
            Body::File(path) => {
                let body = std::fs::read_to_string(path)?;
                Ok(reqwest::Body::from(body))
            }
        }
    }
}

/// A request specification
#[derive(Debug, Deserialize, Clone)]
pub struct Request {
    /// The HTTP method to use.
    pub method: String,
    /// The URL to send the request to.
    pub url: String,
    /// The headers to send with the request.
    #[serde(default)]
    pub headers: HashMap<String, String>,

    #[serde(default)]
    pub body: Body,
}

impl Request {
    pub fn make_request(&self, client: &reqwest::Client) -> anyhow::Result<reqwest::Request> {
        use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

        let method = Method::from_bytes(self.method.as_bytes())?;

        let mut headers = HeaderMap::new();
        for (key, value) in self.headers.iter() {
            headers.append(
                HeaderName::from_bytes(key.as_bytes())?,
                HeaderValue::from_str(value)?,
            );
        }

        let request = client
            .request(method, self.url.clone())
            .headers(headers)
            .body(self.body.as_body()?)
            .build()?;

        Ok(request)
    }
}

/// Command to run specification
#[derive(Debug, Deserialize, Clone)]
pub struct Command {
    /// A command to run
    pub command: String,

    /// A working directory in which to execute the command, defaults to current directory
    pub working_dir: Option<PathBuf>,
}

impl Command {
    pub fn execute(&self) -> anyhow::Result<()> {
        use std::process::Command;

        let mut command_builder = Command::new("sh");
        command_builder.arg("-c").arg(&self.command);

        if let Some(working_dir) = &self.working_dir {
            command_builder.current_dir(working_dir);
        }

        let result = command_builder.output()?;

        if !result.status.success() {
            bail!("Non-zero exit of command: {}", result.status);
        } else {
            log::info!("Command executed successfully");
        }

        Ok(())
    }
}

/// Timeouts, Retry counts, etc. To allow do not run command only if failed one time

#[derive(Debug, Clone, Deserialize)]
pub struct Grace {
    #[serde(default = "Grace::default_check_interval")]
    check_interval_ms: u64,

    check_interval_failed_ms: Option<u64>,

    #[serde(default = "Grace::default_retry_count")]
    retry_count: u32,

    #[serde(default = "Grace::default_timeout")]
    timeout_ms: u64,

    #[serde(default = "Grace::default_wait_after_command")]
    wait_after_command_ms: u64,
}

impl Grace {
    // 1 second
    pub fn default_check_interval() -> u64 {
        1_000
    }

    // 3 times
    pub fn default_retry_count() -> u32 {
        3
    }

    // 30 seconds
    pub fn default_timeout() -> u64 {
        30_000
    }

    // 30 seconds
    pub fn default_wait_after_command() -> u64 {
        30_000
    }

    pub fn check_interval(&self) -> Duration {
        Duration::from_millis(self.check_interval_ms)
    }

    pub fn check_interval_failed(&self) -> Duration {
        match self.check_interval_failed_ms {
            Some(v) => Duration::from_millis(v),
            None => self.check_interval(),
        }
    }

    pub fn retry_count(&self) -> u32 {
        self.retry_count
    }

    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }

    pub fn wait_after_command(&self) -> Duration {
        Duration::from_millis(self.wait_after_command_ms)
    }
}

/// Main configuration for the health check and run command
#[derive(Debug, Deserialize)]
pub struct Config {
    /// A Health request to make periodically.
    pub request: Request,

    /// A command to run when a health check fails.
    pub command: Command,

    /// How often to run the health check and command.
    pub grace: Grace,
}

impl Config {
    pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file_contents = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&file_contents)?)
    }
}
