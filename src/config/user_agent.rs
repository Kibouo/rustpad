use std::{fmt::Display, ops::Deref, str::FromStr};

use anyhow::Result;

const VERSION_TEMPLATE: &str = "<version>";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone)]
pub(crate) struct UserAgent(String);

// `<version>` with actual crate version
fn replace_version(user_agent: &str) -> String {
    user_agent.replace(VERSION_TEMPLATE, VERSION)
}

impl Default for UserAgent {
    fn default() -> Self {
        UserAgent(replace_version("rustpad/<version>"))
    }
}

impl FromStr for UserAgent {
    type Err = anyhow::Error;

    fn from_str(user_agent: &str) -> Result<Self> {
        Ok(UserAgent(replace_version(user_agent)))
    }
}

impl Deref for UserAgent {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for UserAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
