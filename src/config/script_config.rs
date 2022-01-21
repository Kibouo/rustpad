use anyhow::Result;
use getset::Getters;

use super::{cli::ScriptCli, thread_delay::ThreadDelay};

#[derive(Debug, Clone, Getters)]
pub struct ScriptConfig {
    #[getset(get = "pub")]
    thread_delay: ThreadDelay,
}

impl TryFrom<ScriptCli> for ScriptConfig {
    type Error = anyhow::Error;

    fn try_from(cli: ScriptCli) -> Result<Self> {
        Ok(Self {
            thread_delay: cli.thread_delay,
        })
    }
}
