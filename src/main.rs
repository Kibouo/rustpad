mod block;
mod cli;
mod cypher_text;
mod oracle;
mod questioning;

use anyhow::Result;

use crate::{
    block::block_size::BlockSize,
    cli::{block_size_option::BlockSizeOption, Options},
    cypher_text::cypher_text::CypherText,
    questioning::Questioning,
};

fn main() -> Result<()> {
    let options = Options::parse()?;

    let cypher_text = match options.block_size() {
        BlockSizeOption::Eight => CypherText::parse(options.cypher_text(), &BlockSize::Eight)?,
        BlockSizeOption::Sixteen => CypherText::parse(options.cypher_text(), &BlockSize::Sixteen)?,
        BlockSizeOption::Auto => todo!(),
    };

    let question = Questioning::prepare(cypher_text)?.start(options.oracle_location())?;
    eprintln!("question = {:#?}", question);

    Ok(())
}
