pub(super) mod oracle_location;
pub(super) mod script;
pub(super) mod web;

use anyhow::Result;

use crate::{
    config::{thread_delay::ThreadDelay, SubConfig},
    cypher_text::encode::Encode,
};

use self::oracle_location::OracleLocation;

pub(super) trait Oracle: Sync {
    /// Constructor
    fn visit(oracle_location: &OracleLocation, oracle_config: &SubConfig) -> Result<Self>
    where
        Self: Sized;

    /// Ask endpoint to verify cypher text. Return true if padding is valid.
    fn ask_validation<'a>(&self, cypher_text: &'a impl Encode<'a>) -> Result<bool>;

    fn location(&self) -> OracleLocation;
    fn thread_delay(&self) -> &ThreadDelay;
}
