pub mod block_size_option;

use anyhow::{Context, Result};
use clap::{load_yaml, App, ArgMatches};

use crate::{
    oracle::oracle_location::OracleLocation, questioning::calibration_response::CalibrationResponse,
};

use self::block_size_option::BlockSizeOption;

/// Native struct for CLI args.
// Why: because `Clap::ArgMatches` is underlying a `HashMap`, and accessing requires passing strings and error checking. That's ugly.
#[derive(Debug)]
pub struct Config {
    oracle_location: OracleLocation,
    cypher_text: String,
    block_size: BlockSizeOption,
    // sub-commands options
    sub_config: SubConfig,
}

#[derive(Debug)]
pub enum SubConfig {
    Web(WebConfig),
    Script(ScriptConfig),
}

#[derive(Debug, Clone)]
pub struct WebConfig {
    // arguments
    post_data: Option<String>,
    headers: Vec<(String, String)>,
    keyword: String,

    // flags
    redirect: bool,
    insecure: bool,
    consider_body: bool,

    // config to be filled out later
    padding_error_response: Option<CalibrationResponse>,
}

#[derive(Debug, Clone)]
pub struct ScriptConfig {}

impl Config {
    pub fn parse() -> Result<Self> {
        let yaml = load_yaml!("cli.yml");
        let args = App::from(yaml).get_matches();

        let oracle_location = args
            .value_of("oracle")
            .expect("No required argument `oracle` found");
        let cypher_text = args
            .value_of("cypher_text")
            .expect("No required argument `cypher_text` found");
        let block_size: BlockSizeOption = args
            .value_of("block_size")
            .expect("No required argument `block_size` found")
            .into();
        let sub_command = args
            .subcommand_name()
            .expect("No required sub-command found");

        match sub_command {
            "web" => {
                let sub_command_args = args.subcommand_matches(sub_command).unwrap();
                parse_as_web(
                    oracle_location,
                    cypher_text,
                    block_size,
                    sub_command,
                    sub_command_args,
                )
            }
            "script" => {
                let sub_command_args = args.subcommand_matches(sub_command).unwrap();
                parse_as_script(
                    oracle_location,
                    cypher_text,
                    block_size,
                    sub_command,
                    sub_command_args,
                )
            }
            _ => unreachable!(format!("Invalid sub-command: {}", sub_command)),
        }
    }

    pub fn oracle_location(&self) -> &OracleLocation {
        &self.oracle_location
    }
    pub fn cypher_text(&self) -> &str {
        &self.cypher_text
    }
    pub fn block_size(&self) -> &BlockSizeOption {
        &self.block_size
    }
    pub fn sub_config(&self) -> &SubConfig {
        &self.sub_config
    }

    pub fn sub_config_mut(&mut self) -> &mut SubConfig {
        &mut self.sub_config
    }
}

impl WebConfig {
    pub fn post_data(&self) -> &Option<String> {
        &self.post_data
    }
    pub fn headers(&self) -> &Vec<(String, String)> {
        &self.headers
    }
    pub fn keyword(&self) -> &String {
        &self.keyword
    }
    pub fn redirect(&self) -> bool {
        self.redirect
    }
    pub fn insecure(&self) -> bool {
        self.insecure
    }
    pub fn consider_body(&self) -> bool {
        self.consider_body
    }
    pub fn padding_error_response(&self) -> &Option<CalibrationResponse> {
        &self.padding_error_response
    }

    pub fn save_padding_error_response(
        &mut self,
        padding_error_response: CalibrationResponse,
    ) -> &mut Self {
        self.padding_error_response = Some(padding_error_response);
        self
    }
}

fn parse_as_web(
    oracle_location: &str,
    cypher_text: &str,
    block_size: BlockSizeOption,
    sub_command: &str,
    args: &ArgMatches,
) -> Result<Config> {
    fn split_headers<'a>(
        headers: impl IntoIterator<Item = &'a str>,
    ) -> Result<Vec<(String, String)>> {
        headers
            .into_iter()
            .map(|header| -> Result<(String, String)> {
                let split_header = header
                    .split_once(':')
                    .map(|(l, r)| (l.to_owned(), r.to_owned()));
                split_header.context(format!(
                    "Invalid header format! Expected 'HeaderName: HeaderValue', got '{}'.",
                    header
                ))
            })
            .collect::<Result<Vec<_>>>()
    }

    let keyword = args
        .value_of("keyword")
        .expect("No default value for argument `keyword`");

    let web_config = WebConfig {
        post_data: args.value_of("data").map(|d| d.to_owned()),
        headers: match args.values_of("header") {
            Some(headers) => split_headers(headers)?,
            None => Vec::new(),
        },
        keyword: keyword.into(),

        redirect: args.value_of("redirect").is_some(),
        insecure: args.value_of("insecure").is_some(),
        consider_body: args.value_of("consider_body").is_some(),

        padding_error_response: None,
    };

    Ok(Config {
        oracle_location: OracleLocation::new(oracle_location, sub_command)?,
        cypher_text: cypher_text.to_string(),
        block_size,
        sub_config: SubConfig::Web(web_config),
    })
}

fn parse_as_script(
    oracle_location: &str,
    cypher_text: &str,
    block_size: BlockSizeOption,
    sub_command: &str,
    _args: &ArgMatches,
) -> Result<Config> {
    Ok(Config {
        oracle_location: OracleLocation::new(oracle_location, sub_command)?,
        cypher_text: cypher_text.to_string(),
        block_size,
        sub_config: SubConfig::Script(ScriptConfig {}),
    })
}
