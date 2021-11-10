use std::ops::Deref;

use super::Block;

#[derive(Clone, Copy)]
pub enum BlockSize {
    Eight,
    Sixteen,
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
    type Target = usize;

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
