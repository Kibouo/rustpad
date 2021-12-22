use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use clap::{load_yaml, App, ArgMatches};
use getset::{Getters, MutGetters, Setters};
use log::LevelFilter;
use reqwest::Url;

use crate::{
    block::block_size::BlockSize, cypher_text::CypherText, oracle::oracle_location::OracleLocation,
    plain_text::PlainText,
};

const VERSION_TEMPLATE: &str = "<version>";
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Native struct for CLI args.
// Why: because `Clap::ArgMatches` is underlying a `HashMap`, and accessing requires passing strings and error checking. That's ugly.
#[derive(Debug, Getters, MutGetters)]
pub struct Config {
    #[getset(get = "pub")]
    oracle_location: OracleLocation,
    #[getset(get = "pub")]
    cypher_text: CypherText,
    #[getset(get = "pub")]
    plain_text: Option<PlainText>,
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
    #[getset(get = "pub")]
    user_agent: String,
    #[getset(get = "pub")]
    proxy: Option<Url>,
    #[getset(get = "pub")]
    proxy_credentials: Option<(String, String)>,

    // flags
    #[getset(get = "pub")]
    redirect: bool,
    #[getset(get = "pub")]
    insecure: bool,
    #[getset(get = "pub")]
    consider_body: bool,
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
        let block_size: BlockSize = args
            .value_of("block_size")
            .expect("No required argument `block_size` found")
            .into();
        let log_level = match args.occurrences_of("verbose") {
            0 => LevelFilter::Info,
            1 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        };
        let no_iv = args.is_present("no_iv");
        let cypher_text = args
            .value_of("decrypt")
            .expect("No required argument `decrypt` found");
        let plain_text = args.value_of("encrypt");

        let sub_command = args
            .subcommand_name()
            .expect("No required sub-command found");
        match sub_command {
            "web" => {
                let sub_command_args = args.subcommand_matches(sub_command).unwrap();
                parse_as_web(
                    oracle_location,
                    cypher_text,
                    plain_text,
                    block_size,
                    log_level,
                    no_iv,
                    sub_command,
                    sub_command_args,
                )
            }
            "script" => {
                let sub_command_args = args.subcommand_matches(sub_command).unwrap();
                parse_as_script(
                    oracle_location,
                    cypher_text,
                    plain_text,
                    block_size,
                    log_level,
                    no_iv,
                    sub_command,
                    sub_command_args,
                )
            }
            _ => unreachable!(format!("Invalid sub-command: {}", sub_command)),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn parse_as_web(
    oracle_location: &str,
    cypher_text: &str,
    plain_text: Option<&str>,
    block_size: BlockSize,
    log_level: LevelFilter,
    no_iv: bool,
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
                    .map(|(l, r)| (l.trim().to_owned(), r.trim().to_owned()));
                split_header.context(format!(
                    "Header format invalid! Expected `HeaderName: HeaderValue`, got `{}`.",
                    header
                ))
            })
            .collect::<Result<Vec<_>>>()
    }

    let keyword = args
        .value_of("keyword")
        .expect("No default value for argument `keyword`");

    let web_config = WebConfig {
        post_data: args.value_of("data").map(|data| data.to_owned()),
        headers: match args.values_of("header") {
            Some(headers) => split_headers(headers)?,
            None => vec![],
        },
        keyword: keyword.into(),
        user_agent: args
            .value_of("user_agent")
            .map(|agent| agent.replace(VERSION_TEMPLATE, VERSION))
            .expect("No default value for argument `user_agent`"),
        proxy: args
            .value_of("proxy")
            .map(|proxy| Url::from_str(proxy))
            .transpose()
            .context("Proxy URL failed to parse")?,
        proxy_credentials: args
            .value_of("proxy_credentials")
            .map(|credentials| {
                credentials
                    .split_once(':')
                    .map(|(user, pass)| (user.to_owned(), pass.to_owned()))
                    .ok_or_else(|| {
                        anyhow!(
                            "Proxy credentials format invalid! Expected `username:password`, got `{}`."
                        )
                    })
            })
            .transpose()?,

        redirect: args.is_present("redirect"),
        insecure: args.is_present("insecure"),
        consider_body: args.is_present("consider_body"),
    };

    Ok(Config {
        oracle_location: OracleLocation::new(oracle_location, sub_command)?,
        cypher_text: CypherText::parse(cypher_text, &block_size, no_iv)?,
        plain_text: plain_text.map(|plain_text| PlainText::new(plain_text, &block_size)),
        block_size,
        log_level,
        no_iv,
        sub_config: SubConfig::Web(web_config),
    })
}

#[allow(clippy::too_many_arguments)]
fn parse_as_script(
    oracle_location: &str,
    cypher_text: &str,
    plain_text: Option<&str>,
    block_size: BlockSize,
    log_level: LevelFilter,
    no_iv: bool,
    sub_command: &str,
    _args: &ArgMatches,
) -> Result<Config> {
    Ok(Config {
        oracle_location: OracleLocation::new(oracle_location, sub_command)?,
        cypher_text: CypherText::parse(cypher_text, &block_size, no_iv)?,
        plain_text: plain_text.map(|plain_text| PlainText::new(plain_text, &block_size)),
        block_size,
        log_level,
        no_iv,
        sub_config: SubConfig::Script(ScriptConfig {}),
    })
}
