use unsafety_sources::Source;

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockSummary {
    pub in_unsafe_bb: usize,
    pub total_bb: usize,
    pub hir_unsafe_blocks: usize,
    pub hir_total: usize,
}

impl BlockSummary {
    pub fn new(
        in_unsafe_bb: usize,
        total_bb: usize,
        hir_unsafe_blocks: usize,
        hir_total: usize,
    ) -> Self {
        BlockSummary {
            in_unsafe_bb,
            total_bb,
            hir_unsafe_blocks,
            hir_total,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockUnsafetyAnalysisSources {
    sources: Vec<(String, Vec<Source>)>,
}

impl BlockUnsafetyAnalysisSources {
    pub fn new() -> Self {
        BlockUnsafetyAnalysisSources {
            sources: Vec::new(),
        }
    }

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
