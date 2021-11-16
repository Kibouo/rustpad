use anyhow::{anyhow, Result};
use getset::Getters;

use crate::block::block_size::{BlockSize, BlockSizeTrait};

use super::{AmountBlocksTrait, Block, CypherText, Encode, Encoding};

#[derive(Debug, Clone, Getters)]
pub struct ForgedCypherText<'a> {
    original_blocks: &'a [Block],
    url_encoded: bool,
    used_encoding: Encoding,

    current_byte_idx: Option<u8>,
    #[getset(get = "pub")]
    forged_block_wip: Block,
    #[getset(get = "pub")]
    forged_block_solution: Block,
}

impl<'a> ForgedCypherText<'a> {
    pub fn from_part_of_cypher_text(
        cypher_text: &'a CypherText,
        block_to_decrypt_idx: usize,
    ) -> Result<Self> {
        if block_to_decrypt_idx + 1 > cypher_text.amount_blocks() {
            return Err(anyhow!(
                "Tried to create ForgedCypherText for block {}, with only {} blocks existing in the original",
                block_to_decrypt_idx + 1,
                cypher_text.amount_blocks()
            ));
        }

        let original_blocks = &cypher_text.blocks()[..block_to_decrypt_idx + 1];

        let block_size = cypher_text.block_size();
        let forged_cypher_text = Self {
            original_blocks,
            url_encoded: cypher_text.url_encoded(),
            used_encoding: cypher_text.used_encoding(),
            current_byte_idx: Some(*block_size - 1),
            forged_block_wip: Block::new(&cypher_text.block_size()),
            forged_block_solution: Block::new(&cypher_text.block_size()),
        };

        Ok(forged_cypher_text)
    }

    pub fn set_current_byte(&mut self, value: u8) -> Result<&mut Self> {
        let byte_idx = self
            .current_byte_idx
            .ok_or_else(|| anyhow!("Changing bytes failed. They're all locked already"))?;

        self.forged_block_wip.set_byte(byte_idx as usize, value)?;

        Ok(self)
    }

    /// Indicate that the current byte's value was found. Advance and save the solution.
    pub fn lock_byte(&mut self) -> Result<&mut Self> {
        match self.current_byte_idx {
            Some(idx) => {
                // locking a byte means it's supposedly correct. Because it gets adjusted, see below, we gotta save the solution
                self.forged_block_solution[idx as usize] = self.forged_block_wip[idx as usize];

                if idx == 0 {
                    self.current_byte_idx = None;
                } else {
                    self.current_byte_idx = Some(idx - 1);
                }

                Ok(self)
            }
            None => Err(anyhow!(
                "Locked all bytes already! Current forged block layout: {:?}",
                self.forged_block_wip
            )),
        }
    }

    pub fn plain_text_solution(&self) -> Result<String> {
        if !self.is_answered() {
            return Err(anyhow!(
                "Plain text computation failed. Not all bytes of the forged block were locked"
            ));
        }

        let intermediate =
            &self.forged_block_solution ^ &Block::new_incremental_padding(&self.block_size());
        let plain_text = &intermediate ^ self.original_forged_block();

        Ok(plain_text.to_string())
    }

    fn is_answered(&self) -> bool {
        self.current_byte_idx.is_none()
    }

    fn original_forged_block(&self) -> &Block {
        &self.original_blocks[self.amount_blocks() - 2]
    }
}

impl<'a> Encode<'a> for ForgedCypherText<'a> {
    type Blocks = &'a [Block];

    fn encode(&'a self) -> Result<String> {
        let prefix_blocks = &self.blocks()[..self.amount_blocks() - 2];
        let to_decrypt_block = &self.blocks()[self.amount_blocks() - 1];

        // PKCS5/7 padding's value is the same as its length. So the desired padding when testing for the last byte is 0x01. But when testing the 2nd last byte, the last byte must be 0x02. This means that when moving on to the next byte (right to left), all of the previous bytes' solutions must be adjusted.
        let current_byte_idx = self.current_byte_idx.ok_or_else(|| {
            anyhow!(
                "Locked all bytes already! Current forged block layout: {:?}",
                self.forged_block_wip
            )
        })?;
        let forged_block_with_padding_adjusted = self
            .forged_block_wip
            .to_adjusted_for_padding(*self.block_size() - current_byte_idx as u8);

        let raw_bytes: Vec<u8> = prefix_blocks.iter()
            .chain([&forged_block_with_padding_adjusted].into_iter())
            .chain([to_decrypt_block].into_iter())
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
            Ok(urlencoding::encode(&encoded_data).to_string())
        } else {
            Ok(encoded_data)
        }
    }

    fn blocks(&'a self) -> Self::Blocks {
        self.original_blocks
    }

    fn url_encoded(&self) -> bool {
        self.url_encoded
    }

    fn used_encoding(&self) -> Encoding {
        self.used_encoding
    }
}

impl<'a> BlockSizeTrait for ForgedCypherText<'a> {
    fn block_size(&self) -> BlockSize {
        self.blocks()[0].block_size()
    }
}

impl<'a> AmountBlocksTrait for ForgedCypherText<'a> {
    fn amount_blocks(&self) -> usize {
        self.blocks().len()
    }
}
