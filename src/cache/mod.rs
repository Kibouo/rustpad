pub(super) mod cache_config;

use std::{
    collections::HashMap,
    fs::{create_dir_all, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

use anyhow::{Context, Result};

use crate::block::Block;

use self::cache_config::CacheConfig;

const CACHE_FILE_NAME: &str = "cache.bin";

pub(super) struct Cache {
    cache_file: File,
    config: CacheConfig,
    data: HashMap<CacheConfig, HashMap<(Block, Block), Block>>,
}

impl Cache {
    pub(super) fn load_from_file(config: CacheConfig) -> Result<Self> {
        let mut cache_file = open_cache_file()?;

        let mut file_data = vec![];
        cache_file
            .read_to_end(&mut file_data)
            .context("Cache file read failure")?;

        let mut data = if file_data.is_empty() {
            HashMap::new()
        } else {
            rmp_serde::from_read_ref(&file_data)
                .context("Cache file de-serialization failed: corrupted MessagePack data")?
        };

        // create an entry for the current config if needed
        let _ = data.entry(config.clone()).or_insert_with(HashMap::new);

        Ok(Self {
            cache_file,
            config,
            data,
        })
    }

    pub(super) fn insert(&mut self, key: (Block, Block), value: Block) -> Result<()> {
        let _ = self
            .data
            .entry(self.config.clone())
            .or_insert_with(HashMap::new)
            .insert(key, value);

        // write back to file
        // clear file 1st and then write, instead of writing 1st and then adjusting the length. In case of an error, this leaves an empty file. The other approach would leave corrupted binary data in the file.
        self.cache_file
            .set_len(0)
            .context("Cache file emptying failed")?;
        self.cache_file
            .seek(SeekFrom::Start(0))
            .context("Cache file seek-to-start failed")?;
        self.cache_file
            .write_all(&rmp_serde::to_vec(&self.data).context("Cache data serialization failed")?)
            .context("Cache could not be saved")
    }

    pub(super) fn get(&self, key: &(Block, Block)) -> Option<&Block> {
        self.data
            .get(&self.config)
            .and_then(|blocks_mapping| blocks_mapping.get(key))
    }
}

fn open_cache_file() -> Result<File> {
    let cache_file_dir = dirs::cache_dir()
        .map(|dir| dir.join(env!("CARGO_PKG_NAME")))
        .unwrap_or_else(|| PathBuf::from("./cache"));
    create_dir_all(&cache_file_dir).context("Cache directory creation failed")?;

    let cache_file_path = cache_file_dir.join(CACHE_FILE_NAME);
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(cache_file_path.clone())
        .context(format!(
            "Cache file `{}` failed to open",
            cache_file_path.display()
        ))
}
