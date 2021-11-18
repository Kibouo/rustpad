use crate::block::Block;

#[derive(Debug)]
pub enum UiEvent {
    Decryption(UiDecryptionEvent),
    Encryption(UiEncryptionEvent),
    Control(UiControlEvent),
}

#[derive(Debug)]
pub enum UiDecryptionEvent {
    // original_cypher_text_blocks
    InitDecryption(Vec<Block>),
    // (forged_block, cypher_text_block_idx)
    BlockSolved(Block, usize),
    // for WIP updates, doesn't block on mutex
    // (forged_block, cypher_text_block_idx)
    BlockWip(Block, usize),
}

#[derive(Debug)]
pub enum UiEncryptionEvent {
    // (plain_text_blocks, init_cypher_text)
    InitEncryption(Vec<Block>, Block),
    // (forged_block, cypher_text_block_idx)
    BlockSolved(Block, usize),
    // for WIP updates, doesn't block on mutex
    // (forged_block, cypher_text_block_idx)
    BlockWip(Block, usize),
}

#[derive(Debug)]
pub enum UiControlEvent {
    IndicateWork(usize),
    ProgressUpdate(usize), // inform UI that x bytes are solved
    PrintAfterExit(String),
    SlowRedraw,
}
