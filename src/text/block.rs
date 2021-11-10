use std::ops::Deref;

#[derive(Debug)]
pub enum Block {
    Eight([u8; 8]),
    Sixteen([u8; 16]),
}

impl Deref for Block {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            Block::Eight(data) => data,
            Block::Sixteen(data) => data,
        }
    }
}
