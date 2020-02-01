use results;
use results::blocks;
use results::functions;
use std::io::BufReader;
use std::io::BufRead;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;

struct AnalysisResult {
    pub name: String,
    pub crate_unsafe_blocks: usize,
    pub crate_unsafe_functions: usize,
    pub total_unsafe_blocks: usize,
    pub total_unsafe_functions: usize,
}

pub fn calculate_dependency_size() {
    // use the FULL_ANALYSIS folder
    let full_analysis_dir =
        match std::env::var("FULL_ANALYSIS_DIR") {
            Ok (val) => {val.to_string()}
            Err (_) => {"~/unsafe_analysis/analysis-data-2019/full-analysis".to_string()}
        };
    let mut crates_no = 0;
    let mut dependencies_no =0;
    for krate_dir_entry_res in std::fs::read_dir(full_analysis_dir).unwrap() {
        crates_no = crates_no + 1;
        let krate_dir_entry = krate_dir_entry_res.unwrap();
        for dependencies_dir_entry_res in std::fs::read_dir(krate_dir_entry.path()).unwrap() {
            let dependencies_dir_entry = dependencies_dir_entry_res.unwrap();
            if dependencies_dir_entry.file_name() != krate_dir_entry.file_name() {
                dependencies_no = dependencies_no + 1;
            }
        }
    }
    println!("dependencies per crate {}", dependencies_no/crates_no);
}


pub fn calculate_percentage() {
    // use the FULL_ANALYSIS folder
    let full_analysis_dir =
        match std::env::var("FULL_ANALYSIS_DIR") {
            Ok (val) => {val.to_string()}
            Err (_) => {"~/unsafe_analysis/analysis-data-2019/full-analysis".to_string()}
        };

    let mut from_dependency_only = 0;
    let mut both = 0;
    let mut total = 0;
    // for each directory in full_analysis dir
    for krate_dir_entry_res in std::fs::read_dir(full_analysis_dir).unwrap() {
        let krate_dir_entry = krate_dir_entry_res.unwrap();
        let krate = krate_dir_entry.file_name();
        let mut current_crate_summary = blocks::BlockSummary::new(0,0,0);
        let mut dependencies_summary = blocks::BlockSummary::new(0,0,0);
        let mut current_crate_fn_summary = results::functions::Summary::new(0,0);
        let mut dependencies_fn_summary = results::functions::Summary::new(0,0);
        for dependencies_dir_entry_res in std::fs::read_dir(krate_dir_entry.path()).unwrap() {
            //error!("{:?} depends on {:?}", krate, dependencies_dir_entry.unwrap().file_name());
            let dependencies_dir_entry = dependencies_dir_entry_res.unwrap();
            let block_summary = sum_for_crate_blocks(dependencies_dir_entry.path());
            if dependencies_dir_entry.file_name() == krate_dir_entry.file_name() {
                current_crate_summary.user_unsafe_blocks = block_summary.user_unsafe_blocks;
                current_crate_summary.unsafe_blocks = block_summary.unsafe_blocks;
                current_crate_summary.total = block_summary.total;
            } else {
                dependencies_summary.user_unsafe_blocks += block_summary.user_unsafe_blocks;
                dependencies_summary.unsafe_blocks += block_summary.unsafe_blocks;
                dependencies_summary.total += block_summary.total;
            }
            let fn_summary = sum_for_crate_fn(dependencies_dir_entry.path());
            if dependencies_dir_entry.file_name() == krate_dir_entry.file_name() {
                current_crate_fn_summary.unsafe_no = fn_summary.unsafe_no;
                current_crate_fn_summary.total = fn_summary.total;
            } else {
                dependencies_fn_summary.unsafe_no += fn_summary.unsafe_no;
                dependencies_fn_summary.total += fn_summary.total;
            }
        }
        if ((current_crate_summary.user_unsafe_blocks!=0 || current_crate_fn_summary.unsafe_no!=0)||
            dependencies_summary.user_unsafe_blocks != 0 || dependencies_fn_summary.total!= 0) {
            if current_crate_summary.user_unsafe_blocks == 0 &&
                current_crate_fn_summary.unsafe_no ==0 &&
                (dependencies_summary.user_unsafe_blocks > 0 || dependencies_fn_summary.unsafe_no >0) {
                from_dependency_only += 1;
            }
            if (current_crate_summary.user_unsafe_blocks > 0 ||
                current_crate_fn_summary.unsafe_no > 0) &&
                (dependencies_summary.user_unsafe_blocks > 0 || dependencies_fn_summary.unsafe_no >0) {
                both += 1;
            }

            total += 1;

            println!("blocks: {:?}\t{}\t{}\t{}\t{}",
                     krate,
                     current_crate_summary.user_unsafe_blocks,
                     dependencies_summary.user_unsafe_blocks,
                     current_crate_fn_summary.unsafe_no,
                     dependencies_fn_summary.unsafe_no
            );
        }

    }
    println!("blocks: from dependency only {} both {} total with some unsafe {}",
             from_dependency_only, both, total);
}

