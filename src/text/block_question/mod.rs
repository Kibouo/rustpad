use std::ops::Deref;

use anyhow::{anyhow, Result};

use self::cypher_text::CypherText;

pub mod cypher_text;

#[derive(Debug, Clone)]
pub struct BlockQuestion {
    cypher_text: CypherText,
    current_byte_idx_back: Option<usize>,
}

impl BlockQuestion {
    pub fn increment_byte(&mut self) -> Result<&mut Self> {
        let byte_idx = self
            .current_byte_idx_back
            .ok_or_else(|| anyhow!("Can't increment byte after they're all locked"))?;

        let tweakable_block_idx = self.cypher_text.amount_blocks() - 2;
        self.cypher_text.blocks[tweakable_block_idx].increment_byte(byte_idx)?;

        Ok(self)
    }

    pub fn lock_byte(&mut self) -> Result<&mut Self> {
        match self.current_byte_idx_back {
            Some(idx) => {
                // PKCS5/7 padding's value is the same as its length. So the desired padding when testing for the last byte is 0x01. But when testing the 2nd last byte, the last byte must be 0x02. This means that when moving on to the next byte (right to left), all previous ones must be incremented by 1.
                let tweakable_block_idx = self.cypher_text.amount_blocks() - 2;
                (idx..usize::from(self.block_size()))
                    .map(|i| {
                        self.cypher_text.blocks[tweakable_block_idx]
                            .increment_byte(i)
                            .map(|_| ())
                    })
                    .collect::<Result<Vec<_>>>()?;

                if idx == 0 {
                    self.current_byte_idx_back = None;
                } else {
                    self.current_byte_idx_back = Some(idx - 1);
                }

                Ok(self)
            }
            None => Err(anyhow!(
                "Already locked all bytes! Current tweakable block layout: {:?}",
                self.cypher_text.blocks[self.cypher_text.amount_blocks() - 2]
            )),
        }
    }
}

impl From<CypherText> for BlockQuestion {
    fn from(cypher_text: CypherText) -> Self {
        let current_byte_idx_back = Some(usize::from(cypher_text.block_size()) - 1);

        Self {
            cypher_text,
            current_byte_idx_back,
        }
    }
}

impl Deref for BlockQuestion {
    type Target = CypherText;

    fn deref(&self) -> &Self::Target {
        &self.cypher_text
    }
}
