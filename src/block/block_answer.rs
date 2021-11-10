use std::ops::Deref;

use super::{block_question::BlockQuestion, Block};

#[derive(Debug)]
pub struct BlockAnswer(Block);

impl Deref for BlockAnswer {
    type Target = Block;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&mut BlockQuestion> for BlockAnswer {
    type Error = anyhow::Error;

    fn try_from(question: &mut BlockQuestion) -> Result<Self, Self::Error> {
        let intermediate = question.tweakable_block_solution()
            ^ &Block::incremental_padding(&question.block_size());
        let intermediate = intermediate?;

        let plaintext = &intermediate ^ question.original_tweakable_block();
        let plaintext = plaintext?;

        Ok(Self(plaintext))
    }
}
