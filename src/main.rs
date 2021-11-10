mod cli;
mod oracle;
mod text;

use anyhow::Result;
use oracle::script::ScriptOracle;

use crate::{
    cli::{block_size_option::BlockSizeOption, Options},
    oracle::{oracle_location::OracleLocation, web::WebOracle, Oracle},
    text::{block_size::BlockSize, cypher_text::CypherText},
};

fn main() -> Result<()> {
    let options = Options::parse()?;

    let oracle: Box<dyn Oracle> = match options.oracle_location {
        OracleLocation::Web(_) => Box::new(WebOracle::visit(&options.oracle_location)?),
        OracleLocation::Script(_) => Box::new(ScriptOracle::visit(&options.oracle_location)?),
    };

    let cypher_text = match options.block_size {
        BlockSizeOption::Eight => CypherText::decode(&options.cypher_text, &BlockSize::Eight)?,
        BlockSizeOption::Sixteen => CypherText::decode(&options.cypher_text, &BlockSize::Sixteen)?,
        BlockSizeOption::Auto => todo!(),
    };

    oracle.ask_validation(cypher_text)?;

    Ok(())
}
