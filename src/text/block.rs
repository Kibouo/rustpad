use std::ops::{Deref, DerefMut};

use anyhow::{anyhow, Result};

use super::block_size::BlockSize;

#[derive(Debug, Clone)]
pub enum Block {
    Eight([u8; 8]),
    Sixteen([u8; 16]),
}

impl Block {
    pub fn new(block_size: &BlockSize) -> Self {
        match block_size {
            BlockSize::Eight => Block::Eight([0; 8]),
            BlockSize::Sixteen => Block::Sixteen([0; 16]),
        }
    }

    pub fn increment_byte(&mut self, idx: usize) -> Result<&mut Self> {
        match self {
            Block::Eight(data) => {
                if idx < 8 {
                    data[idx] += 1;
                } else {
                    return Err(anyhow!("Tried to increment byte {} of 8-byte block", idx));
                }
            }
            Block::Sixteen(data) => {
                if idx < 16 {
                    data[idx] += 1;
                } else {
                    return Err(anyhow!("Tried to increment byte {} of 16-byte block", idx));
                }
            }
        }

        Ok(self)
    }
}

impl From<(&[u8], &BlockSize)> for Block {
    fn from((chunk_data, block_size): (&[u8], &BlockSize)) -> Self {
        match block_size {
            BlockSize::Eight => Block::Eight(chunk_data.try_into().unwrap_or_else(|_| {
                panic!(
                    "Not enough data to fill block of {}",
                    usize::from(*block_size)
                )
            })),
            BlockSize::Sixteen => Block::Sixteen(chunk_data.try_into().unwrap_or_else(|_| {
                panic!(
                    "Not enough data to fill block of {}",
                    usize::from(*block_size)
                )
            })),
        }
    }
}

impl Deref for Block {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            Block::Eight(data) => data,
            Block::Sixteen(data) => data,
        }
    }
}

impl DerefMut for Block {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Block::Eight(data) => data,
            Block::Sixteen(data) => data,
        }
    }
}
