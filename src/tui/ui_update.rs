use crate::block::Block;

pub enum UiUpdate {
    ForgedBlock((Block, usize)), // (forged block, block to decrypt idx)
    // for WIP updates, doesn't block on mutex
    ForgedBlockWip((Block, usize)), // (forged block, block to decrypt idx)
    ProgressUpdate,
    SlowRedraw,
    Done,
}
