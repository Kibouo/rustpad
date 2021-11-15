mod block;
mod config;
mod cypher_text;
mod logging;
mod oracle;
mod questioning;
mod tui;

use std::process;

use anyhow::{Context, Result};
use crossbeam::thread;
use cypher_text::encode::Encode;
use log::{error, info};
use logging::init_logging;

use crate::{
    config::{Config, SubConfig},
    cypher_text::CypherText,
    logging::LOG_TARGET,
    oracle::{
        oracle_location::OracleLocation,
        script::ScriptOracle,
        web::{calibrate_web::CalibrationWebOracle, WebOracle},
        Oracle,
    },
    questioning::Questioning,
    tui::{ui_update::UiUpdate, Tui},
};

fn decrypt_main(
    mut config: Config,
    cypher_text: CypherText,
    update_ui_callback: impl FnMut(UiUpdate) + Sync + Send + Clone,
) -> Result<()> {
    match config.oracle_location() {
        OracleLocation::Web(_) => {
            info!(target: LOG_TARGET, "Using web oracle");
            let mut questioning =
                Questioning::prepare(update_ui_callback, &cypher_text, *config.no_iv())?;

            let calibration_oracle =
                CalibrationWebOracle::visit(config.oracle_location(), config.sub_config())?;
            let padding_error_response = questioning.calibrate_web_oracle(calibration_oracle)?;

            if let SubConfig::Web(web_config) = config.sub_config_mut() {
                web_config.set_padding_error_response(Some(padding_error_response));
            }

            let oracle = WebOracle::visit(config.oracle_location(), config.sub_config())?;
            questioning.start(oracle)
        }
        OracleLocation::Script(_) => {
            info!(target: LOG_TARGET, "Using script oracle");
            let oracle = ScriptOracle::visit(config.oracle_location(), config.sub_config())?;
            Questioning::prepare(update_ui_callback, &cypher_text, *config.no_iv())?.start(oracle)
        }
    }
}

fn main() -> Result<()> {
    let config = Config::parse()?;
    let cypher_text = CypherText::parse(config.cypher_text(), config.block_size())?;

    let tui = Tui::new(
        config.block_size(),
        cypher_text.blocks().to_vec(),
        *config.no_iv(),
    )
    .context("TUI creation failed")?;
    init_logging(*config.log_level())?;

    let update_ui_callback = |update| tui.update(update);
    thread::scope(|scope| {
        if let Err(e) = scope.builder().name("TUI".to_string()).spawn(|_| {
            if let Err(e) = tui.main_loop() {
                error!(target: LOG_TARGET, "{:?}", e);
                // decryption thread can stop the draw main loop, but the other way around there is no such thing
                process::exit(1)
            }
        }) {
            error!(target: LOG_TARGET, "{:?}", e);
            process::exit(1)
        }

        if let Err(e) = scope.builder().name("Decryption".to_string()).spawn(|_| {
            if let Err(e) = decrypt_main(config, cypher_text, update_ui_callback) {
                error!(target: LOG_TARGET, "{:?}", e);
                (update_ui_callback)(UiUpdate::SlowRedraw);
            }
        }) {
            error!(target: LOG_TARGET, "{:?}", e);
            (update_ui_callback)(UiUpdate::SlowRedraw);
        }
    })
    .unwrap();

    Ok(())
}
