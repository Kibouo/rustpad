use std::str::FromStr;

use anyhow::{Context, Result};
use getset::Getters;

#[derive(Debug, Clone, Getters)]
pub struct Header {
    #[get = "pub"]
    name: String,
    #[get = "pub"]
    value: String,
}

impl FromStr for Header {
    type Err = anyhow::Error;

    fn from_str(header: &str) -> Result<Self> {
        header
            .split_once(':')
            .map(|(l, r)| Header {
                name: l.trim().to_owned(),
                value: r.trim().to_owned(),
            })
            .context(format!(
                "`{}` is not a valid header. Expected format `<name>:<value>`",
                header
            ))
    }
}
