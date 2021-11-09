use anyhow::Result;

use crate::{cli::oracle_location::OracleLocation, question::Question};

pub mod script;
pub mod web;

pub trait Oracle {
    fn visit(oracle_location: OracleLocation) -> Result<Self>
    where
        Self: Sized;
    fn ask_divination(self, )
    fn ask_validation(self, question: Question) -> bool;
    fn location(&self) -> OracleLocation;
}
