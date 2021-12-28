use anyhow::{Context, Result};
use clap::ArgMatches;
use getset::Getters;

#[derive(Debug, Clone, Getters)]
pub struct ScriptConfig {
    #[getset(get = "pub")]
    thread_delay: u64,
}

impl ScriptConfig {
    pub(super) fn parse(args: &ArgMatches) -> Result<Self> {
        Ok(Self {
            thread_delay: args
                .value_of("delay")
                .map(|delay| delay.parse().context("Thread delay failed to parse"))
                .transpose()?
                .expect("No default value for argument `delay`"),
        })
    }
}
