use anyhow::Result;
use log::{debug, info};

use crate::{
    block::{block_size::BlockSizeTrait, Block},
    cypher_text::{
        encode::AmountBlocksTrait,
        forged_cypher_text::{solved::SolvedForgedCypherText, ForgedCypherText},
        CypherText,
    },
    divination::solve_block,
    logging::LOG_TARGET,
    oracle::Oracle,
    plain_text::PlainText,
    tui::ui_event::{UiControlEvent, UiEncryptionEvent, UiEvent},
};

/// Manages the oracle attack (encryption) on a high level.
pub struct Encryptor<'a, U>
where
    U: FnMut(UiEvent) + Sync + Send + Clone,
{
    // intermediate of last block of the user provided cypher text
    initial_block: SolvedForgedCypherText<'a>,
    update_ui_callback: U,
}

impl<'a, U> Encryptor<'a, U>
where
    U: FnMut(UiEvent) + Sync + Send + Clone,
{
    pub fn new(update_ui_callback: U, initial_block: SolvedForgedCypherText<'a>) -> Self {
        debug!(target: LOG_TARGET, "Preparing to encrypt plain text",);

        Self {
            initial_block,
            update_ui_callback,
        }
    }

    pub fn encrypt_plain_text(
        &self,
        plain_text: &PlainText,
        oracle: &impl Oracle,
    ) -> Result<CypherText> {
        let mut encrypted_blocks_backwards = vec![self.initial_block.block_to_decrypt().clone()];

        // encrypting requires iteratively, backwards, building up the cypher text. Unlike with decrypting, we don't know each block beforehand. So obviously this can't be done in parallel
        for (i, plain_text_block) in plain_text.blocks().iter().rev().enumerate() {
            let intermediate = if i == 0 {
                let block_solution = self.initial_block.forged_block_solution();

                // encryption of this block is already finished as it's identical to decryption of the initial block
                (self.update_ui_callback.clone())(UiEvent::Control(
                    UiControlEvent::ProgressUpdate(*plain_text_block.block_size() as usize),
                ));
                (self.update_ui_callback.clone())(UiEvent::Encryption(
                    UiEncryptionEvent::BlockSolved(
                        block_solution.clone(),
                        plain_text.blocks().len() - i,
                    ),
                ));
                block_solution.to_intermediate()
            } else {
                // prepend an empty block which is to serve as the forgeable block
                let blocks_to_solve = vec![
                    Block::new(&plain_text_block.block_size()),
                    encrypted_blocks_backwards
                        .last()
                        .expect(
                            "Encryption starts with 1 block, yet no block was found in the list",
                        )
                        .clone(),
                ];
                let forged_cypher_text = ForgedCypherText::from_slice(
                    &blocks_to_solve[..],
                    plain_text_block.block_size(),
                    *self.initial_block.url_encoded(),
                    *self.initial_block.used_encoding(),
                );
                let block_solution = solve_block(
                    oracle,
                    &forged_cypher_text,
                    |block, idx| {
                        (self.update_ui_callback.clone())(UiEvent::Encryption(
                            UiEncryptionEvent::BlockWip(block, idx),
                        ));
                    },
                    |newly_solved_bytes| {
                        (self.update_ui_callback.clone())(UiEvent::Control(
                            UiControlEvent::ProgressUpdate(newly_solved_bytes),
                        ));
                    },
                )?;
                let block_solution = block_solution.forged_block_solution();

                (self.update_ui_callback.clone())(UiEvent::Encryption(
                    UiEncryptionEvent::BlockSolved(
                        block_solution.clone(),
                        plain_text.blocks().len() - i,
                    ),
                ));
                block_solution.to_intermediate()
            };

            info!(
                target: LOG_TARGET,
                "Block {}/{}: encrypted!",
                i + 1,
                plain_text.amount_blocks()
            );
            // if this is the last block, it's the IV
            let new_cypher_text_block = &intermediate ^ plain_text_block;
            encrypted_blocks_backwards.push(new_cypher_text_block);
        }

        Ok(CypherText::from_iter(
            encrypted_blocks_backwards.iter().rev(),
            *self.initial_block.url_encoded(),
            *self.initial_block.used_encoding(),
        ))
    }
}
