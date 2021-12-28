use std::str::FromStr;

use anyhow::{anyhow, Result};

use crate::block::Block;

#[derive(Debug, Clone, Copy)]
pub enum Encoding {
    Hex,
    Base64,
    Base64Url,
}

pub trait Encode<'a> {
    type Blocks: IntoIterator<Item = &'a Block>;

    fn encode(&'a self) -> String;

    fn blocks(&'a self) -> Self::Blocks;
    fn url_encoded(&self) -> &bool;
    fn used_encoding(&self) -> &Encoding;
}

pub trait AmountBlocksTrait {
    fn amount_blocks(&self) -> usize;
}

impl FromStr for Encoding {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self> {
        if input == "base64" {
            Ok(Encoding::Base64)
        } else if input == "base64url" {
            Ok(Encoding::Base64Url)
        } else if input == "hex" {
            Ok(Encoding::Hex)
        } else {
            Err(anyhow!("Unknown encoding: {}", input))
        }
    }
}
