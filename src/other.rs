use std::io;

use anyhow::{Context, Result};
use clap::IntoApp;
use clap_complete::{generate, Shell};

use crate::{cli::Cli, config::thread_count::ThreadCount};

pub(super) const RETRY_DELAY_MS: u64 = 100;
pub(super) const RETRY_MAX_ATTEMPTS: u64 = 3;

pub(super) fn config_thread_pool(thread_count: &ThreadCount) -> Result<()> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(**thread_count)
        .build_global()
        .context("Thread pool initialisation failed")
}

pub(super) fn generate_shell_autocomplete(shell: &Shell) {
    let mut app = Cli::into_app();
    generate(*shell, &mut app, env!("CARGO_PKG_NAME"), &mut io::stdout());
}
