use anyhow::Result;

use crate::cypher_text::Encode;

use self::oracle_location::OracleLocation;

pub mod oracle_location;
pub mod script;
pub mod web;

pub trait Oracle: Sync {
    /// Ask endpoint to verify cypher text
    fn ask_validation<'a>(&self, cypher_text: &'a impl Encode<'a>) -> Result<bool>;

    fn location(&self) -> OracleLocation;
}
