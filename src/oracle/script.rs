use std::path::PathBuf;

use anyhow::{anyhow, Result};

use crate::{cli::oracle_location::OracleLocation, question::Question};

use super::Oracle;

pub struct ScriptOracle {
    path: PathBuf,
}

impl Oracle for ScriptOracle {
    fn visit(oracle_location: OracleLocation) -> Result<Self> {
        let path = match oracle_location {
            OracleLocation::Script(path) => path,
            OracleLocation::Web(_) => {
                return Err(anyhow!(
                    "Tried to visit a 'script' oracle using a file path!"
                ))
            }
        };

        Ok(Self { path })
    }

    fn ask_validation(self, question: Question) -> bool {
        todo!()
    }

    fn location(&self) -> OracleLocation {
        OracleLocation::Script(self.path.clone())
    }
}
