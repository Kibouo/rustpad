use std::ops::Deref;

use anyhow::{anyhow, Result};

use self::cypher_text::CypherText;

use super::Block;

pub mod cypher_text;

#[derive(Debug, Clone)]
pub struct BlockQuestion {
    cypher_text: CypherText,
    current_byte_idx: Option<u8>,
    original_tweakable_block: Block,
    tweakable_block_solution: Block,
}

impl BlockQuestion {
    pub fn clone_part_of_cypher_text(
        cypher_text: &CypherText,
        tweakable_block_idx: usize,
    ) -> Result<Self> {
        if tweakable_block_idx + 2 > cypher_text.amount_blocks() {
            return Err(anyhow!(
                "Can't create a BlockQuestion for block {}, with only {} blocks existing",
                tweakable_block_idx + 2, // +1 to specify target block, +1 for 1-indexing
                cypher_text.amount_blocks()
            ));
        }

        // don't use `CypherText::tweakable_block` here as that would get the tweakable block of the original cypher text. Meanwhile this splits off a part from the original such that we can solve a specific block
        let original_tweakable_block = cypher_text.blocks[tweakable_block_idx].clone();

        let prefix_blocks = cypher_text.blocks[..tweakable_block_idx].iter();
        // already init the tweakable block with 0's
        let tweakable_block = &Block::new(&cypher_text.block_size());
        let to_decrypt_block = &cypher_text.blocks[tweakable_block_idx + 1];

        let blocks = prefix_blocks
            .chain([tweakable_block].into_iter())
            .chain([to_decrypt_block].into_iter())
            .into_iter()
            .cloned()
            .collect();

        let cypher_text = cypher_text.clone_with_blocks(blocks);
        let current_byte_idx = Some(*cypher_text.block_size() - 1);
        let tweakable_block_solution = Block::new(&cypher_text.block_size());

        let block_question = Self {
            cypher_text,
            current_byte_idx,
            original_tweakable_block,
            tweakable_block_solution,
        };

        Ok(block_question)
    }

    pub fn increment_current_byte(&mut self) -> Result<&mut Self> {
        let byte_idx = self
            .current_byte_idx
            .ok_or_else(|| anyhow!("Can't increment byte after they're all locked"))?;

        self.cypher_text
            .tweakable_block_mut()
            .increment_byte(byte_idx as usize)?;

        Ok(self)
    }

    /// Indicate that the current byte's value was found. Advance and save the solution.
    pub fn lock_byte(&mut self) -> Result<&mut Self> {
        match self.current_byte_idx {
            Some(idx) => {
                // locking a byte means it's supposedly correct. Because it gets adjusted, see below, we gotta save the solution
                self.tweakable_block_solution[idx as usize] =
                    self.cypher_text.tweakable_block()[idx as usize];

                if idx == 0 {
                    self.current_byte_idx = None;
                } else {
                    self.current_byte_idx = Some(idx - 1);

                    // PKCS5/7 padding's value is the same as its length. So the desired padding when testing for the last byte is 0x01. But when testing the 2nd last byte, the last byte must be 0x02. This means that when moving on to the next byte (right to left), all of the previous bytes' solutions must be adjusted.
                    let block_size = *self.cypher_text.block_size();
                    self.cypher_text
                        .tweakable_block_mut()
                        .adjust_for_incremented_padding(block_size - (idx - 1));
                }

                Ok(self)
            }
            None => Err(anyhow!(
                "Already locked all bytes! Current tweakable block layout: {:?}",
                self.cypher_text.tweakable_block()
            )),
        }
    }

    pub fn tweakable_block_solution(&self) -> &Block {
        &self.tweakable_block_solution
    }

    pub fn original_tweakable_block(&self) -> &Block {
        &self.original_tweakable_block
    }
}

impl Deref for BlockQuestion {
    type Target = CypherText;

    fn deref(&self) -> &Self::Target {
        &self.cypher_text
    }
}
