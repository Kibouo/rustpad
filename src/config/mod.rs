use std::{ops::Deref, path::PathBuf, str::FromStr};

use anyhow::{anyhow, Context, Result};
use clap::{load_yaml, App, ArgMatches};
use getset::{Getters, MutGetters};
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
    main_config: MainConfig,
    #[getset(get = "pub", get_mut = "pub")]
    sub_config: SubConfig,
}

#[derive(Debug, Getters)]
pub struct MainConfig {
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
    #[getset(get = "pub")]
    thread_count: Option<usize>,
    #[getset(get = "pub")]
    output_file: Option<PathBuf>,
}

#[derive(Debug)]
pub enum SubConfig {
    Web(WebConfig),
    Script(ScriptConfig),
}

#[derive(Debug, Clone, Getters)]
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
    #[getset(get = "pub")]
    request_timeout: u64,
    #[getset(get = "pub")]
    thread_delay: u64,

    // flags
    #[getset(get = "pub")]
    redirect: bool,
    #[getset(get = "pub")]
    insecure: bool,
    #[getset(get = "pub")]
    consider_body: bool,
}

#[derive(Debug, Clone, Getters)]
pub struct ScriptConfig {
    #[getset(get = "pub")]
    thread_delay: u64,
}

impl Config {
    pub fn parse() -> Result<Self> {
        let yaml = load_yaml!("cli.yml");
        let args = App::from(yaml).get_matches();
        let sub_command = args
            .subcommand_name()
            .expect("No required sub-command found");

        let main_config = MainConfig::parse(&args, sub_command)?;
        let sub_config =
            SubConfig::parse(args.subcommand_matches(sub_command).unwrap(), sub_command)?;

        Ok(Config {
            main_config,
            sub_config,
        })
    }
}

impl MainConfig {
    fn parse(args: &ArgMatches, config_type: &str) -> Result<Self> {
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
        let thread_count = args
            .value_of("threads")
            .map(|threads| {
                let threads = threads.parse().context("Thread count failed to parse")?;
                if threads > 0 {
                    Ok(threads)
                } else {
                    Err(anyhow!("Thread count must be greater than 0"))
                }
            })
            .transpose()?;
        let output_file = args
            .value_of("output")
            .map(|file_path| {
                let path = PathBuf::from(file_path);
                if path.exists() {
                    Err(anyhow!(
                        "Log file `{}` already exists. Refusing to overwrite/append",
                        path.display()
                    ))
                } else {
                    Ok(path)
                }
            })
            .transpose()?;

        Ok(Self {
            oracle_location: OracleLocation::new(oracle_location, config_type)?,
            cypher_text: CypherText::parse(cypher_text, &block_size, no_iv)?,
            plain_text: plain_text.map(|plain_text| PlainText::new(plain_text, &block_size)),
            block_size,
            log_level,
            no_iv,
            thread_count,
            output_file,
        })
    }
}

impl SubConfig {
    fn parse(args: &ArgMatches, sub_command: &str) -> Result<Self> {
        let thread_delay = args
            .value_of("delay")
            .map(|delay| delay.parse().context("Thread delay failed to parse"))
            .transpose()?
            .expect("No default value for argument `delay`");

        let sub_config = match sub_command {
            "web" => SubConfig::Web(WebConfig::parse(args, thread_delay)?),
            "script" => SubConfig::Script(ScriptConfig::parse(args, thread_delay)?),
            _ => unreachable!(format!("Invalid sub-command: {}", sub_command)),
        };

        Ok(sub_config)
    }
}

impl WebConfig {
    fn parse(args: &ArgMatches, thread_delay: u64) -> Result<Self> {
        let keyword = args
            .value_of("keyword")
            .expect("No default value for argument `keyword`");

        Ok(Self {
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
                    let split_credentials = credentials
                        .split_once(':')
                        .map(|(user, pass)| (user.to_owned(), pass.to_owned()));
                    split_credentials.context(format!(
                        "Proxy credentials format invalid! Expected `username:password`, got `{}`.",
                        credentials
                    ))
                })
                .transpose()?,
            request_timeout: args
                .value_of("timeout")
                .map(|timeout| {
                    let timeout = timeout.parse().context("Request timeout failed to parse")?;
                    if timeout > 0 {
                        Ok(timeout)
                    } else {
                        Err(anyhow!("Request timeout must be greater than 0"))
                    }
                })
                .transpose()?
                .expect("No default value for argument `timeout`"),
            thread_delay,

            redirect: args.is_present("redirect"),
            insecure: args.is_present("insecure"),
            consider_body: args.is_present("consider_body"),
        })
    }
}

impl ScriptConfig {
    fn parse(_args: &ArgMatches, thread_delay: u64) -> Result<Self> {
        Ok(Self { thread_delay })
    }
}

fn split_headers<'a>(headers: impl IntoIterator<Item = &'a str>) -> Result<Vec<(String, String)>> {
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

impl Deref for Config {
    type Target = MainConfig;

    fn deref(&self) -> &Self::Target {
        &self.main_config
    }
}
