#[derive(Clone, Copy)]
pub enum BlockSize {
    Eight,
    Sixteen,
}

impl From<&str> for BlockSize {
    fn from(data: &str) -> Self {
        match data {
            "8" => Self::Eight,
            "16" => Self::Sixteen,
            _ => unreachable!(format!("Invalid block size: {}", data)),
        }
    }
}

impl From<BlockSize> for usize {
    fn from(block_size: BlockSize) -> Self {
        match block_size {
            BlockSize::Eight => 8,
            BlockSize::Sixteen => 16,
        }
    }
}
