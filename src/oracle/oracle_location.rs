use anyhow::{anyhow, Context, Result};
use is_executable::IsExecutable;
use reqwest::Url;
use std::{convert::TryFrom, path::PathBuf};

#[derive(Debug)]
pub enum OracleLocation {
    Web(Url),
    Script(PathBuf),
}

impl OracleLocation {
    pub fn new(oracle_location: &str, oracle_type: &str) -> Result<Self> {
        match oracle_type {
            "web" => Ok(Self::Web(
                Url::try_from(oracle_location)
                    .context(format!("URL format invalid: {}", oracle_location))?,
            )),
            "script" => {
                let path = PathBuf::from(oracle_location);
                return if !path.is_file() {
                    Err(anyhow!("Path does not point to file: {}", oracle_location))
                } else if !path.is_executable() {
                    Err(anyhow!("Can't execute file: {}", oracle_location))
                } else {
                    Ok(Self::Script(path))
                };
            }
            _ => unreachable!(format!("Sub-command invalid: {}", oracle_type)),
        }
    }
}
