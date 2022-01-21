use anyhow::Result;
use getset::Getters;
use log::LevelFilter;
use std::path::PathBuf;

use crate::{
    block::block_size::BlockSize, cypher_text::CypherText, oracle::oracle_location::OracleLocation,
    plain_text::PlainText,
};

use super::{cli::Cli, thread_count::ThreadCount};

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
    log_level: LevelFilter,
    #[getset(get = "pub")]
    thread_count: ThreadCount,
    #[getset(get = "pub")]
    output_file: Option<PathBuf>,
    #[getset(get = "pub")]
    no_cache: bool,
}

impl TryFrom<&Cli> for MainConfig {
    type Error = anyhow::Error;

    fn try_from(cli: &Cli) -> Result<Self> {
        let log_level = match cli.verbosity() {
            0 => LevelFilter::Info,
            1 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        };

        Ok(Self {
            oracle_location: cli.oracle_location().clone(),
            cypher_text: CypherText::parse(
                cli.cypher_text(),
                cli.block_size(),
                *cli.no_iv(),
                cli.encoding(),
                *cli.no_url_encode(),
            )?,
            plain_text: cli
                .plain_text()
                .as_ref()
                .map(|plain_text| PlainText::new(plain_text, cli.block_size())),
            block_size: *cli.block_size(),
            log_level,
            thread_count: cli.thread_count().clone(),
            output_file: cli.log_file().clone(),
            no_cache: *cli.no_cache(),
        })
    }
}
