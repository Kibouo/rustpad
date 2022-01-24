use std::{fmt::Display, ops::Deref, str::FromStr, time::Duration};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub(crate) struct RequestTimeout(Duration);

impl Default for RequestTimeout {
    fn default() -> Self {
        RequestTimeout(Duration::from_secs(10))
    }
}

impl FromStr for RequestTimeout {
    type Err = anyhow::Error;

    fn from_str(delay: &str) -> Result<Self> {
        delay
            .parse::<u64>()
            .context(format!("`{}`. Expected a positive integer", delay))
            .map(|secs| Self(Duration::from_secs(secs)))
    }
}

impl Deref for RequestTimeout {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for RequestTimeout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.as_secs())
    }
}
