use std::{path::PathBuf, process::Command};

use anyhow::{anyhow, Context, Result};

use crate::{
    cli::{ScriptOptions, SubOptions},
    cypher_text::encode::Encode,
};

use super::{oracle_location::OracleLocation, Oracle};

pub struct ScriptOracle {
    path: PathBuf,
    _options: ScriptOptions,
}

impl Oracle for ScriptOracle {
    fn visit(oracle_location: &OracleLocation, oracle_options: &SubOptions) -> Result<Self> {
        let path = match oracle_location {
            OracleLocation::Script(path) => path,
            OracleLocation::Web(_) => {
                return Err(anyhow!("Tried to visit the script oracle using a URL!"))
            }
        };

        let options = match oracle_options {
            SubOptions::Script(options) => options,
            SubOptions::Web(_) => {
                return Err(anyhow!(
                    "Tried to visit the script oracle using web options!"
                ))
            }
        };

        let oracle = Self {
            path: path.to_path_buf(),
            _options: options.clone(),
        };
        Ok(oracle)
    }

    fn ask_validation<'a>(&self, cypher_text: &'a impl Encode<'a>) -> Result<bool> {
        let status = Command::new("/bin/sh")
            .arg("-c")
            .arg(format!(
                "{} {}",
                self.path
                    .as_path()
                    .to_str()
                    .ok_or_else(|| anyhow!("Invalid path: {}", self.path.display()))?,
                cypher_text.encode()
            ))
            .status()
            .context(format!("Failed to run script: {}", self.path.display()))?;

        Ok(status.success())
    }

    fn location(&self) -> OracleLocation {
        OracleLocation::Script(self.path.clone())
    }
}
