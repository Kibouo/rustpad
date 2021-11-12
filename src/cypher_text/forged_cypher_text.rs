use anyhow::{anyhow, Result};

use crate::block::block_size::{BlockSize, BlockSizeTrait};

use super::{AmountBlocksTrait, Block, CypherText, Encode, Encoding};

#[derive(Debug, Clone)]
pub struct ForgedCypherText<'a> {
    original_blocks: &'a [Block],
    url_encoded: bool,
    used_encoding: Encoding,

    current_byte_idx: Option<u8>,
    tweakable_block_wip: Block,
    tweakable_block_solution: Block,
}

impl<'a> ForgedCypherText<'a> {
    pub fn from_part_of_cypher_text(
        cypher_text: &'a CypherText,
        block_to_decrypt: usize,
    ) -> Result<Self> {
        if block_to_decrypt + 1 > cypher_text.amount_blocks() {
            return Err(anyhow!(
                "Can't create a ForgedCypherText for block {}, with only {} blocks existing in the original",
                block_to_decrypt + 1,
                cypher_text.amount_blocks()
            ));
        }

        let original_blocks = &cypher_text.blocks()[..block_to_decrypt + 1];

        let block_size = cypher_text.block_size();
        let forged_cypher_text = Self {
            original_blocks,
            url_encoded: cypher_text.url_encoded(),
            used_encoding: cypher_text.used_encoding(),
            current_byte_idx: Some(*block_size - 1),
            tweakable_block_wip: Block::new(&cypher_text.block_size()),
            tweakable_block_solution: Block::new(&cypher_text.block_size()),
        };

        Ok(forged_cypher_text)
    }

    pub fn set_current_byte(&mut self, value: u8) -> Result<&mut Self> {
        let byte_idx = self
            .current_byte_idx
            .ok_or_else(|| anyhow!("Can't change bytes after they're all locked"))?;

        self.tweakable_block_wip
            .set_byte(byte_idx as usize, value)?;

        Ok(self)
    }

    /// Indicate that the current byte's value was found. Advance and save the solution.
    pub fn lock_byte(&mut self) -> Result<&mut Self> {
        match self.current_byte_idx {
            Some(idx) => {
                // locking a byte means it's supposedly correct. Because it gets adjusted, see below, we gotta save the solution
                self.tweakable_block_solution[idx as usize] =
                    self.tweakable_block_wip[idx as usize];

                if idx == 0 {
                    self.current_byte_idx = None;
                } else {
                    self.current_byte_idx = Some(idx - 1);

                    // PKCS5/7 padding's value is the same as its length. So the desired padding when testing for the last byte is 0x01. But when testing the 2nd last byte, the last byte must be 0x02. This means that when moving on to the next byte (right to left), all of the previous bytes' solutions must be adjusted.
                    let block_size = *self.block_size();
                    self.tweakable_block_wip
                        .adjust_for_incremented_padding(block_size - (idx - 1));
                }

                Ok(self)
            }
            None => Err(anyhow!(
                "Already locked all bytes! Current tweakable block layout: {:?}",
                self.tweakable_block_wip
            )),
        }
    }

    pub fn tweakable_block_solution(&self) -> &Block {
        &self.tweakable_block_solution
    }

    pub fn plaintext_block_solution(&self) -> Result<String> {
        if !self.is_answered() {
            return Err(anyhow!(
                "Can't compute plaintext. Not all bytes of the tweakable block were locked"
            ));
        }

        let intermediate =
            &self.tweakable_block_solution ^ &Block::new_incremental_padding(&self.block_size());
        let intermediate = intermediate?;

        let plaintext = &intermediate ^ self.original_tweakable_block();
        let plaintext = plaintext?;

        Ok(plaintext.to_string())
    }

    fn is_answered(&self) -> bool {
        self.current_byte_idx.is_none()
    }

    fn original_tweakable_block(&self) -> &Block {
        &self.original_blocks[self.amount_blocks() - 2]
    }
}

impl<'a> Encode<'a> for ForgedCypherText<'a> {
    type Blocks = &'a [Block];

    fn encode(&'a self) -> String {
        let prefix_blocks = &self.blocks()[..self.amount_blocks() - 2];
        let to_decrypt_block = &self.blocks()[self.amount_blocks() - 1];

        let raw_bytes: Vec<u8> = prefix_blocks.iter()
            .chain([&self.tweakable_block_wip].into_iter())
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
            urlencoding::encode(&encoded_data).to_string()
        } else {
            encoded_data
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
