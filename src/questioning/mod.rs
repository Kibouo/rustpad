use anyhow::{Context, Result};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    block::block_size::BlockSizeTrait,
    cypher_text::{
        cypher_text::CypherText, forged_cypher_text::ForgedCypherText, AmountBlocksTrait,
    },
    oracle::{oracle_location::OracleLocation, script::ScriptOracle, web::WebOracle, Oracle},
};

/// Manages the oracle attack on a high level.
pub struct Questioning<'a> {
    forged_cypher_texts: Vec<ForgedCypherText<'a>>,
    original_cypher_text: CypherText,
}

impl<'a> Questioning<'a> {
    /// Divides the cypher text into a modifiable part for each block.
    pub fn prepare(cypher_text: CypherText) -> Result<Self> {
        // all blocks, except the 0-th which is the IV, are to be decrypted.
        // This could change with a "noiv" option
        let blocks_to_skip = 1;

        // decryption is based on recognizing padding. Padding is only at the end of a message. So to decrypt the n-th block, all blocks after it have to be dropped and the "n - 1"-th block must be tweaked.
        let mut forged_cypher_texts =
            Vec::with_capacity(cypher_text.amount_blocks() - blocks_to_skip);
        for block_to_decrypt_idx in blocks_to_skip..cypher_text.amount_blocks() {
            forged_cypher_texts.push(ForgedCypherText::from_part_of_cypher_text(
                &cypher_text,
                block_to_decrypt_idx - 1,
            )?);
        }

        Ok(Self {
            forged_cypher_texts,
            original_cypher_text: cypher_text,
        })
    }

    /// Actually performs the oracle attack for each block.
    pub fn start(&mut self, oracle_location: &OracleLocation) -> Result<String> {
        let oracle: Box<dyn Oracle> = match oracle_location {
            OracleLocation::Web(_) => Box::new(WebOracle::visit(oracle_location)?),
            OracleLocation::Script(_) => Box::new(ScriptOracle::visit(oracle_location)?),
        };

        self.forged_cypher_texts
            .iter_mut()
            .map(|forged_cypher_text| {
                let current_block_idx = forged_cypher_text.amount_blocks();

                let mut bytes_answered = 0;
                while bytes_answered < *forged_cypher_text.block_size() {
                    let mut current_byte_solution = (u8::MIN..u8::MAX)
                        .into_par_iter()
                        .map(|byte_value| -> Result<ForgedCypherText> {
                            let forged_cypher_text = forged_cypher_text.clone();

                            forged_cypher_text.set_current_byte(byte_value);
                            let correct_padding = oracle.ask_validation(&forged_cypher_text)?;

                            if correct_padding {
                                forged_cypher_text.lock_byte().map(|_| forged_cypher_text)
                            } else {
                                Err(anyhow::anyhow!(
                                    "Invalid padding for byte {} with block layout {:?}",
                                    *forged_cypher_text.block_size() - bytes_answered,
                                    forged_cypher_text.tweakable_block_solution()
                                ))
                            }
                        })
                        .filter(|potential_solution| potential_solution.is_ok())
                        .collect::<Result<Vec<_>>>()
                        .context(format!("Failed to decrypt block {}", current_block_idx))?;

                    match current_byte_solution.get_mut(0) {
                        Some(current_solution) => {
                            bytes_answered += 1;
                            forged_cypher_text = current_solution;
                        }
                        None => unreachable!("A solution for the current byte should exist at this point. We namely unpacked the `.collect::<Result<_>>()`")
                    }
                }

                (&mut forged_cypher_text).try_into()
            })
            .collect::<Result<_>>()
    }
}
