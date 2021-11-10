mod cli;
mod oracle;
mod questioning;
mod text;

use anyhow::Result;

use crate::{
    cli::{block_size_option::BlockSizeOption, Options},
    questioning::Questioning,
    text::{block_question::cypher_text::CypherText, block_size::BlockSize},
};

fn main() -> Result<()> {
    let options = Options::parse()?;

    let cypher_text = match options.block_size() {
        BlockSizeOption::Eight => CypherText::decode(options.cypher_text(), &BlockSize::Eight)?,
        BlockSizeOption::Sixteen => CypherText::decode(options.cypher_text(), &BlockSize::Sixteen)?,
        BlockSizeOption::Auto => todo!(),
    };

    let question = Questioning::prepare(cypher_text)?.start(options.oracle_location());

    Ok(())
}
