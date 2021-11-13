mod block;
mod config;
mod cypher_text;
mod oracle;
mod questioning;

use anyhow::Result;

use crate::{
    block::block_size::BlockSize,
    config::{block_size_option::BlockSizeOption, Config, SubConfig},
    cypher_text::CypherText,
    oracle::{
        oracle_location::OracleLocation,
        script::ScriptOracle,
        web::{calibrate_web::CalibrationWebOracle, WebOracle},
        Oracle,
    },
    questioning::Questioning,
};

fn main() -> Result<()> {
    let mut config = Config::parse()?;

    let cypher_text = match config.block_size() {
        BlockSizeOption::Eight => CypherText::parse(config.cypher_text(), &BlockSize::Eight)?,
        BlockSizeOption::Sixteen => CypherText::parse(config.cypher_text(), &BlockSize::Sixteen)?,
        BlockSizeOption::Auto => todo!(),
    };

    let decoded = match config.oracle_location() {
        OracleLocation::Web(_) => {
            let mut questioning = Questioning::prepare(&cypher_text)?;

            let calibration_oracle =
                CalibrationWebOracle::visit(config.oracle_location(), config.sub_config())?;
            let padding_error_response = questioning.calibrate_web_oracle(calibration_oracle)?;

            if let SubConfig::Web(web_config) = config.sub_config_mut() {
                web_config.save_padding_error_response(padding_error_response);
            }

            let oracle = WebOracle::visit(config.oracle_location(), config.sub_config())?;
            questioning.start(oracle)?
        }
        OracleLocation::Script(_) => {
            let oracle = ScriptOracle::visit(config.oracle_location(), config.sub_config())?;
            Questioning::prepare(&cypher_text)?.start(oracle)?
        }
    };

    eprintln!("decoded = {:#?}", decoded);
    todo!();

    Ok(())
}
