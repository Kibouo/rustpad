use anyhow::{anyhow, Context, Result};
use log::LevelFilter;
use tui_logger::{init_logger, set_default_level};

pub const LOG_TARGET: &str = "rustpad";

pub(super) fn init_logging(log_level: LevelFilter) -> Result<()> {
    init_logger(log_level)
        .map_err(|e| anyhow!("{}", e))
        .context("Logger creation failed")?;
    set_default_level(LevelFilter::Trace);

    Ok(())
}
