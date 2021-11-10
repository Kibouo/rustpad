use anyhow::{anyhow, Result};
use reqwest::Url;

use crate::text::cypher_text::CypherText;

use super::{oracle_location::OracleLocation, Oracle};

pub struct WebOracle {
    url: Url,
}

impl Oracle for WebOracle {
    fn visit(oracle_location: &OracleLocation) -> Result<Self> {
        let url = match oracle_location {
            OracleLocation::Web(url) => url,
            OracleLocation::Script(_) => {
                return Err(anyhow!("Tried to visit the web oracle using a file path!"))
            }
        };

        Ok(Self {
            url: url.to_owned(),
        })
    }

    fn ask_validation(&self, cypher_text: CypherText) -> Result<bool> {
        todo!()
    }

    fn location(&self) -> OracleLocation {
        OracleLocation::Web(self.url.clone())
    }
}
