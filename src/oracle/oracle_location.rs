use anyhow::{anyhow, Context, Result};
use reqwest::Url;
use std::{convert::TryFrom, path::PathBuf};

#[derive(Debug)]
pub enum OracleLocation {
    Web(Url),
    Script(PathBuf),
}

impl TryFrom<(&str, &str)> for OracleLocation {
    type Error = anyhow::Error;

    fn try_from((oracle_location, oracle_type): (&str, &str)) -> Result<Self> {
        match oracle_type {
            "web" => Ok(Self::Web(
                Url::try_from(oracle_location)
                    .context(format!("Invalid URL format: {}", oracle_location))?,
            )),
            "script" => {
                let path = PathBuf::from(oracle_location);
                return if !path.is_file() {
                    Err(anyhow!("Invalid file path: {}", oracle_location))
                } else {
                    Ok(Self::Script(path))
                };
            }
            _ => unreachable!(format!("Invalid sub-command: {}", oracle_type)),
        }
    }
}
