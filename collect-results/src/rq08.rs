use results;
use results::blocks;
use results::functions;
use std::io::BufReader;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Write;

struct AnalysisResult {
    pub name: String,
    pub crate_unsafe_blocks: uint,
    pub crate_unsafe_functions: uint,
    pub total_unsafe_blocks: uint,
    pub total_unsafe_functions: uint,
}

pub fn calculate_percentage() {
    // use the FULL_ANALYSIS folder
    let full_analysis_dir =
        match std::env::var("FULL_ANALYSIS_DIR") {
            Ok (val) => {val.to_string()}
            Err (_) => {"~/unsafe_analysis/analysis-data-2019/full-analysis".to_string()}
        };
    // for each directory in full_analysis dir
    for krate_dir_entry_res in std::fs::read_dir(full_analysis_dir).unwrap() {
        let krate_dir_entry = krate_dir_entry_res.unwrap();
        let krate = krate_dir_entry.file_name();
        error!("{:?}", );
    }
}