use anyhow::Result;

use crate::text::block_question::BlockQuestion;

use self::oracle_location::OracleLocation;

pub mod oracle_location;
pub mod script;
pub mod web;

pub trait Oracle {
    /// New
    fn visit(oracle_location: &OracleLocation) -> Result<Self>
    where
        Self: Sized;
    /// Ask endpoint to verify cypher text
    fn ask_validation(&self, cypher_text: &BlockQuestion) -> Result<bool>;
    fn location(&self) -> OracleLocation;
}
