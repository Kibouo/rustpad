mod block;
mod cli;
mod cypher_text;
mod oracle;
mod questioning;

use anyhow::Result;

use crate::{
    block::block_size::BlockSize,
    cli::{block_size_option::BlockSizeOption, Options, SubOptions},
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
    // TODO: rename to config to prevent confusion with std::option
    let mut options = Options::parse()?;

    let cypher_text = match options.block_size() {
        BlockSizeOption::Eight => CypherText::parse(options.cypher_text(), &BlockSize::Eight)?,
        BlockSizeOption::Sixteen => CypherText::parse(options.cypher_text(), &BlockSize::Sixteen)?,
        BlockSizeOption::Auto => todo!(),
    };

    let decoded = match options.oracle_location() {
        OracleLocation::Web(_) => {
            let mut questioning = Questioning::prepare(&cypher_text)?;

            let calibration_oracle =
                CalibrationWebOracle::visit(options.oracle_location(), options.sub_options())?;
            let padding_error_response = questioning.calibrate_web_oracle(calibration_oracle)?;

            if let SubOptions::Web(web_options) = options.sub_options_mut() {
                web_options.save_padding_error_response(padding_error_response);
            }

            let oracle = WebOracle::visit(options.oracle_location(), options.sub_options())?;
            questioning.start(oracle)?
        }
        OracleLocation::Script(_) => {
            let oracle = ScriptOracle::visit(options.oracle_location(), options.sub_options())?;
            Questioning::prepare(&cypher_text)?.start(oracle)?
        }
    };

    eprintln!("decoded = {:#?}", decoded);
    todo!();

    Ok(())
}
