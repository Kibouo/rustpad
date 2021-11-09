#[derive(Debug)]
pub enum BlockSize {
    Eight,
    Sixteen,
    Auto,
}

impl From<&str> for BlockSize {
    fn from(data: &str) -> Self {
        match data {
            "auto" => Self::Auto,
            "8" => Self::Eight,
            "16" => Self::Sixteen,
            _ => unreachable!(format!("Invalid block size: {}", data)),
        }
    }
}
