use anyhow::{anyhow, Context, Result};
use clap::ArgMatches;
use getset::Getters;
use log::LevelFilter;
use std::{path::PathBuf, str::FromStr};

use crate::{
    block::block_size::BlockSize,
    cypher_text::{encode::Encoding, CypherText},
    oracle::oracle_location::OracleLocation,
    plain_text::PlainText,
};

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
    thread_count: usize,
    #[getset(get = "pub")]
    output_file: Option<PathBuf>,
    #[getset(get = "pub")]
    no_cache: bool,
}

impl MainConfig {
    pub(super) fn parse(args: &ArgMatches, config_type: &str) -> Result<Self> {
        let input_oracle_location = args
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
        let input_cypher_text = args
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
            .transpose()?
            .expect("No required argument `threads` found");
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
        let specified_encoding = {
            let encoding_choice = args
                .value_of("encoding")
                .expect("No default value for argument `encoding`");
            if encoding_choice == "auto" {
                None
            } else {
                Some(Encoding::from_str(encoding_choice).context("Encoding failed to parse")?)
            }
        };
        let no_url_encode = args.is_present("no_url_encode");
        let no_cache = args.is_present("no_cache");

        Ok(Self {
            oracle_location: OracleLocation::new(input_oracle_location, config_type)?,
            cypher_text: CypherText::parse(
                input_cypher_text,
                &block_size,
                no_iv,
                specified_encoding,
                no_url_encode,
            )?,
            plain_text: plain_text.map(|plain_text| PlainText::new(plain_text, &block_size)),
            block_size,
            log_level,
            thread_count,
            output_file,
            no_cache,
        })
    }
}
