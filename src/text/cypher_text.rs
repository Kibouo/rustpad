use std::borrow::Cow;

use super::{block::Block, block_size::BlockSize};

use anyhow::{anyhow, Result};

#[derive(Debug)]
pub struct CypherText {
    blocks: Vec<Block>,
    url_encoded: bool,
    used_encoding: Encoding,
}

#[derive(Debug)]
enum Encoding {
    Base64,
    Base64Web,
    Hex,
}

impl CypherText {
    pub fn decode(input_data: &str, block_size: &BlockSize) -> Result<Self> {
        let url_decoded = urlencoding::decode(input_data).unwrap_or(Cow::Borrowed(input_data));

        let (decoded_data, used_encoding) = decode(input_data)?;
        let blocks = split_into_blocks(&decoded_data[..], *block_size)?;

        Ok(Self {
            blocks,
            url_encoded: input_data == url_decoded,
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
}

fn decode(input_data: &str) -> Result<(Vec<u8>, Encoding)> {
    if let Ok(decoded_data) = base64::decode_config(&*input_data, base64::STANDARD) {
        return Ok((decoded_data, Encoding::Base64));
    }

    if let Ok(decoded_data) = base64::decode_config(&*input_data, base64::URL_SAFE) {
        return Ok((decoded_data, Encoding::Base64Web));
    }

    if let Ok(decoded_data) = hex::decode(&*input_data) {
        return Ok((decoded_data, Encoding::Hex));
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
        .map(|chunk| match block_size {
            BlockSize::Eight => Block::Eight(chunk.try_into().unwrap_or_else(|_| {
                panic!(
                    "Not enough data to fill block of {}",
                    usize::from(block_size)
                )
            })),
            BlockSize::Sixteen => Block::Sixteen(chunk.try_into().unwrap_or_else(|_| {
                panic!(
                    "Not enough data to fill block of {}",
                    usize::from(block_size)
                )
            })),
        })
        .collect();

    Ok(blocks)
}
