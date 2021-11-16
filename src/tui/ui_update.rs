use crate::block::Block;

#[derive(Debug)]
pub enum UiEvent {
    ForgedBlockUpdate((Block, usize)), // (forged block, block to decrypt idx)
    // for WIP updates, doesn't block on mutex
    ForgedBlockWipUpdate((Block, usize)), // (forged block, block to decrypt idx)
    ProgressUpdate,
    SlowRedraw,
}
