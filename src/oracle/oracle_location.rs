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
            "web" => Ok(Self::Web(Url::try_from(oracle_location).context(
                format!("URL `{}` invalid. Double check the URL", oracle_location),
            )?)),
            "script" => {
                let path = PathBuf::from(oracle_location);
                if !path.is_file() {
                    Err(anyhow!(
                        "Path `{}` does not point to a file. Double check the path",
                        oracle_location
                    ))
                } else if !path.is_executable() {
                    Err(anyhow!(
                        "File `{}` is not executable. Double check its permissions",
                        oracle_location
                    ))
                } else {
                    Ok(Self::Script(path))
                }
            }
            _ => unreachable!(format!("Sub-command invalid: {}", oracle_type)),
        }
    }
}
