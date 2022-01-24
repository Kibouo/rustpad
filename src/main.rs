mod block;
mod cache;
mod calibrator;
mod cli;
mod config;
mod cypher_text;
mod divination;
mod logging;
mod oracle;
mod other;
mod plain_text;
mod tui;

use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use async_std::task;
use clap::StructOpt;
use crossbeam::thread;
use humantime::format_duration;
use log::{error, info};

use crate::{
    block::block_size::BlockSizeTrait,
    cache::{cache_config::CacheConfig, Cache},
    calibrator::calibration_response::CalibrationResponse,
    cli::Cli,
    config::Config,
    cypher_text::encode::{AmountBlocksTrait, Encode},
    divination::{decryptor::Decryptor, encryptor::Encryptor},
    logging::{init_logging, LOG_TARGET},
    oracle::{
        oracle_location::OracleLocation,
        script::ScriptOracle,
        web::{calibrate_web::CalibrationWebOracle, WebOracle},
        Oracle,
    },
    other::{config_thread_pool, generate_shell_autocomplete},
    tui::{
        ui_event::{UiControlEvent, UiDecryptionEvent, UiEncryptionEvent, UiEvent},
        Tui,
    },
};

fn main() -> Result<()> {
    let cli = Cli::parse();
    if let cli::SubCommand::Setup(setup_cli) = cli.sub_command {
        generate_shell_autocomplete(setup_cli.shell());
        return Ok(());
    }
    let config = Config::try_from(cli)?;

    config_thread_pool(config.thread_count())?;
    init_logging(*config.log_level(), config.output_file().as_deref())?;
    // couldn't log cypher text info during parsing as logger wasn't initiated yet
    info!(target: LOG_TARGET, "Using encoding:");
    info!(
        target: LOG_TARGET,
        "- {:?}",
        config.cypher_text().used_encoding(),
    );
    info!(
        target: LOG_TARGET,
        "- URL encoded: {}",
        config.cypher_text().url_encoded()
    );

    let tui = Tui::new(config.block_size()).context("TUI creation failed")?;

    let update_ui_callback = |event| tui.handle_application_event(event);
    thread::scope(|scope| {
        if let Err(e) = scope.builder().name("TUI".to_string()).spawn(|_| {
            if let Err(e) = task::block_on(tui.main_loop()) {
                error!(target: LOG_TARGET, "{:?}", e);
                // logic thread can stop the draw main loop, but there is no such thing the other way around
                update_ui_callback(UiEvent::Control(UiControlEvent::PrintAfterExit(format!(
                    "Error: {:?}",
                    e
                ))));
                update_ui_callback(UiEvent::Control(UiControlEvent::ExitCode(1)));
                tui.exit()
            }
        }) {
            error!(target: LOG_TARGET, "{:?}", e);
            update_ui_callback(UiEvent::Control(UiControlEvent::PrintAfterExit(format!(
                "Error: {:?}",
                e
            ))));
            update_ui_callback(UiEvent::Control(UiControlEvent::ExitCode(2)));
            tui.exit()
        }

        if let Err(e) = scope
            .builder()
            .name("Padding oracle attack".to_string())
            .spawn(|_| {
                if let Err(e) = logic_preparation(config, update_ui_callback) {
                    error!(target: LOG_TARGET, "{:?}", e);
                    update_ui_callback(UiEvent::Control(UiControlEvent::PrintAfterExit(format!(
                        "Error: {:?}",
                        e
                    ))));
                    update_ui_callback(UiEvent::Control(UiControlEvent::ExitCode(3)));
                    (update_ui_callback)(UiEvent::Control(UiControlEvent::SlowRedraw));
                }
            })
        {
            error!(target: LOG_TARGET, "{:?}", e);
            update_ui_callback(UiEvent::Control(UiControlEvent::PrintAfterExit(format!(
                "Error: {:?}",
                e
            ))));
            update_ui_callback(UiEvent::Control(UiControlEvent::ExitCode(4)));
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
            let mut oracle = WebOracle::visit(config.oracle_location(), config.sub_config())?;
            let padding_error_response =
                calibrate_web(&decryptor, update_ui_callback.clone(), &config)?;
            oracle.set_padding_error_response(Some(padding_error_response.clone()));
            let cache = if *config.no_cache() {
                None
            } else {
                Some(Cache::load_from_file(CacheConfig::new(
                    oracle.location(),
                    Some(padding_error_response),
                ))?)
            };

            logic_main(
                &decryptor,
                &oracle,
                Arc::new(Mutex::new(cache)),
                encryption_mode,
                update_ui_callback.clone(),
                &config,
            )?;
        }
        OracleLocation::Script(_) => {
            info!(target: LOG_TARGET, "Using script oracle");
            let oracle = ScriptOracle::visit(config.oracle_location(), config.sub_config())?;
            let cache = if *config.no_cache() {
                None
            } else {
                Some(Cache::load_from_file(CacheConfig::new(
                    oracle.location(),
                    None,
                ))?)
            };

            logic_main(
                &decryptor,
                &oracle,
                Arc::new(Mutex::new(cache)),
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

fn calibrate_web<U>(
    decryptor: &Decryptor<U>,
    mut update_ui_callback: U,
    config: &Config,
) -> Result<CalibrationResponse>
where
    U: FnMut(UiEvent) + Sync + Send + Clone,
{
    // draw UI already so user doesn't think application is dead during calibration
    (update_ui_callback)(UiEvent::Decryption(UiDecryptionEvent::InitDecryption(
        config.cypher_text().blocks().to_vec(),
    )));

    info!(target: LOG_TARGET, "Calibrating web oracle...");
    let web_calibrator = decryptor.web_calibrator();
    let calibration_oracle =
        CalibrationWebOracle::visit(config.oracle_location(), config.sub_config())?;
    web_calibrator.determine_padding_error_response(calibration_oracle)
}

fn logic_main<U>(
    decryptor: &Decryptor<U>,
    oracle: &impl Oracle,
    cache: Arc<Mutex<Option<Cache>>>,
    encryption_mode: bool,
    mut update_ui_callback: U,
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
    let decryption_results = decryptor.decrypt_blocks(oracle, cache.clone())?;

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

        let encryptor = Encryptor::new(update_ui_callback.clone(), last_block);

        let encrypted_plain_text = encryptor
            .encrypt_plain_text(
                config
                    .plain_text()
                    .as_ref()
                    .expect("Should have a plain text in encryption mode"),
                oracle,
                cache,
            )?
            .encode();

        info!(
            target: LOG_TARGET,
            "The oracle talked some gibberish. It took {}",
            format_duration(Duration::new(now.elapsed().as_secs(), 0))
        );
        info!(
            target: LOG_TARGET,
            "Their divination is: {}", encrypted_plain_text
        );
        (update_ui_callback)(UiEvent::Control(UiControlEvent::PrintAfterExit(
            encrypted_plain_text,
        )));
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
        (update_ui_callback)(UiEvent::Control(UiControlEvent::PrintAfterExit(
            plain_text_solution,
        )));
    };

    Ok(())
}
