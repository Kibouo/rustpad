use std::str::FromStr;

use anyhow::{anyhow, Result};

use crate::{block::Block, config::encoding_option::EncodingOption};

#[derive(Debug, Clone, Copy)]
pub(crate) enum Encoding {
    Hex,
    Base64,
    Base64Url,
}

pub(crate) trait Encode<'a> {
    type Blocks: IntoIterator<Item = &'a Block>;

    fn encode(&'a self) -> String;

    fn blocks(&'a self) -> Self::Blocks;
    fn url_encoded(&self) -> &bool;
    fn used_encoding(&self) -> &Encoding;
}

pub(crate) trait AmountBlocksTrait {
    fn amount_blocks(&self) -> usize;
}

impl FromStr for Encoding {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self> {
        if input == "hex" {
            Ok(Encoding::Hex)
        } else if input == "base64" {
            Ok(Encoding::Base64)
        } else if input == "base64url" {
            Ok(Encoding::Base64Url)
        } else {
            Err(anyhow!("Unknown encoding: {}", input))
        }
    }
}

impl TryFrom<&EncodingOption> for Encoding {
    type Error = anyhow::Error;

    fn try_from(encoding: &EncodingOption) -> Result<Self> {
        match encoding {
            EncodingOption::Hex => Ok(Self::Hex),
            EncodingOption::Base64 => Ok(Self::Base64),
            EncodingOption::Base64Url => Ok(Self::Base64Url),
            EncodingOption::Auto => Err(anyhow!(
                "`EncodingOption::Auto` cannot be converted into a specific `Encoding`"
            )),
        }
    }
}
