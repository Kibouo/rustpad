use std::ops::Deref;

use super::Block;

#[derive(Clone, Copy)]
pub enum BlockSize {
    Eight,
    Sixteen,
}

pub trait BlockSizeTrait {
    fn block_size(&self) -> BlockSize;
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

impl From<&str> for BlockSize {
    fn from(data: &str) -> Self {
        match data {
            "8" => Self::Eight,
            "16" => Self::Sixteen,
            _ => unreachable!(format!("Invalid block size: {}", data)),
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

impl From<&Block> for BlockSize {
    fn from(block: &Block) -> Self {
        match block {
            Block::Eight(_) => Self::Eight,
            Block::Sixteen(_) => Self::Sixteen,
        }
    }
}
