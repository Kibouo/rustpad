mod block;
mod calibrator;
mod config;
mod cypher_text;
mod divination;
mod logging;
mod oracle;
mod other;
mod plain_text;
mod tui;

use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use block::block_size::BlockSizeTrait;
use crossbeam::thread;
use cypher_text::encode::{AmountBlocksTrait, Encode};
use divination::decryptor::Decryptor;
use humantime::format_duration;
use log::{error, info};
use logging::init_logging;

use crate::{
    config::Config,
    divination::encryptor::Encryptor,
    logging::LOG_TARGET,
    oracle::{
        oracle_location::OracleLocation,
        script::ScriptOracle,
        web::{calibrate_web::CalibrationWebOracle, WebOracle},
        Oracle,
    },
    tui::{
        ui_event::{UiControlEvent, UiDecryptionEvent, UiEncryptionEvent, UiEvent},
        Tui,
    },
};

fn main() -> Result<()> {
    let config = Config::parse()?;

    let tui = Tui::new(config.block_size()).context("TUI creation failed")?;
    init_logging(*config.log_level())?;

    let update_ui_callback = |event| tui.handle_application_event(event);
    thread::scope(|scope| {
        if let Err(e) = scope.builder().name("TUI".to_string()).spawn(|_| {
            if let Err(e) = tui.main_loop() {
                error!(target: LOG_TARGET, "{:?}", e);
                // logic thread can stop the draw main loop, but there is no such thing the other way around
                tui.exit(1)
            }
        }) {
            error!(target: LOG_TARGET, "{:?}", e);
            tui.exit(1)
        }

        if let Err(e) = scope
            .builder()
            .name("Padding oracle attack".to_string())
            .spawn(|_| {
                if let Err(e) = logic_preparation(config, update_ui_callback) {
                    error!(target: LOG_TARGET, "{:?}", e);
                    (update_ui_callback)(UiEvent::Control(UiControlEvent::SlowRedraw));
                }
            })
        {
            error!(target: LOG_TARGET, "{:?}", e);
            (update_ui_callback)(UiEvent::Control(UiControlEvent::SlowRedraw));
        }
    })
    .unwrap();

    Ok(())
}

fn logic_preparation<U>(config: Config, mut update_ui_callback: U) -> Result<()>
where
    U: FnMut(UiEvent) + Sync + Send + Clone,
{
    let encryption_mode = config.plain_text().is_some();
    let decryptor = if encryption_mode {
        Decryptor::new_encryption(update_ui_callback.clone(), config.cypher_text())
    } else {
        Decryptor::new_decryption_only(update_ui_callback.clone(), config.cypher_text())
    };

    match config.oracle_location() {
        OracleLocation::Web(_) => {
            info!(target: LOG_TARGET, "Using web oracle");
            let web_calibrator = decryptor.request_calibrator();
            let calibration_oracle =
                CalibrationWebOracle::visit(config.oracle_location(), config.sub_config())?;
            let padding_error_response =
                web_calibrator.determine_padding_error_response(calibration_oracle)?;

            let mut oracle = WebOracle::visit(config.oracle_location(), config.sub_config())?;
            oracle.set_padding_error_response(Some(padding_error_response));

            logic_main(
                &decryptor,
                &oracle,
                encryption_mode,
                update_ui_callback.clone(),
                &config,
            )?;
        }
        OracleLocation::Script(_) => {
            info!(target: LOG_TARGET, "Using script oracle");
            let oracle = ScriptOracle::visit(config.oracle_location(), config.sub_config())?;
            decryptor.decrypt_blocks(&oracle)?;

            logic_main(
                &decryptor,
                &oracle,
                encryption_mode,
                update_ui_callback.clone(),
                &config,
            )?;
        }
    };

    // keep window open for user to read results
    (update_ui_callback)(UiEvent::Control(UiControlEvent::SlowRedraw));
    Ok(())
}

fn logic_main<U>(
    decryptor: &Decryptor<U>,
    oracle: &impl Oracle,
    encryption_mode: bool,
    update_ui_callback: U,
    config: &Config,
) -> Result<()>
where
    U: FnMut(UiEvent) + Sync + Send + Clone,
{
    (update_ui_callback.clone())(UiEvent::Decryption(UiDecryptionEvent::InitDecryption(
        config.cypher_text().blocks().to_vec(),
    )));
    (update_ui_callback.clone())(UiEvent::Control(UiControlEvent::IndicateWork(
        if encryption_mode {
            let plain_text = config
                .plain_text()
                .as_ref()
                .expect("Should have a plain text in encryption mode");
            // + 1 for decrypting a block of cypher text
            (plain_text.amount_blocks() + 1) * *plain_text.block_size() as usize
        } else {
            let cypher_text = config.cypher_text();
            // -1 as IV doesn't have to be decrypted
            (cypher_text.amount_blocks() - 1) * *cypher_text.block_size() as usize
        },
    )));

    let now = Instant::now();
    let decryption_results = decryptor.decrypt_blocks(oracle)?;

    if encryption_mode {
        let last_block = decryption_results
            .into_iter()
            .max_by_key(|cypher_text| cypher_text.original_blocks().len())
            .expect("Can't encrypt without having decrypted a block");
        (update_ui_callback.clone())(UiEvent::Encryption(UiEncryptionEvent::InitEncryption(
            config
                .plain_text()
                .as_ref()
                .expect("Should have a plain text in encryption mode")
                .blocks()
                .to_vec(),
            last_block.block_to_decrypt().clone(),
        )));

        let encryptor = Encryptor::new(update_ui_callback, last_block);

        let encrypted_plain_text = encryptor.encrypt_plain_text(
            config
                .plain_text()
                .as_ref()
                .expect("Should have a plain text in encryption mode"),
            oracle,
        )?;

        info!(
            target: LOG_TARGET,
            "The oracle talked some gibberish. It took {}",
            format_duration(Duration::new(now.elapsed().as_secs(), 0))
        );
        info!(
            target: LOG_TARGET,
            "Their divination is: {}",
            encrypted_plain_text.encode()
        );
    } else {
        info!(
            target: LOG_TARGET,
            "The oracle talked some gibberish. It took {}",
            format_duration(Duration::new(now.elapsed().as_secs(), 0))
        );
        let plain_text_solution: String = decryption_results
            .iter()
            .map(|forged_cypher_text| forged_cypher_text.plain_text_solution())
            .collect();
        info!(
            target: LOG_TARGET,
            "Their divination is: {}", plain_text_solution
        );
    };

    Ok(())
}
