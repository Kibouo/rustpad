use std::path::Path;

use anyhow::{anyhow, Context, Result};
use log::LevelFilter;

pub const LOG_TARGET: &str = "rustpad";

pub(super) fn init_logging(log_level: LevelFilter, output_file: Option<&Path>) -> Result<()> {
    tui_logger::init_logger(log_level)
        .map_err(|e| anyhow!("{}", e))
        .context("Logger setup failed")?;
    tui_logger::set_default_level(LevelFilter::Trace);
    if let Some(output_file) = output_file {
        tui_logger::set_log_file(&output_file.to_string_lossy())
            .context(format!("Log file ({:?}) failed to open", output_file))?;
    }

    Ok(())
}
