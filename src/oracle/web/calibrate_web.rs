use anyhow::{anyhow, Context, Result};
use getset::Getters;
use reqwest::{
    blocking::{Client, ClientBuilder, Response},
    redirect::Policy,
    Url,
};

use crate::{
    config::{SubConfig, WebConfig},
    cypher_text::encode::Encode,
    oracle::oracle_location::OracleLocation,
};

use super::{keyword_location, replace_keyword_occurrences, KeywordLocation};

/// Unlike with `ScriptOracle`, we don't know which response from the web server corresponds with "valid", and which corresponds to "incorrect padding". For `WebOracle` to magically work, we need to determine the "incorrect padding" response. This struct manages the requests used for the calibration.
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
        let url = match oracle_location {
            OracleLocation::Web(url) => url,
            OracleLocation::Script(_) => {
                panic!("Tried to visit the web oracle using a file path!");
            }
        };

        let oracle_config = match oracle_config {
            SubConfig::Web(config) => config,
            SubConfig::Script(_) => {
                panic!("Tried to visit the web oracle using script configs!");
            }
        };

        let keyword_locations = keyword_location(url, oracle_config);
        if keyword_locations.is_empty() {
            return Err(anyhow!(
                "Keyword not found in URL, headers, or POST data. See `--keyword` for further info"
            ));
        }

        let mut client_builder = ClientBuilder::new()
            .danger_accept_invalid_certs(*oracle_config.insecure())
            .user_agent(oracle_config.user_agent());
        if !oracle_config.redirect() {
            client_builder = client_builder.redirect(Policy::none());
        }

        let web_client = client_builder.build().context("Web client setup failed")?;

        let oracle = Self {
            url: url.to_owned(),
            config: oracle_config.clone(),
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
}