pub fn sum_for_crate_blocks( crate_path: PathBuf ) -> blocks::BlockSummary {
    let mut result = blocks::BlockSummary::new(0,0,0);
    // for each version
    for version in std::fs::read_dir(crate_path).unwrap() {
        //for each file
        for file_res in std::fs::read_dir(version.unwrap().path()).unwrap() {
            let file = file_res.unwrap();
            let filename_tmp = file.file_name();
            // check if it is a block summary file
            let filename = filename_tmp.to_str().unwrap();
            if ( filename.starts_with(results::BLOCK_SUMMARY_BB) ) {
                let file_tmp = OpenOptions::new()
                    .read(true)
                    .create(false)
                    .open(file.path())
                    .unwrap();
                let mut reader = BufReader::new(file_tmp);
                //read line by line
                loop {
                    let mut line = String::new();
                    let len = reader.read_line(&mut line).expect("Error reading file");
                    if len == 0 {
                        //EOF reached
                        break;
                    } else {
                        //process line
                        let trimmed_line = line.trim_right();
                        if trimmed_line.len() > 0 { // ignore empty lines
                            let block_summary: blocks::BlockSummary = serde_json::from_str(&trimmed_line).unwrap();
                            result.user_unsafe_blocks += block_summary.user_unsafe_blocks;
                            result.unsafe_blocks += block_summary.unsafe_blocks;
                            result.total += block_summary.total;
                        }
                    }

                }
            }
        }
    }
    result

}

pub fn sum_for_crate_fn( crate_path: PathBuf ) -> results::functions::Summary {
    let mut result = results::functions::Summary::new(0,0);
    // for each version
    for version in std::fs::read_dir(crate_path).unwrap() {
        //for each file
        for file_res in std::fs::read_dir(version.unwrap().path()).unwrap() {
            let file = file_res.unwrap();
            let filename_tmp = file.file_name();
            // check if it is a block summary file
            let filename = filename_tmp.to_str().unwrap();
            if ( filename.starts_with(results::SUMMARY_FUNCTIONS_FILE_NAME) ) {
                let file_tmp = OpenOptions::new()
                    .read(true)
                    .create(false)
                    .open(file.path())
                    .unwrap();
                let mut reader = BufReader::new(file_tmp);
                //read line by line
                loop {
                    let mut line = String::new();
                    let len = reader.read_line(&mut line).expect("Error reading file");
                    if len == 0 {
                        //EOF reached
                        break;
                    } else {
                        //process line
                        let trimmed_line = line.trim_right();
                        if trimmed_line.len() > 0 { // ignore empty lines
                            let summary: results::functions::Summary = serde_json::from_str(&trimmed_line).unwrap();
                            result.unsafe_no += summary.unsafe_no;
                            result.total += summary.total;
                        }
                    }

                }
            }
        }
    }
    result

}
