use anyhow::Result;
use clap::ArgMatches;
use getset::Getters;

#[derive(Debug, Clone, Getters)]
pub struct ScriptConfig {
    #[getset(get = "pub")]
    thread_delay: u64,
}

impl ScriptConfig {
    pub(super) fn parse(_args: &ArgMatches, thread_delay: u64) -> Result<Self> {
        Ok(Self { thread_delay })
    }
}
