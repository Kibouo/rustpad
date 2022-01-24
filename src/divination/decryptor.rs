use std::sync::{Arc, Mutex};

use anyhow::Result;
use log::{debug, info};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::{
    cache::Cache,
    calibrator::Calibrator,
    cypher_text::{
        encode::AmountBlocksTrait,
        forged_cypher_text::{solved::SolvedForgedCypherText, ForgedCypherText},
        CypherText,
    },
    divination::solve_block,
    logging::LOG_TARGET,
    oracle::Oracle,
    tui::ui_event::{UiControlEvent, UiDecryptionEvent, UiEvent},
};

/// Manages the oracle attack (decryption) on a high level.
pub(crate) struct Decryptor<'a, U>
where
    U: FnMut(UiEvent) + Sync + Send + Clone,
{
    forged_cypher_texts: Vec<ForgedCypherText<'a>>,
    update_ui_callback: U,
}

impl<'a, U> Decryptor<'a, U>
where
    U: FnMut(UiEvent) + Sync + Send + Clone,
{
    pub(crate) fn new_decryption_only(update_ui_callback: U, cypher_text: &'a CypherText) -> Self {
        Self::new(
            update_ui_callback,
            cypher_text,
            // IV is not decrypted
            1,
        )
    }
    pub(crate) fn new_encryption(update_ui_callback: U, cypher_text: &'a CypherText) -> Self {
        Self::new(
            update_ui_callback,
            cypher_text,
            cypher_text.amount_blocks() - 1,
        )
    }

    pub(crate) fn web_calibrator(&self) -> Calibrator {
        // can't panic as the constructor checks for at least 1 forged cypher text being created
        Calibrator::new(self.forged_cypher_texts[0].clone())
    }

    /// Prepares everything for decryption. Extracts a `ForgedCypherText` for each block to solve from the `CypherText`. This forged cypher text manages the state of its respective block's decryption.
    fn new(update_ui_callback: U, cypher_text: &'a CypherText, blocks_to_skip: usize) -> Self {
        if blocks_to_skip + 1 > cypher_text.amount_blocks() {
            panic!("Need at least 2 blocks to decrypt");
        } else {
            debug!(
                target: LOG_TARGET,
                "Preparing forged cypher texts to decrypt {} block(s)",
                cypher_text.amount_blocks() - blocks_to_skip
            );
        }

        // decryption is based on recognizing padding. Padding is only at the end of a message. So to decrypt the n-th block, all blocks after it have to be dropped and the "n - 1"-th block must be forged.
        let forged_cypher_texts = (blocks_to_skip..cypher_text.amount_blocks())
            .map(|block_to_decrypt_idx| {
                ForgedCypherText::from_cypher_text(cypher_text, block_to_decrypt_idx)
            })
            .collect();

        Self {
            forged_cypher_texts,
            update_ui_callback,
        }
    }

    /// Actually performs the oracle attack to decrypt each block available through `ForgedCypherText`s.
    pub(crate) fn decrypt_blocks(
        &self,
        oracle: &impl Oracle,
        cache: Arc<Mutex<Option<Cache>>>,
    ) -> Result<Vec<SolvedForgedCypherText<'a>>> {
        self.forged_cypher_texts
            .par_iter()
            .enumerate()
            .map(
                |(i, forged_cypher_text)| -> Result<SolvedForgedCypherText<'a>> {
                    let block_to_decrypt_idx = forged_cypher_text.amount_blocks() - 1;
                    let block_solution = solve_block(
                        oracle,
                        cache.clone(),
                        forged_cypher_text,
                        |block, idx| {
                            (self.update_ui_callback.clone())(UiEvent::Decryption(
                                UiDecryptionEvent::BlockWip(block, idx),
                            ));
                        },
                        |newly_solved_bytes| {
                            (self.update_ui_callback.clone())(UiEvent::Control(
                                UiControlEvent::ProgressUpdate(newly_solved_bytes),
                            ));
                        },
                    )?;

                    info!(
                        target: LOG_TARGET,
                        "Block {}/{}: decrypted!",
                        i + 1,
                        self.forged_cypher_texts.len()
                    );
                    (self.update_ui_callback.clone())(UiEvent::Decryption(
                        UiDecryptionEvent::BlockSolved(
                            block_solution.forged_block_solution().clone(),
                            block_to_decrypt_idx,
                        ),
                    ));

                    Ok(block_solution)
                },
            )
            .collect()
    }
}
