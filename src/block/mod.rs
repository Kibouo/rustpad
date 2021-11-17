pub mod block_size;

use std::{
    fmt::Display,
    ops::{BitXor, Deref, DerefMut},
};

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

    pub fn set_byte(&mut self, index: usize, value: u8) -> &mut Self {
        match self {
            Block::Eight(data) => {
                if index < 8 {
                    data[index] += value;
                } else {
                    panic!(
                        "Tried to increment byte at index {} of 8-byte block",
                        index + 1
                    );
                }
            }
            Block::Sixteen(data) => {
                if index < 16 {
                    data[index] = value;
                } else {
                    panic!(
                        "Tried to increment byte at index {} of 16-byte block",
                        index + 1
                    );
                }
            }
        }

        self
    }

    /// Clone this block and adjusts bytes to produce the correct padding
    /// Due to xor's working, this cannot be done as a simple +1 in byte value. We must use xor's commutative property.
    pub fn to_adjusted_for_padding(&self, pad_size: u8) -> Self {
        let mut adjusted_block = self.clone();

        for i in self.len() - (pad_size as usize)..self.len() {
            adjusted_block[i] ^= (self.len() - i) as u8; // get actual padding out
            adjusted_block[i] ^= pad_size; // put WIP padding in
        }

        adjusted_block
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&**self)
    }

    pub fn to_ascii(&self) -> String {
        self.iter()
            .map(|byte_value| *byte_value as char)
            .map(|c| {
                if !c.is_ascii() || c.is_ascii_control() {
                    '.'
                } else {
                    c
                }
            })
            .collect::<String>()
    }

    pub fn to_intermediate(&self) -> Block {
        self ^ &Block::new_incremental_padding(&self.block_size())
    }
}

impl BlockSizeTrait for Block {
    fn block_size(&self) -> BlockSize {
        BlockSize::from(self)
    }
}

impl BitXor for &Block {
    type Output = Block;

    fn bitxor(self, rhs: Self) -> Self::Output {
        if *self.block_size() != *rhs.block_size() {
            panic!(
                "Can't XOR blocks of size {} and {}",
                *self.block_size(),
                *rhs.block_size()
            );
        }

        let xored_bytes: Vec<u8> = self
            .deref()
            .iter()
            .zip(rhs.deref().iter())
            .into_iter()
            .map(|(l, r)| l ^ r)
            .collect();

        xored_bytes[..].into()
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

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.iter()
                .map(|byte_value| *byte_value as char)
                .collect::<String>()
        )
    }
}
