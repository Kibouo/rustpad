use crate::block::{block_size::BlockSizeTrait, Block};
use std::borrow::Cow;

use anyhow::{anyhow, Result};

use crate::block::block_size::BlockSize;

use super::{AmountBlocksTrait, Encode, Encoding};

#[derive(Debug, Clone)]
pub struct CypherText {
    blocks: Vec<Block>,
    url_encoded: bool,
    used_encoding: Encoding,
}

impl CypherText {
    pub fn parse(input_data: &str, block_size: &BlockSize) -> Result<Self> {
        // url decode if needed
        let url_decoded = urlencoding::decode(input_data).unwrap_or(Cow::Borrowed(input_data));

        let (decoded_data, used_encoding) = decode(&url_decoded)?;
        let blocks = split_into_blocks(&decoded_data[..], *block_size)?;

        Ok(Self {
            blocks,
            url_encoded: input_data != url_decoded,
            used_encoding,
        })
    }
}

impl<'a> Encode<'a> for CypherText {
    type Blocks = &'a [Block];

    fn blocks(&'a self) -> Self::Blocks {
        &self.blocks[..]
    }

    fn url_encoded(&self) -> bool {
        self.url_encoded
    }

    fn used_encoding(&self) -> Encoding {
        self.used_encoding
    }
}

impl BlockSizeTrait for CypherText {
    fn block_size(&self) -> BlockSize {
        self.blocks()[0].block_size()
    }
}

impl AmountBlocksTrait for CypherText {
    fn amount_blocks(&self) -> usize {
        self.blocks().len()
    }
}

// auto-detect encoding & decode it
fn decode(input_data: &str) -> Result<(Vec<u8>, Encoding)> {
    if let Ok(decoded_data) = hex::decode(&*input_data) {
        return Ok((decoded_data, Encoding::Hex));
    }

    if let Ok(decoded_data) = base64::decode_config(&*input_data, base64::STANDARD) {
        return Ok((decoded_data, Encoding::Base64));
    }

    if let Ok(decoded_data) = base64::decode_config(&*input_data, base64::URL_SAFE) {
        return Ok((decoded_data, Encoding::Base64Web));
    }

    Err(anyhow!(
        "{} is not valid base64, base64 (web-safe), or hex",
        input_data
    ))
}

fn split_into_blocks(decoded_data: &[u8], block_size: BlockSize) -> Result<Vec<Block>> {
    if decoded_data.len() % (*block_size as usize) != 0 {
        return Err(anyhow!(
            "Failed to split cypher text into blocks of size {}",
            *block_size
        ));
    }

    let blocks = decoded_data
        .chunks_exact(*block_size as usize)
        .map(|chunk| chunk.into())
        .collect();

    Ok(blocks)
}