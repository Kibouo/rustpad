use std::borrow::Cow;

use anyhow::{anyhow, Result};

use crate::text::{block::Block, block_size::BlockSize};

use super::BlockQuestion;

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

    pub fn to_block_question(&self, tweakable_block_idx: usize) -> Result<BlockQuestion> {
        if tweakable_block_idx + 2 > self.amount_blocks() {
            return Err(anyhow!(
                "Can't create a BlockQuestion for block {}, with only {} blocks existing",
                tweakable_block_idx + 2, // +1 to specify target block, +1 for 1-indexing
                self.amount_blocks()
            ));
        }

        let prefix_blocks = self.blocks[..tweakable_block_idx].iter();
        // already init the tweakable block with 0's
        let tweakable_block = &Block::new(&self.block_size());
        let to_decrypt_block = &self.blocks[tweakable_block_idx + 1];

        let blocks = prefix_blocks
            .chain([tweakable_block].into_iter())
            .chain([to_decrypt_block].into_iter())
            .into_iter()
            .cloned()
            .collect();

        Ok(Self {
            blocks,
            url_encoded: self.url_encoded,
            used_encoding: self.used_encoding,
        }
        .into())
    }

    pub fn block_size(&self) -> BlockSize {
        BlockSize::from(&self.blocks[0])
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
    if decoded_data.len() % usize::from(block_size) != 0 {
        return Err(anyhow!(
            "Failed to split cypher text into blocks of size {}",
            Into::<usize>::into(block_size)
        ));
    }

    let blocks = decoded_data
        .chunks_exact(usize::from(block_size))
        .map(|chunk| Block::from((chunk, &block_size)))
        .collect();

    Ok(blocks)
}
