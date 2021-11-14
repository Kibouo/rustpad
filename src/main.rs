mod block;
mod config;
mod cypher_text;
mod oracle;
mod questioning;
mod tui;

use anyhow::{Context, Result};
use crossbeam::thread;
use cypher_text::encode::Encode;

use crate::{
    config::{Config, SubConfig},
    cypher_text::CypherText,
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
    .context("Failed to create terminal UI")?;

    let update_ui_callback = |update| tui.update(update);
    thread::scope(|scope| {
        scope
            .builder()
            .name("TUI".to_string())
            .spawn(|_| tui.main_loop().expect("Error in TUI's main loop"))
            .expect("Failed to create OS thread");

        scope
            .builder()
            .name("Decryption".to_string())
            .spawn(|_| {
                decrypt_main(config, cypher_text, update_ui_callback)
                    .map_err(|e| {
                        (update_ui_callback)(UiUpdate::Done);
                        e
                    })
                    .expect("Error in decryption's main loop")
            })
            .expect("Failed to create OS thread");
    })
    .unwrap();

    Ok(())
}
