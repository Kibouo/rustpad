pub mod block_size;

use std::ops::{BitXor, Deref, DerefMut};

use anyhow::{anyhow, Result};

use self::block_size::{BlockSize, BlockSizeTrait};

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

    pub fn new_incremental_padding(block_size: &BlockSize) -> Self {
        match block_size {
            BlockSize::Eight => Block::Eight([8, 7, 6, 5, 4, 3, 2, 1]),
            BlockSize::Sixteen => {
                Block::Sixteen([16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1])
            }
        }
    }

    pub fn set_byte(&mut self, index: usize, value: u8) -> Result<&mut Self> {
        match self {
            Block::Eight(data) => {
                if index < 8 {
                    data[index] += value;
                } else {
                    return Err(anyhow!(
                        "Tried to increment byte {} of 8-byte block",
                        index + 1
                    ));
                }
            }
            Block::Sixteen(data) => {
                if index < 16 {
                    data[index] = value;
                } else {
                    return Err(anyhow!(
                        "Tried to increment byte {} of 16-byte block",
                        index + 1
                    ));
                }
            }
        }

        Ok(self)
    }

    pub fn increment_byte(&mut self, idx: usize) -> Result<&mut Self> {
        match self {
            Block::Eight(data) => {
                if idx < 8 {
                    // error instead of wrap around to 0 as wrapping could introduce infinite loops
                    if data[idx] == u8::MAX {
                        return Err(anyhow!(
                            "Can't increment byte {} without overflowing",
                            idx + 1
                        ));
                    }

                    data[idx] += 1;
                } else {
                    return Err(anyhow!(
                        "Tried to increment byte {} of 8-byte block",
                        idx + 1
                    ));
                }
            }
            Block::Sixteen(data) => {
                if idx < 16 {
                    // error instead of wrap around to 0 as wrapping could introduce infinite loops
                    if data[idx] == u8::MAX {
                        return Err(anyhow!(
                            "Can't increment byte {} without overflowing",
                            idx + 1
                        ));
                    }

                    data[idx] += 1;
                } else {
                    return Err(anyhow!(
                        "Tried to increment byte {} of 16-byte block",
                        idx + 1
                    ));
                }
            }
        }

        Ok(self)
    }

    // adjust bytes to produce the new correct padding.
    // Due to xor's working, this cannot be done as a simple +1 in byte value. We must use xor's commutative property.
    pub fn adjust_for_incremented_padding(&mut self, new_pad_size: u8) -> &mut Self {
        let old_pad_size = new_pad_size - 1;
        let xor_diff = old_pad_size ^ new_pad_size;

        for i in self.len() - (old_pad_size as usize)..self.len() {
            self[i] ^= xor_diff;
        }
        self
    }

    pub fn into_string(&self) -> String {
        self.iter().map(|byte_value| *byte_value as char).collect()
    }
}

impl BlockSizeTrait for Block {
    fn block_size(&self) -> BlockSize {
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

        Ok(xored_bytes[..].into())
    }
}

impl From<&[u8]> for Block {
    fn from(chunk_data: &[u8]) -> Self {
        let block_size = chunk_data.len().into();
        match block_size {
            BlockSize::Eight => Block::Eight(
                chunk_data
                    .try_into()
                    .unwrap_or_else(|_| panic!("Not enough data to fill block of {}", *block_size)),
            ),
            BlockSize::Sixteen => Block::Sixteen(
                chunk_data
                    .try_into()
                    .unwrap_or_else(|_| panic!("Not enough data to fill block of {}", *block_size)),
            ),
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
