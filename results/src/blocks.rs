use unsafety_sources::Source;

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockSummary {
    pub user_unsafe_blocks: usize,
    pub unsafe_blocks: usize,
    pub total: usize,
}

impl BlockSummary {
    pub fn new( user_unsafe_blocks: usize, unsafe_blocks: usize, total: usize) -> Self {
        BlockSummary {
            user_unsafe_blocks,
            unsafe_blocks,
            total,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockUnsafetySource {
    pub block_id: String,
    pub source: Source
}


