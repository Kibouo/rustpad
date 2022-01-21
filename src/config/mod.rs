mod cli;
pub mod encoding_option;
mod header;
pub mod main_config;
mod proxy_credentials;
mod request_timeout;
pub mod script_config;
pub mod thread_count;
pub mod thread_delay;
mod user_agent;
pub mod web_config;

use std::ops::Deref;

use anyhow::Result;
use clap::Parser;
use getset::{Getters, MutGetters};

use self::{
    cli::{Cli, SubCommand},
    main_config::MainConfig,
    script_config::ScriptConfig,
    web_config::WebConfig,
};

/// Application configuration based on processed CLI args.
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
        let cli = Cli::parse();

        let main_config = MainConfig::try_from(&cli)?;
        let sub_config = SubConfig::try_from(cli.sub_command)?;

        Ok(Config {
            main_config,
            sub_config,
        })
    }
}

impl TryFrom<SubCommand> for SubConfig {
    type Error = anyhow::Error;

    fn try_from(sub_command: SubCommand) -> Result<Self> {
        match sub_command {
            SubCommand::Web(web_cli) => Ok(SubConfig::Web(WebConfig::try_from(web_cli)?)),
            SubCommand::Script(script_cli) => {
                Ok(SubConfig::Script(ScriptConfig::try_from(script_cli)?))
            }
        }
    }
}

impl Deref for Config {
    type Target = MainConfig;

    fn deref(&self) -> &Self::Target {
        &self.main_config
    }
}
