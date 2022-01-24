use anyhow::Result;
use getset::Getters;
use log::LevelFilter;
use std::path::PathBuf;

use crate::{
    block::block_size::BlockSize, cli::GlobalOptions, cypher_text::CypherText,
    oracle::oracle_location::OracleLocation, plain_text::PlainText,
};

use super::thread_count::ThreadCount;

#[derive(Debug, Getters)]
pub(crate) struct GlobalConfig {
    #[getset(get = "pub(crate)")]
    oracle_location: OracleLocation,
    #[getset(get = "pub(crate)")]
    cypher_text: CypherText,
    #[getset(get = "pub(crate)")]
    plain_text: Option<PlainText>,
    #[getset(get = "pub(crate)")]
    block_size: BlockSize,
    #[getset(get = "pub(crate)")]
    log_level: LevelFilter,
    #[getset(get = "pub(crate)")]
    thread_count: ThreadCount,
    #[getset(get = "pub(crate)")]
    output_file: Option<PathBuf>,
    #[getset(get = "pub(crate)")]
    no_cache: bool,
}

impl TryFrom<&GlobalOptions> for GlobalConfig {
    type Error = anyhow::Error;

    fn try_from(options: &GlobalOptions) -> Result<Self> {
        let log_level = match options.verbosity() {
            0 => LevelFilter::Info,
            1 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        };

        Ok(Self {
            oracle_location: options.oracle_location().clone(),
            cypher_text: CypherText::parse(
                options.cypher_text(),
                options.block_size(),
                *options.no_iv(),
                options.encoding(),
                *options.no_url_encode(),
            )?,
            plain_text: options
                .plain_text()
                .as_ref()
                .map(|plain_text| PlainText::new(plain_text, options.block_size())),
            block_size: *options.block_size(),
            log_level,
            thread_count: options.thread_count().clone(),
            output_file: options.log_file().clone(),
            no_cache: *options.no_cache(),
        })
    }
}
