use std::{fmt::Display, str::FromStr};

use anyhow::{anyhow, Result};
use itertools::Itertools;

#[derive(Debug, Clone)]
pub enum EncodingOption {
    Auto,
    Hex,
    Base64,
    Base64Url,
}

impl EncodingOption {
    fn variants() -> &'static [Self] {
        &[Self::Auto, Self::Hex, Self::Base64, Self::Base64Url]
    }
}

impl Display for EncodingOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncodingOption::Auto => write!(f, "auto"),
            EncodingOption::Hex => write!(f, "hex"),
            EncodingOption::Base64 => write!(f, "base64"),
            EncodingOption::Base64Url => write!(f, "base64url"),
        }
    }
}

impl FromStr for EncodingOption {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self> {
        let input = input.to_lowercase();

        if input == "auto" {
            Ok(EncodingOption::Auto)
        } else if input == "hex" {
            Ok(EncodingOption::Hex)
        } else if input == "base64" {
            Ok(EncodingOption::Base64)
        } else if input == "base64url" {
            Ok(EncodingOption::Base64Url)
        } else {
            Err(anyhow!(
                "`{}` is not a supported encoding. Expected one of: [{}]",
                input,
                Self::variants()
                    .iter()
                    .map(|variant| variant.to_string())
                    .join(", ")
            ))
        }
    }
}
