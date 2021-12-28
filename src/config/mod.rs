pub mod main_config;
pub mod script_config;
pub mod web_config;

use std::ops::Deref;

use anyhow::{Context, Result};
use clap::{load_yaml, App, ArgMatches};
use getset::{Getters, MutGetters};

use self::{main_config::MainConfig, script_config::ScriptConfig, web_config::WebConfig};

/// Native struct for CLI args.
// Why: because `Clap::ArgMatches` is underlying a `HashMap`, and accessing requires passing strings and error checking. That's ugly.
#[derive(Debug, Getters, MutGetters)]
pub struct Config {
    main_config: MainConfig,
    #[getset(get = "pub", get_mut = "pub")]
    sub_config: SubConfig,
}

#[derive(Debug)]
pub enum SubConfig {
    Web(WebConfig),
    Script(ScriptConfig),
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
