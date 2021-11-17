pub mod solved;

use getset::Getters;

use crate::block::block_size::{BlockSize, BlockSizeTrait};

use self::solved::SolvedForgedCypherText;

use super::{AmountBlocksTrait, Block, CypherText, Encode, Encoding};

pub enum ByteLockResult<'a> {
    BytesLeft(ForgedCypherText<'a>),
    Solved(SolvedForgedCypherText<'a>),
}

#[derive(Debug, Clone, Getters)]
pub struct ForgedCypherText<'a> {
    original_blocks: &'a [Block],
    url_encoded: bool,
    used_encoding: Encoding,

    current_byte_idx: u8,
    #[getset(get = "pub")]
    forged_block_wip: Block,
    forged_block_solution: Block,
}

impl<'a> ForgedCypherText<'a> {
    pub fn from_cypher_text(cypher_text: &'a CypherText, block_to_decrypt_idx: usize) -> Self {
        if block_to_decrypt_idx > cypher_text.amount_blocks() - 1 {
            panic!(
                "Tried to create ForgedCypherText to decrypt block {}, but only {} blocks exist in the original cypher text",
                block_to_decrypt_idx + 1,
                cypher_text.amount_blocks()
            );
        }

        let original_blocks = &cypher_text.blocks()[..block_to_decrypt_idx + 1];

        let block_size = cypher_text.block_size();
        let forged_cypher_text = Self {
            original_blocks,
            url_encoded: *cypher_text.url_encoded(),
            used_encoding: *cypher_text.used_encoding(),
            current_byte_idx: *block_size - 1,
            forged_block_wip: Block::new(&block_size),
            forged_block_solution: Block::new(&block_size),
        };

        forged_cypher_text
    }

    pub fn from_slice(
        original_blocks: &'a [Block],
        block_size: BlockSize,
        url_encoded: bool,
        used_encoding: Encoding,
    ) -> Self {
        Self {
            original_blocks,
            url_encoded,
            used_encoding,
            current_byte_idx: *block_size - 1,
            forged_block_wip: Block::new(&block_size),
            forged_block_solution: Block::new(&block_size),
        }
    }

    pub fn set_current_byte(&mut self, value: u8) -> &mut Self {
        self.forged_block_wip
            .set_byte(self.current_byte_idx as usize, value);

        self
    }

    /// Indicate that the current byte's value was found. Advance and save the solution.
    pub fn lock_byte(mut self) -> ByteLockResult<'a> {
        let idx = self.current_byte_idx;

        // locking a byte means it's supposedly correct
        self.forged_block_solution[idx as usize] = self.forged_block_wip[idx as usize];

        // we just solved the last (L<-R) byte so the whole block is solved
        if idx == 0 {
            ByteLockResult::Solved(SolvedForgedCypherText::from(self))
        } else {
            self.current_byte_idx = idx - 1;
            ByteLockResult::BytesLeft(self)
        }
    }

    pub fn bytes_answered(&self) -> u8 {
        (*self.block_size() - 1) - self.current_byte_idx
    }
}

impl<'a> Encode<'a> for ForgedCypherText<'a> {
    type Blocks = &'a [Block];

    fn encode(&'a self) -> String {
        // exclude forge-able block and block to decrypt
        let prefix_blocks = &self.blocks()[..self.amount_blocks() - 2];
        let to_decrypt_block = &self.blocks()[self.amount_blocks() - 1];

        // PKCS5/7 padding's value is the same as its length. So the desired padding when testing for the last byte is 0x01. But when testing the 2nd last byte, the last byte must be 0x02. This means that when moving on to the next byte (right to left), all of the previous bytes' solutions must be adjusted.
        let forged_block_with_padding_adjusted = self
            .forged_block_wip
            .to_adjusted_for_padding(*self.block_size() - self.current_byte_idx as u8);

        let raw_bytes: Vec<u8> = prefix_blocks.iter()
            .chain([&forged_block_with_padding_adjusted].into_iter())
            .chain([to_decrypt_block].into_iter())
            .map(|block| &**block)
            .flatten()
            // blocks are scattered through memory, gotta collect them
            .cloned()
            .collect();

        let encoded_data = match self.used_encoding() {
            Encoding::Base64 => base64::encode_config(raw_bytes, base64::STANDARD),
            Encoding::Base64Web => base64::encode_config(raw_bytes, base64::URL_SAFE),
            Encoding::Hex => hex::encode(raw_bytes),
        };

        if *self.url_encoded() {
            urlencoding::encode(&encoded_data).to_string()
        } else {
            encoded_data
        }
    }

    fn blocks(&'a self) -> Self::Blocks {
        self.original_blocks
    }

    fn url_encoded(&self) -> &bool {
        &self.url_encoded
    }

    fn used_encoding(&self) -> &Encoding {
        &self.used_encoding
    }
}

impl<'a> BlockSizeTrait for ForgedCypherText<'a> {
    fn block_size(&self) -> BlockSize {
        self.blocks()[0].block_size()
    }
}

impl<'a> AmountBlocksTrait for ForgedCypherText<'a> {
    fn amount_blocks(&self) -> usize {
        self.blocks().len()
    }
}
