use anyhow::Result;

use crate::{cli::SubOptions, cypher_text::encode::Encode};

use self::oracle_location::OracleLocation;

pub mod oracle_location;
pub mod script;
pub mod web;

pub trait Oracle: Sync {
    /// Constructor
    fn visit(oracle_location: &OracleLocation, oracle_options: &SubOptions) -> Result<Self>
    where
        Self: Sized;

    /// Ask endpoint to verify cypher text. Return true if padding is valid.
    fn ask_validation<'a>(&self, cypher_text: &'a impl Encode<'a>) -> Result<bool>;

    fn location(&self) -> OracleLocation;
}
