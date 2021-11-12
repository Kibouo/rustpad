mod block;
mod cli;
mod cypher_text;
mod oracle;
mod questioning;

use anyhow::Result;

use crate::{
    block::block_size::BlockSize,
    cli::{block_size_option::BlockSizeOption, Options},
    cypher_text::CypherText,
    oracle::{oracle_location::OracleLocation, script::ScriptOracle, web::WebOracle, Oracle},
    questioning::Questioning,
};

fn main() -> Result<()> {
    let options = Options::parse()?;

    let cypher_text = match options.block_size() {
        BlockSizeOption::Eight => CypherText::parse(options.cypher_text(), &BlockSize::Eight)?,
        BlockSizeOption::Sixteen => CypherText::parse(options.cypher_text(), &BlockSize::Sixteen)?,
        BlockSizeOption::Auto => todo!(),
    };

    let decoded = match options.oracle_location() {
        OracleLocation::Web(_) => {
            let oracle = WebOracle::visit(options.oracle_location())?;
            Questioning::prepare(&cypher_text)?.start(oracle)?
        }
        OracleLocation::Script(_) => {
            let oracle = ScriptOracle::visit(options.oracle_location())?;
            Questioning::prepare(&cypher_text)?.start(oracle)?
        }
    };

    eprintln!("decoded = {:#?}", decoded);

    Ok(())
}
