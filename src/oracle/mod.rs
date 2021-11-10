use anyhow::Result;

use crate::text::cypher_text::CypherText;

use self::oracle_location::OracleLocation;

pub mod oracle_location;
pub mod script;
pub mod web;

pub trait Oracle {
    fn visit(oracle_location: &OracleLocation) -> Result<Self>
    where
        Self: Sized;
    fn ask_validation(&self, cypher_text: CypherText) -> Result<bool>;
    fn location(&self) -> OracleLocation;
}
