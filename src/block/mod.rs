pub mod block_answer;
pub mod block_question;
pub mod block_size;

use std::ops::{BitXor, Deref, DerefMut};

use anyhow::{anyhow, Result};

use self::block_size::BlockSize;

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

    pub fn incremental_padding(block_size: &BlockSize) -> Self {
        match block_size {
            BlockSize::Eight => Block::Eight([8, 7, 6, 5, 4, 3, 2, 1]),
            BlockSize::Sixteen => {
                Block::Sixteen([16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1])
            }
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

    pub fn block_size(&self) -> BlockSize {
        BlockSize::from(self)
    }
}

impl BitXor for &Block {
    type Output = Result<Block>;

    fn bitxor(self, rhs: Self) -> Self::Output {
        if *self.block_size() != *rhs.block_size() {
            return Err(anyhow!(
                "Can't XOR blocks of size {} and {}",
                *self.block_size(),
                *rhs.block_size()
            ));
        }

        let xored_bytes: Vec<u8> = self
            .deref()
            .iter()
            .zip(rhs.deref().iter())
            .into_iter()
            .map(|(l, r)| l ^ r)
            .collect();

        Ok((&xored_bytes[..], &self.block_size()).into())
    }
}

impl From<(&[u8], &BlockSize)> for Block {
    fn from((chunk_data, block_size): (&[u8], &BlockSize)) -> Self {
        match block_size {
            BlockSize::Eight => {
                Block::Eight(chunk_data.try_into().unwrap_or_else(|_| {
                    panic!("Not enough data to fill block of {}", **block_size)
                }))
            }
            BlockSize::Sixteen => {
                Block::Sixteen(chunk_data.try_into().unwrap_or_else(|_| {
                    panic!("Not enough data to fill block of {}", **block_size)
                }))
            }
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
