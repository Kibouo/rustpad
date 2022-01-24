use getset::Getters;
use itertools::Itertools;

use crate::{
    block::{
        block_size::{BlockSize, BlockSizeTrait},
        Block,
    },
    cypher_text::encode::AmountBlocksTrait,
};

/// PKCS7 padded plain text.
#[derive(Debug, Getters)]
pub(super) struct PlainText {
    #[getset(get = "pub(super)")]
    blocks: Vec<Block>,
}

impl PlainText {
    pub(super) fn new(input_data: &str, block_size: &BlockSize) -> Self {
        let block_size = **block_size as usize;
        let padding_size = block_size - input_data.len() % block_size;

        let padded_blocks = input_data
            .as_bytes()
            .iter()
            .cloned()
            .pad_using(input_data.len() + padding_size, |_| padding_size as u8)
            .chunks(block_size)
            .into_iter()
            .map(|chunk| Block::from(&chunk.collect::<Vec<_>>()[..]))
            .collect();

        Self {
            blocks: padded_blocks,
        }
    }
}

impl AmountBlocksTrait for PlainText {
    fn amount_blocks(&self) -> usize {
        self.blocks.len()
    }
}

impl BlockSizeTrait for PlainText {
    fn block_size(&self) -> BlockSize {
        self.blocks()[0].block_size()
    }
}
