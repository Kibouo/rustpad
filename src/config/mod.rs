use anyhow::{Context, Result};
use clap::{load_yaml, App, ArgMatches};
use getset::{Getters, MutGetters, Setters};
use log::LevelFilter;

use crate::{
    block::block_size::BlockSize, oracle::oracle_location::OracleLocation,
    questioning::calibration_response::CalibrationResponse,
};

/// Native struct for CLI args.
// Why: because `Clap::ArgMatches` is underlying a `HashMap`, and accessing requires passing strings and error checking. That's ugly.
#[derive(Debug, Getters, MutGetters)]
pub struct Config {
    #[getset(get = "pub")]
    oracle_location: OracleLocation,
    #[getset(get = "pub")]
    cypher_text: String,
    #[getset(get = "pub")]
    block_size: BlockSize,
    #[getset(get = "pub")]
    no_iv: bool,
    #[getset(get = "pub")]
    log_level: LevelFilter,
    // sub-commands options
    #[getset(get = "pub", get_mut = "pub")]
    sub_config: SubConfig,
}

#[derive(Debug)]
pub enum SubConfig {
    Web(WebConfig),
    Script(ScriptConfig),
}

#[derive(Debug, Clone, Getters, Setters)]
pub struct WebConfig {
    // arguments
    #[getset(get = "pub")]
    post_data: Option<String>,
    #[getset(get = "pub")]
    headers: Vec<(String, String)>,
    #[getset(get = "pub")]
    keyword: String,

    // flags
    #[getset(get = "pub")]
    redirect: bool,
    #[getset(get = "pub")]
    insecure: bool,
    #[getset(get = "pub")]
    consider_body: bool,

    // config to be filled out later
    #[getset(get = "pub", set = "pub")]
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
        let block_size: BlockSize = args
            .value_of("block_size")
            .expect("No required argument `block_size` found")
            .into();
        let log_level = match args.occurrences_of("verbose") {
            0 => LevelFilter::Info,
            1 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        };
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
                    log_level,
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
                    log_level,
                    sub_command,
                    sub_command_args,
                )
            }
            _ => unreachable!(format!("Invalid sub-command: {}", sub_command)),
        }
    }
}

fn parse_as_web(
    oracle_location: &str,
    cypher_text: &str,
    block_size: BlockSize,
    log_level: LevelFilter,
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
        log_level,
        // all blocks, except the 0-th which is the IV, are to be decrypted.
        no_iv: false,
        sub_config: SubConfig::Web(web_config),
    })
}

fn parse_as_script(
    oracle_location: &str,
    cypher_text: &str,
    block_size: BlockSize,
    log_level: LevelFilter,
    sub_command: &str,
    _args: &ArgMatches,
) -> Result<Config> {
    Ok(Config {
        oracle_location: OracleLocation::new(oracle_location, sub_command)?,
        cypher_text: cypher_text.to_string(),
        block_size,
        log_level,
        // all blocks, except the 0-th which is the IV, are to be decrypted.
        // This could change with a "noiv" option
        no_iv: false,
        sub_config: SubConfig::Script(ScriptConfig {}),
    })
}
