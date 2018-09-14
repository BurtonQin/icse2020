use unsafety_sources::Source;

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockSummary {
    pub unsafe_blocks: usize,
}

impl BlockSummary {
    pub fn new( unsafe_blocks: usize) -> Self {
        BlockSummary {
            unsafe_blocks
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockUnsafetySourcesAnalysis {
    sources: Vec<(String, Vec<Source>)>,
}

impl BlockUnsafetySourcesAnalysis {
    pub fn new() -> Self {
        BlockUnsafetySourcesAnalysis {
            sources: Vec::new(),
        }
    }

    pub fn sources(&self) -> &Vec<(String, Vec<Source>)> { &&self.sources }

    pub fn add_source(&mut self, block_id: String, source: Source) {
        let found = self.sources.iter().any(|(node_id, _)| *node_id == block_id);
        if found {
            for (ref mut node_id, ref mut block_sources) in self.sources.iter_mut() {
                if *node_id == block_id {
                    block_sources.push(source);
                    break; // TODO change to while
                }
            }
        } else {
            let mut block_sources = Vec::new();
            block_sources.push(source);
            self.sources.push((block_id, block_sources));
        }
    }
}
