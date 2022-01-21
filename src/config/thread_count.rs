use std::{fmt::Display, ops::Deref, str::FromStr};

use anyhow::{anyhow, Context, Result};

#[derive(Debug, Clone)]
pub struct ThreadCount(usize);

impl Default for ThreadCount {
    fn default() -> Self {
        ThreadCount(64)
    }
}

impl FromStr for ThreadCount {
    type Err = anyhow::Error;

    fn from_str(thread_count: &str) -> Result<Self> {
        let thread_count = thread_count.parse::<usize>().context(format!(
            "`{}`. Expected a positive, non-zero integer",
            thread_count
        ))?;
        if thread_count > 0 {
            Ok(Self(thread_count))
        } else {
            Err(anyhow!(
                "`{}`. Expected a positive, non-zero integer",
                thread_count
            ))
        }
    }
}

impl Deref for ThreadCount {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for ThreadCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
