use std::{
    collections::HashMap,
    path::{Path, PathBuf},
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
    Empty,
    String(String),
    File(PathBuf),
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
            .build()?;

        Ok(request)
    }
}

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

        let result = command_builder.spawn()?.wait()?;

        if !result.success() {
            bail!("Non-zero exit of command: {:?}", result);
        }

        Ok(())
    }
}

/// Main configuration for the health check and run command
#[derive(Debug, Deserialize)]
pub struct Config {
    /// A Health request to make periodically.
    pub request: Request,

    /// A command to run when a health check fails.
    pub command: Command,
}

impl Config {
    pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file_contents = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&file_contents)?)
    }
}
