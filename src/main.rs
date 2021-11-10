mod block;
mod cli;
mod oracle;
mod questioning;

use anyhow::Result;

use crate::{
    block::{block_question::cypher_text::CypherText, block_size::BlockSize},
    cli::{block_size_option::BlockSizeOption, Options},
    questioning::Questioning,
};

fn main() -> Result<()> {
    let options = Options::parse()?;

    let cypher_text = match options.block_size() {
        BlockSizeOption::Eight => CypherText::decode(options.cypher_text(), &BlockSize::Eight)?,
        BlockSizeOption::Sixteen => CypherText::decode(options.cypher_text(), &BlockSize::Sixteen)?,
        BlockSizeOption::Auto => todo!(),
    };

    let question = Questioning::prepare(cypher_text)?.start(options.oracle_location())?;
    eprintln!("question = {:#?}", question);

    Ok(())
}
