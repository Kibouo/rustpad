use std::mem;

use anyhow::{Context, Result};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    block::block_size::BlockSizeTrait,
    cypher_text::{encode::AmountBlocksTrait, forged_cypher_text::ForgedCypherText, CypherText},
    oracle::Oracle,
};

/// Manages the oracle attack on a high level.
pub struct Questioning<'a> {
    forged_cypher_texts: Vec<ForgedCypherText<'a>>,
}

impl<'a> Questioning<'a> {
    /// Divides the cypher text into a modifiable part for each block.
    pub fn prepare(cypher_text: &'a CypherText) -> Result<Self> {
        // all blocks, except the 0-th which is the IV, are to be decrypted.
        // This could change with a "noiv" option
        let blocks_to_skip = 1;

        // decryption is based on recognizing padding. Padding is only at the end of a message. So to decrypt the n-th block, all blocks after it have to be dropped and the "n - 1"-th block must be tweaked.
        let mut forged_cypher_texts =
            Vec::with_capacity(cypher_text.amount_blocks() - blocks_to_skip);
        for block_to_decrypt_idx in blocks_to_skip..cypher_text.amount_blocks() {
            forged_cypher_texts.push(ForgedCypherText::from_part_of_cypher_text(
                cypher_text,
                block_to_decrypt_idx,
            )?);
        }

        Ok(Self {
            forged_cypher_texts,
        })
    }

    /// Actually performs the oracle attack for each block.
    pub fn start(&mut self, oracle: impl Oracle) -> Result<String> {
        self.forged_cypher_texts
            .iter_mut()
            .map(|forged_cypher_text| {
                let current_block_idx = forged_cypher_text.amount_blocks();

                let mut bytes_answered = 0;
                while bytes_answered < *forged_cypher_text.block_size() {
                    let current_byte_solution = (u8::MIN..u8::MAX)
                        .into_par_iter()
                        .map(|byte_value| -> Result<ForgedCypherText> {
                            let mut forged_cypher_text = forged_cypher_text.clone();

                            forged_cypher_text.set_current_byte(byte_value)?;
                            let correct_padding = oracle.ask_validation(&forged_cypher_text)?;

                            if correct_padding {
                                forged_cypher_text.lock_byte()?;
                                Ok(forged_cypher_text)
                            } else {
                                Err(anyhow::anyhow!(
                                    "Invalid padding for byte {} with block layout {:?}",
                                    *forged_cypher_text.block_size() - bytes_answered,
                                    forged_cypher_text.tweakable_block_solution()
                                ))
                            }
                        })
                        .find_any(|potential_solution| potential_solution.is_ok());

                    match current_byte_solution {
                        Some(current_solution) => {
                            bytes_answered += 1;
                            mem::swap(
                                forged_cypher_text,
                                &mut current_solution.context(format!("Failed to decrypt block {}", current_block_idx))?
                            );
                        }
                        None => unreachable!("A solution for the current byte should exist at this point. We namely unpacked the `.collect::<Result<_>>()`")
                    }
                }

                forged_cypher_text.plaintext_block_solution()
            })
            .collect::<Result<_>>()
    }
}
