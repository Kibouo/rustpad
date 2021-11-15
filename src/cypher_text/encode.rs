use crate::block::Block;

use anyhow::Result;

#[derive(Debug, Clone, Copy)]
pub enum Encoding {
    Base64,
    Base64Web,
    Hex,
}

pub trait Encode<'a> {
    type Blocks: IntoIterator<Item = &'a Block>;

    fn encode(&'a self) -> Result<String>;

    fn blocks(&'a self) -> Self::Blocks;
    fn url_encoded(&self) -> bool;
    fn used_encoding(&self) -> Encoding;
}

pub trait AmountBlocksTrait {
    fn amount_blocks(&self) -> usize;
}
