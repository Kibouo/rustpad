mod cli;
mod cypher_text;
mod oracle;
mod question;

use anyhow::Result;
use oracle::script::ScriptOracle;

use crate::{
    cli::{oracle_location::OracleLocation, Options},
    oracle::{web::WebOracle, Oracle},
    question::Question,
};

fn main() -> Result<()> {
    let options = Options::parse()?;

    eprintln!("options = {:#?}", options);
    // let question = Question::formulate();

    let oracle: Box<dyn Oracle> = match options.oracle_location {
        OracleLocation::Web(_) => Box::new(WebOracle::visit(options.oracle_location)?),
        OracleLocation::Script(_) => Box::new(ScriptOracle::visit(options.oracle_location)?),
    };
    // oracle.ask_validation(question);

    Ok(())
}
