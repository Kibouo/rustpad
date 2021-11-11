use std::ops::Deref;

use anyhow::anyhow;

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
        if !question.is_answered() {
            return Err(anyhow!(
                "Can't compute plaintext. Not all bytes of this block were locked"
            ));
        }

        let intermediate = question.tweakable_block_solution()
            ^ &Block::new_incremental_padding(&question.block_size());
        let intermediate = intermediate?;

        let plaintext = &intermediate ^ question.original_tweakable_block();
        let plaintext = plaintext?;

        Ok(Self(plaintext))
    }
}
