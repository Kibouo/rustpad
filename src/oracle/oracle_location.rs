use anyhow::{anyhow, Context, Result};
use is_executable::IsExecutable;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};

#[derive(Debug, Clone)]
pub(crate) enum OracleLocation {
    Web(Url),
    Script(PathBuf),
}

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone)]
pub(crate) enum SerializableOracleLocation {
    Web(String),
    Script(PathBuf),
}

impl FromStr for OracleLocation {
    type Err = anyhow::Error;

    fn from_str(oracle_location: &str) -> Result<Self> {
        Url::parse(oracle_location).map(Self::Web).or_else(|_| {
            let path = PathBuf::from(oracle_location);
            if !path.is_file() {
                Err(anyhow!(
                    "`{}` does not point to a file. Double check the path",
                    oracle_location
                ))
            } else if !path.is_executable() {
                Err(anyhow!(
                    "`{}` is not executable. Double check its permissions",
                    oracle_location
                ))
            } else {
                Ok(Self::Script(path))
            }
        })
    }
}

impl From<OracleLocation> for SerializableOracleLocation {
    fn from(oracle_location: OracleLocation) -> Self {
        match oracle_location {
            OracleLocation::Web(url) => Self::Web(String::from(url.as_str())),
            OracleLocation::Script(path) => Self::Script(path),
        }
    }
}

impl From<SerializableOracleLocation> for OracleLocation {
    fn from(oracle_location: SerializableOracleLocation) -> Self {
        match oracle_location {
            SerializableOracleLocation::Web(url) => Self::Web(url.parse().context("URL stored in cache is invalid").expect("Data stored in the cache was verified when it was created. As such, the only possible reason for this must be a corrupted cache file.")),
            SerializableOracleLocation::Script(path) => Self::Script(path),
        }
    }
}
