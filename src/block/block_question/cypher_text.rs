use std::borrow::Cow;

use anyhow::{anyhow, Result};

use crate::block::{block_size::BlockSize, Block};

#[derive(Debug, Clone)]
pub struct CypherText {
    pub(super) blocks: Vec<Block>,
    url_encoded: bool,
    used_encoding: Encoding,
}

#[derive(Debug, Clone, Copy)]
enum Encoding {
    Base64,
    Base64Web,
    Hex,
}

impl CypherText {
    pub fn decode(input_data: &str, block_size: &BlockSize) -> Result<Self> {
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

    pub fn encode(&self) -> String {
        let raw_bytes: Vec<u8> = self
            .blocks
            .iter()
            .map(|block| &(**block)[..])
            .flatten()
            // blocks are scattered through memory, gotta collect them
            .cloned()
            .collect();

        let encoded_data = match self.used_encoding {
            Encoding::Base64 => base64::encode_config(raw_bytes, base64::STANDARD),
            Encoding::Base64Web => base64::encode_config(raw_bytes, base64::URL_SAFE),
            Encoding::Hex => hex::encode(raw_bytes),
        };

        if self.url_encoded {
            urlencoding::encode(&encoded_data).to_string()
        } else {
            encoded_data
        }
    }

    pub fn amount_blocks(&self) -> usize {
        self.blocks.len()
    }

    pub fn block_size(&self) -> BlockSize {
        self.blocks[0].block_size()
    }

    pub fn tweakable_block(&self) -> &Block {
        &self.blocks[self.amount_blocks() - 2]
    }

    pub fn tweakable_block_mut(&mut self) -> &mut Block {
        let idx = self.amount_blocks() - 2;
        &mut self.blocks[idx]
    }

    pub fn clone_with_blocks(&self, blocks: Vec<Block>) -> Self {
        Self {
            blocks,
            url_encoded: self.url_encoded,
            used_encoding: self.used_encoding,
        }
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
    if decoded_data.len() % *block_size != 0 {
        return Err(anyhow!(
            "Failed to split cypher text into blocks of size {}",
            *block_size
        ));
    }

    let blocks = decoded_data
        .chunks_exact(*block_size)
        .map(|chunk| Block::from((chunk, &block_size)))
        .collect();

    Ok(blocks)
}
