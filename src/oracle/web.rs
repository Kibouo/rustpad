use anyhow::{anyhow, Result};
use reqwest::Url;

use crate::cypher_text::encode::Encode;

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

        let oracle = Self {
            url: url.to_owned(),
        };
        Ok(oracle)
    }

    fn ask_validation<'a>(&self, cypher_text: &'a impl Encode<'a>) -> Result<bool> {
        todo!()
    }

    fn location(&self) -> OracleLocation {
        OracleLocation::Web(self.url.clone())
    }
}
