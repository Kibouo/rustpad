use std::sync::{Arc, Mutex};

use anyhow::Result;
use log::{debug, info};

use crate::{
    block::{block_size::BlockSizeTrait, Block},
    cache::Cache,
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
pub(crate) struct Encryptor<'a, U>
where
    U: FnMut(UiEvent) + Sync + Send + Clone,
{
    // intermediate of last block of the user provided cypher text
    initial_block_solution: SolvedForgedCypherText<'a>,
    update_ui_callback: U,
}

impl<'a, U> Encryptor<'a, U>
where
    U: FnMut(UiEvent) + Sync + Send + Clone,
{
    pub(crate) fn new(
        update_ui_callback: U,
        initial_block_solution: SolvedForgedCypherText<'a>,
    ) -> Self {
        debug!(target: LOG_TARGET, "Preparing to encrypt plain text");

        Self {
            initial_block_solution,
            update_ui_callback,
        }
    }

    // encryption looks for the intermediate of the cypher text block, which is then xor-ed with the plain text block to create the cypher text block to be prepended.
    pub(crate) fn encrypt_plain_text(
        &self,
        plain_text: &PlainText,
        oracle: &impl Oracle,
        cache: Arc<Mutex<Option<Cache>>>,
    ) -> Result<CypherText> {
        let mut encrypted_blocks_backwards =
            vec![self.initial_block_solution.block_to_decrypt().clone()];

        // encrypting requires iteratively, backwards, building up the cypher text. Unlike with decrypting, we don't know each block beforehand. So obviously this can't be done in parallel
        for (i, plain_text_block) in plain_text.blocks().iter().rev().enumerate() {
            if i == 0 {
                // decryption of this block is already finished as it's simply the initial block (the last block of the cypher text)
                let block_solution = self.initial_block_solution.forged_block_solution();

                (self.update_ui_callback.clone())(UiEvent::Control(
                    UiControlEvent::ProgressUpdate(*plain_text_block.block_size() as usize),
                ));
                (self.update_ui_callback.clone())(UiEvent::Encryption(
                    UiEncryptionEvent::BlockSolved(
                        block_solution.clone(),
                        plain_text.blocks().len() - i,
                    ),
                ));

                let prepend_cypher_text_block =
                    &block_solution.to_intermediate() ^ plain_text_block;
                encrypted_blocks_backwards.push(prepend_cypher_text_block.clone());

                cache_decryption_equivalent(
                    cache.clone(),
                    prepend_cypher_text_block,
                    self.initial_block_solution.block_to_decrypt().clone(),
                    block_solution.clone(),
                )?;
            } else {
                // Use only the last block, as forging a block is only dependant on its "pair". Not having to send all blocks saves bandwidth and processing time, for us and the oracle
                let cypher_text_block = encrypted_blocks_backwards
                    .last()
                    .expect("Encryption starts with 1 block, yet no block was found in the list")
                    .clone();

                // prepend an empty block which is to serve as the forgeable block
                let blocks_to_solve = vec![
                    Block::new(&plain_text_block.block_size()),
                    cypher_text_block.clone(),
                ];

                let forged_cypher_text = ForgedCypherText::from_slice(
                    &blocks_to_solve[..],
                    plain_text_block.block_size(),
                    *self.initial_block_solution.url_encoded(),
                    *self.initial_block_solution.used_encoding(),
                );
                let block_solution = solve_block(
                    oracle,
                    cache.clone(),
                    &forged_cypher_text,
                    // we don't send all blocks, but only the 2 (pair) needed to progress. The current block thus cannot be determined from the length of `ForgedCypherText`, as is done in `solve_block`.
                    |block, _| {
                        (self.update_ui_callback.clone())(UiEvent::Encryption(
                            UiEncryptionEvent::BlockWip(block, plain_text.amount_blocks() - i),
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

                // if this is the last block, it's the IV
                let prepend_cypher_text_block =
                    &block_solution.to_intermediate() ^ plain_text_block;
                encrypted_blocks_backwards.push(prepend_cypher_text_block.clone());

                cache_decryption_equivalent(
                    cache.clone(),
                    prepend_cypher_text_block,
                    cypher_text_block,
                    block_solution.clone(),
                )?;
            };

            info!(
                target: LOG_TARGET,
                "Block {}/{}: encrypted!",
                i + 1,
                plain_text.amount_blocks()
            );
        }

        Ok(CypherText::from_iter(
            encrypted_blocks_backwards.iter().rev(),
            *self.initial_block_solution.url_encoded(),
            *self.initial_block_solution.used_encoding(),
        ))
    }
}

// encryption uses a (dummy block, cypher block)-pair to build the actual cypher text block to prepend. `solve_block` will cache this pair, instead of the eventual (cypher block - 1, cypher block)-pair. We store this 2nd type of pair here.
fn cache_decryption_equivalent(
    cache: Arc<Mutex<Option<Cache>>>,
    prepend_cypher_text_block: Block,
    cypher_text_block: Block,
    block_solution: Block,
) -> Result<()> {
    cache
        .lock()
        .unwrap()
        .as_mut()
        .map(|cache| {
            cache.insert(
                (prepend_cypher_text_block, cypher_text_block),
                block_solution,
            )
        })
        .transpose()
        .map(|_| ())
}
