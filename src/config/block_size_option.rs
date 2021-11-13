// TODO: deprecate this & make positional args required flags
#[derive(Debug)]
pub enum BlockSizeOption {
    Eight,
    Sixteen,
    Auto,
}

impl From<&str> for BlockSizeOption {
    fn from(data: &str) -> Self {
        match data {
            "auto" => Self::Auto,
            "8" => Self::Eight,
            "16" => Self::Sixteen,
            _ => unreachable!(format!("Invalid block size: {}", data)),
        }
    }
}
