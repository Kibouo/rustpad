pub mod block_size_option;

use anyhow::{Context, Result};
use clap::{load_yaml, App, ArgMatches};

use crate::oracle::oracle_location::OracleLocation;

use self::block_size_option::BlockSizeOption;

/// Native struct for CLI args.
// Why: because `Clap::ArgMatches` is underlying a `HashMap`, and accessing requires passing strings and error checking. That's ugly.
#[derive(Debug)]
pub struct Options {
    oracle_location: OracleLocation,
    cypher_text: String,
    block_size: BlockSizeOption,
    // sub-commands options
    sub_options: SubOptions,
}

#[derive(Debug)]
pub enum SubOptions {
    Web(WebOptions),
    Script(ScriptOptions),
}

#[derive(Debug, Clone)]
pub struct WebOptions {
    post_data: Option<String>,
    headers: Vec<(String, String)>,
    redirect: bool,
    insecure: bool,
    keyword: String,
}

#[derive(Debug, Clone)]
pub struct ScriptOptions {}

impl Options {
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
    pub fn sub_options(&self) -> &SubOptions {
        &self.sub_options
    }
}

impl WebOptions {
    pub fn post_data(&self) -> &Option<String> {
        &self.post_data
    }

    pub fn headers(&self) -> &Vec<(String, String)> {
        &self.headers
    }

    pub fn redirect(&self) -> bool {
        self.redirect
    }

    pub fn insecure(&self) -> bool {
        self.insecure
    }

    pub fn keyword(&self) -> &String {
        &self.keyword
    }
}

fn parse_as_web(
    oracle_location: &str,
    cypher_text: &str,
    block_size: BlockSizeOption,
    sub_command: &str,
    args: &ArgMatches,
) -> Result<Options> {
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

    let web_options = WebOptions {
        post_data: args.value_of("data").map(|d| d.to_owned()),
        headers: match args.values_of("header") {
            Some(headers) => split_headers(headers)?,
            None => Vec::new(),
        },
        redirect: args.value_of("redirect").is_some(),
        insecure: args.value_of("insecure").is_some(),
        keyword: keyword.into(),
    };

    Ok(Options {
        oracle_location: OracleLocation::new(oracle_location, sub_command)?,
        cypher_text: cypher_text.to_string(),
        block_size,
        sub_options: SubOptions::Web(web_options),
    })
}

fn parse_as_script(
    oracle_location: &str,
    cypher_text: &str,
    block_size: BlockSizeOption,
    sub_command: &str,
    _args: &ArgMatches,
) -> Result<Options> {
    Ok(Options {
        oracle_location: OracleLocation::new(oracle_location, sub_command)?,
        cypher_text: cypher_text.to_string(),
        block_size,
        sub_options: SubOptions::Script(ScriptOptions {}),
    })
}
