use std::str::FromStr;

use anyhow::{Context, Result};
use getset::Getters;

#[derive(Debug, Getters)]
pub(super) struct ProxyCredentials {
    #[get = "pub(super)"]
    username: String,
    #[get = "pub(super)"]
    password: String,
}

impl FromStr for ProxyCredentials {
    type Err = anyhow::Error;

    fn from_str(proxy_credentials: &str) -> Result<Self> {
        let split_creds = proxy_credentials.split_once(':').context(format!(
            "`{}`. Expected format `<username>:<password>`",
            proxy_credentials
        ))?;
        Ok(Self {
            username: split_creds.0.to_owned(),
            password: split_creds.1.to_owned(),
        })
    }
}
