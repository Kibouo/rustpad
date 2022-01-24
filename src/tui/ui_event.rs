use crate::block::Block;

#[derive(Debug)]
pub(crate) enum UiEvent {
    Decryption(UiDecryptionEvent),
    Encryption(UiEncryptionEvent),
    Control(UiControlEvent),
}

#[derive(Debug)]
pub(crate) enum UiDecryptionEvent {
    // original_cypher_text_blocks
    InitDecryption(Vec<Block>),
    // (forged_block, cypher_text_block_idx)
    BlockSolved(Block, usize),
    // for WIP updates, doesn't block on mutex
    // (forged_block, cypher_text_block_idx)
    BlockWip(Block, usize),
}

#[derive(Debug)]
pub(crate) enum UiEncryptionEvent {
    // (plain_text_blocks, init_cypher_text)
    InitEncryption(Vec<Block>, Block),
    // (forged_block, cypher_text_block_idx)
    BlockSolved(Block, usize),
    // for WIP updates, doesn't block on mutex
    // (forged_block, cypher_text_block_idx)
    BlockWip(Block, usize),
}

#[derive(Debug)]
pub(crate) enum UiControlEvent {
    IndicateWork(usize),
    ProgressUpdate(usize), // inform UI that x bytes are solved
    PrintAfterExit(String),
    ExitCode(i32),
    /// The application is done. Basically indicates that the program should stop running, without actually quitting. This keeps the UI open for users to read the output, while also decreasing the amount of draw calls.
    SlowRedraw,
}
