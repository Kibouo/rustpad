use anyhow::{Context, Result};

pub const RETRY_DELAY_MS: u64 = 100;
pub const RETRY_MAX_ATTEMPTS: u64 = 3;

pub(super) fn config_thread_pool(thread_count: Option<usize>) -> Result<()> {
    if let Some(thread_count) = thread_count {
        rayon::ThreadPoolBuilder::new()
            .num_threads(thread_count)
            .build_global()
            .context("Thread pool initialisation failed")
    } else {
        Ok(())
    }
}
