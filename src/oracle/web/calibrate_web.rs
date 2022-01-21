use anyhow::{Context, Result};
use getset::Getters;
use reqwest::{
    blocking::{Client, Response},
    Url,
};

use crate::{
    config::{thread_delay::ThreadDelay, web_config::WebConfig, SubConfig},
    cypher_text::encode::Encode,
    oracle::oracle_location::OracleLocation,
};

use super::{build_web_oracle, replace_keyword_occurrences, KeywordLocation};

/// Unlike with `ScriptOracle`, we don't know which response from the web oracle corresponds with "valid", and which corresponds to "incorrect padding". For `WebOracle` to magically work, we need to determine the "incorrect padding" response. This struct manages the requests used for the calibration.
/// `ask_validation` needs to return the web request's `Response`.Meaning, `Oracle` can't be implemented. Also, implementing it would be confusing as `CalibrateWebOracle`'s purpose is different from normal oracles.
#[derive(Getters)]
pub struct CalibrationWebOracle {
    url: Url,
    #[getset(get = "pub")]
    config: WebConfig,
    web_client: Client,
    keyword_locations: Vec<KeywordLocation>,
}

impl CalibrationWebOracle {
    pub fn visit(oracle_location: &OracleLocation, oracle_config: &SubConfig) -> Result<Self> {
        let (url, web_client, keyword_locations, web_config) =
            build_web_oracle(oracle_location, oracle_config)?;

        let oracle = Self {
            url,
            config: web_config.clone(),
            web_client,
            keyword_locations,
        };
        Ok(oracle)
    }

    pub fn ask_validation<'a>(&self, cypher_text: &'a impl Encode<'a>) -> Result<Response> {
        let (url, data, headers) = replace_keyword_occurrences(
            &self.url,
            &self.config,
            self.keyword_locations.iter(),
            &cypher_text.encode(),
        )
        .context("Replacing all occurrences of keyword failed")?;

        let request = if self.config.post_data().is_none() {
            self.web_client.get(url)
        } else {
            self.web_client.post(url)
        };
        let request = request.headers(headers);
        let request = match data {
            Some(data) => request.body(data),
            None => request,
        };

        request.send().context("Sending request failed")
    }

    pub fn thread_delay(&self) -> &ThreadDelay {
        self.config.thread_delay()
    }
}
