use std::{ops::Deref, str::FromStr};

use anyhow::{anyhow, Result};
use itertools::Itertools;

use super::Block;

#[derive(Clone, Copy, Debug)]
pub(crate) enum BlockSize {
    Eight,
    Sixteen,
}

pub(crate) trait BlockSizeTrait {
    fn block_size(&self) -> BlockSize;
}

impl BlockSize {
    fn variants() -> &'static [Self] {
        &[BlockSize::Eight, BlockSize::Sixteen]
    }
}

impl From<u8> for BlockSize {
    fn from(data: u8) -> Self {
        match data {
            8 => Self::Eight,
            16 => Self::Sixteen,
            _ => unreachable!(format!("Invalid block size: {}", data)),
        }
    }
}

impl From<usize> for BlockSize {
    fn from(data: usize) -> Self {
        match data {
            8 => Self::Eight,
            16 => Self::Sixteen,
            _ => unreachable!(format!("Invalid block size: {}", data)),
        }
    }
}

impl FromStr for BlockSize {
    type Err = anyhow::Error;

    fn from_str(data: &str) -> Result<Self> {
        match data {
            "8" => Ok(Self::Eight),
            "16" => Ok(Self::Sixteen),
            _ => Err(anyhow!(
                "`{}` is an invalid block size. Expected one of: [{}]",
                data,
                Self::variants()
                    .iter()
                    .map(|variant| variant.to_string())
                    .join(", ")
            )),
        }
    }
}

impl From<&Block> for BlockSize {
    fn from(block: &Block) -> Self {
        match block {
            Block::Eight(_) => Self::Eight,
            Block::Sixteen(_) => Self::Sixteen,
        }
    }
}

impl Deref for BlockSize {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        match self {
            BlockSize::Eight => &8,
            BlockSize::Sixteen => &16,
        }
    }
}
