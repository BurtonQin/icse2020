use std::fs::File;

use util;

static SAFE_FUNCTIONS_FILENAME: &'static str = "00_safe_functions";

#[derive(Serialize, Deserialize, Debug)]
pub struct LongFnInfo {
    name: String,
    node_id: String,
    location: String,
    // pairs (name,node_id)
    local_calls: Vec<(String,String)>,
    external_calls: Vec<String>,
}

pub fn get_safe_functions_file(crate_name: String,
                                crate_version: String
        ) -> util::FileOps {
    util::FileOps::new( crate_name, crate_version, SAFE_FUNCTIONS_FILENAME)
}