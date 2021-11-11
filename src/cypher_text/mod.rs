pub mod cypher_text;
pub mod forged_cypher_text;

use crate::block::Block;

#[derive(Debug, Clone, Copy)]
enum Encoding {
    Base64,
    Base64Web,
    Hex,
}

pub trait Encode<'a> {
    type Blocks: IntoIterator<Item = &'a Block>;

    fn encode(&'a self) -> String {
        let raw_bytes: Vec<u8> = self
            .blocks()
            .into_iter()
            .map(|block| &**block)
            .flatten()
            // blocks are scattered through memory, gotta collect them
            .cloned()
            .collect();

        let encoded_data = match self.used_encoding() {
            Encoding::Base64 => base64::encode_config(raw_bytes, base64::STANDARD),
            Encoding::Base64Web => base64::encode_config(raw_bytes, base64::URL_SAFE),
            Encoding::Hex => hex::encode(raw_bytes),
        };

        if self.url_encoded() {
            urlencoding::encode(&encoded_data).to_string()
        } else {
            encoded_data
        }
    }

    fn blocks(&'a self) -> Self::Blocks;
    fn url_encoded(&self) -> bool;
    fn used_encoding(&self) -> Encoding;
}

pub trait AmountBlocksTrait {
    fn amount_blocks(&self) -> usize;
}
