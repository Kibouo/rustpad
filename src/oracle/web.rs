use anyhow::{anyhow, Result};
use reqwest::Url;

use crate::{cli::oracle_location::OracleLocation, question::Question};

use super::Oracle;

pub struct WebOracle {
    url: Url,
}

impl Oracle for WebOracle {
    fn visit(oracle_location: OracleLocation) -> Result<Self> {
        let url = match oracle_location {
            OracleLocation::Web(url) => url,
            OracleLocation::Script(_) => {
                return Err(anyhow!("Tried to visit a 'web' oracle using a file path!"))
            }
        };

        Ok(Self { url })
    }

    fn ask_validation(self, question: Question) -> bool {
        todo!()
    }

    fn location(&self) -> OracleLocation {
        OracleLocation::Web(self.url.clone())
    }
}
