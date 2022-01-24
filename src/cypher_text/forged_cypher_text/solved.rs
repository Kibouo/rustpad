use getset::Getters;

use crate::{
    block::Block,
    cypher_text::encode::{AmountBlocksTrait, Encoding},
};

use super::ForgedCypherText;

#[derive(Getters, Clone, Debug)]
pub(crate) struct SolvedForgedCypherText<'a> {
    #[getset(get = "pub(crate)")]
    original_blocks: &'a [Block],
    #[getset(get = "pub(crate)")]
    url_encoded: bool,
    #[getset(get = "pub(crate)")]
    used_encoding: Encoding,

    #[getset(get = "pub(crate)")]
    forged_block_solution: Block,
}

impl<'a> SolvedForgedCypherText<'a> {
    pub(crate) fn plain_text_solution(&self) -> String {
        let plain_text =
            &self.forged_block_solution.to_intermediate() ^ self.original_forged_block();

        plain_text.to_string()
    }

    pub(crate) fn block_to_decrypt(&self) -> &Block {
        &self.original_blocks[self.amount_blocks() - 1]
    }

    fn original_forged_block(&self) -> &Block {
        // -1 for 0-idx, and another -1 so we get the original of the forged block
        &self.original_blocks[self.amount_blocks() - 2]
    }
}

impl<'a> From<ForgedCypherText<'a>> for SolvedForgedCypherText<'a> {
    fn from(forged_cypher_text: ForgedCypherText<'a>) -> Self {
        Self {
            original_blocks: forged_cypher_text.original_blocks,
            url_encoded: forged_cypher_text.url_encoded,
            used_encoding: forged_cypher_text.used_encoding,

            forged_block_solution: forged_cypher_text.forged_block_solution,
        }
    }
}

/// from cached Block
impl<'a> From<(ForgedCypherText<'a>, Block)> for SolvedForgedCypherText<'a> {
    fn from((forged_cypher_text, forged_block_solution): (ForgedCypherText<'a>, Block)) -> Self {
        Self {
            original_blocks: forged_cypher_text.original_blocks,
            url_encoded: forged_cypher_text.url_encoded,
            used_encoding: forged_cypher_text.used_encoding,

            forged_block_solution,
        }
    }
}

impl<'a> AmountBlocksTrait for SolvedForgedCypherText<'a> {
    fn amount_blocks(&self) -> usize {
        self.original_blocks().len()
    }
}
