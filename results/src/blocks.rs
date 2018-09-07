use util;
use unsafety_sources::Source;

static BLOCK_UNSAFETY_SOURCES_FILE_NAME: &'static str = "40_unsafe_blocks";

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
        let found =  self.sources.iter().any( |(node_id,_)| *node_id == block_id );
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
            self.sources.push((block_id,block_sources));
        }
    }

}

pub fn get_blocks_unsafety_sources_file(crate_name: String,
                                    crate_version: String) -> util::FileOps {
    util::FileOps::new( crate_name, crate_version, BLOCK_UNSAFETY_SOURCES_FILE_NAME)
}