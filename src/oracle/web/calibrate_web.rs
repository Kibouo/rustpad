use anyhow::{anyhow, Context, Result};
use reqwest::{
    blocking::{Client, ClientBuilder, Response},
    redirect::Policy,
    Url,
};

use crate::{
    cli::{SubOptions, WebOptions},
    cypher_text::encode::Encode,
    oracle::oracle_location::OracleLocation,
};

use super::{keyword_location, replace_keyword_occurrences, KeywordLocation};

/// Unlike with `ScriptOracle`, we don't know which response from the web server corresponds with "valid", and which corresponds to "incorrect padding". For `WebOracle` to magically work, we need to determine the "incorrect padding" response. This struct manages the requests used for the calibration.
/// `ask_validation` needs to return the web request's `Response`.Meaning, `Oracle` can't be implemented. Also, implementing it would be confusing as `CalibrateWebOracle`'s purpose is different from normal oracles.
pub struct CalibrationWebOracle {
    url: Url,
    options: WebOptions,
    web_client: Client,
    keyword_locations: Vec<KeywordLocation>,
}

impl CalibrationWebOracle {
    pub fn visit(oracle_location: &OracleLocation, oracle_options: &SubOptions) -> Result<Self> {
        let url = match oracle_location {
            OracleLocation::Web(url) => url,
            OracleLocation::Script(_) => {
                return Err(anyhow!("Tried to visit the web oracle using a file path!"));
            }
        };

        let options = match oracle_options {
            SubOptions::Web(options) => options,
            SubOptions::Script(_) => {
                return Err(anyhow!(
                    "Tried to visit the web oracle using script options!"
                ));
            }
        };

        let keyword_locations = keyword_location(url, options);
        if keyword_locations.is_empty() {
            return Err(anyhow!(
                "Keyword not found in URL, headers, or POST data. See `--keyword` for further info"
            ));
        }

        let mut client_builder =
            ClientBuilder::new().danger_accept_invalid_certs(options.insecure());
        if !options.redirect() {
            client_builder = client_builder.redirect(Policy::none());
        }

        let web_client = client_builder
            .build()
            .context("Failed to setup web client")?;

        let oracle = Self {
            url: url.to_owned(),
            options: options.clone(),
            web_client,
            keyword_locations,
        };
        Ok(oracle)
    }

    pub fn ask_validation<'a>(&self, cypher_text: &'a impl Encode<'a>) -> Result<Response> {
        let (url, data, headers) = replace_keyword_occurrences(
            &self.url,
            &self.options,
            self.keyword_locations.iter(),
            &cypher_text.encode(),
        )
        .context("Failed to replace all occurrences of the keyword")?;

        let request = if self.options.post_data().is_none() {
            self.web_client.get(url)
        } else {
            self.web_client.post(url)
        };
        let request = request.headers(headers);
        let request = match data {
            Some(data) => request.body(data),
            None => request,
        };

        request.send().context("Failed to send request")
    }
}
