use std::{fmt::Display, ops::Deref, str::FromStr, time::Duration};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct ThreadDelay(Duration);

impl Default for ThreadDelay {
    fn default() -> Self {
        ThreadDelay(Duration::from_millis(0))
    }
}

impl FromStr for ThreadDelay {
    type Err = anyhow::Error;

    fn from_str(delay: &str) -> Result<Self> {
        delay
            .parse::<u64>()
            .context(format!("`{}`. Expected a positive integer", delay))
            .map(|millis| Self(Duration::from_millis(millis)))
    }
}

impl Deref for ThreadDelay {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for ThreadDelay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.as_millis())
    }
}
