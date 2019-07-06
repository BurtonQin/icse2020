#![feature(rustc_private)]
#![feature(extern_prelude)]
#![feature(str_escape)]

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate chrono;
extern crate serde;
extern crate serde_json;

extern crate results;

mod rq01;
mod rq02;
mod rq03;
mod rq04;
mod rq05;
mod rq06;
mod rq07;

use std::fs::File ;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::fs::{DirBuilder,DirEntry};
use std::io::BufReader;
use std::io::BufRead;
use results::FileOps;
use std::collections::HashSet;

fn main() {
    // create ouput dir if it does not exists
    let root_dir = match std::env::var("RQ_DIR") {
        Ok (val) => {val.to_string()}
        Err (_) => {"/home/ans5k/unsafe_analysis/analysis-data/research-questions".to_string()}
    };
    let crates_file = match std::env::var("CRATES_FILE") {
        Ok (val) => { Some (val.to_string()) }
        Err (_) => { None }
    };
    let dir_path: PathBuf = [root_dir].iter().collect();
    DirBuilder::new().recursive(true).create(dir_path).unwrap();
    // logger
    env_logger::init();
    // consider only the most recent version of each crate
    let crates = get_crates_recent_versions(crates_file);
//    rq01::process_rq(&crates);
//    rq02::process_rq(&crates);
//    rq03::process_rq(&crates);
//    rq04::process_rq(&crates);
//    rq05::process_rq(&crates);
//    rq06::process_rq(&crates);
//    rq07::process_rq(&crates);

    external_unsafe();
}


pub fn get_output_file( name: &'static str ) -> File {
    let root_dir = match std::env::var("RQ_DIR") {
        Ok (val) => {val.to_string()}
        Err (_) => {"/home/ans5k/unsafe_analysis/analysis-data/research-questions".to_string()}
    };
    let path : PathBuf = [root_dir,name.to_string()].iter().collect();
    debug!("{:?}", path);
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .unwrap()
}

fn get_full_analysis_dir() -> String {
    match std::env::var("ANALYSIS_RESULTS_DIR") {
        Ok (val) => {val.to_string()}
        Err (_) => {"~/unsafe_analysis/analysis-data/results".to_string()}
    }
}

fn get_crates_recent_versions(file: Option<String>) -> Vec<(String,String)> {
    let mut res = Vec::new();
    let root_dir = std::fs::read_dir(get_full_analysis_dir()).unwrap();
    match file {
        Some(file_name) => {
            let file = File::open(file_name).unwrap();
            let mut reader = BufReader::new(file);
            loop {
                let mut line= String::new();
                let len = reader.read_line(&mut line).expect("Error reading file");
                let crate_name = line.trim_right();
                if len == 0 {
                    //EOF reached
                    break;
                } else {
                    //process line
                    let crate_path : PathBuf = [get_full_analysis_dir(), crate_name.to_string()].iter().collect();
                    if crate_path.exists() {
                        let version = FileOps::get_max_version(&crate_path);
                        res.push((crate_name.to_string(), version));
                    } else {
                        error!("Crate not found: {:?}", crate_name);
                    }
                }
            }
        }
        None => {
            for dir_result in root_dir {
                let d = dir_result.unwrap();
                let path_buf = d.path();
                let crate_name = path_buf.file_name().unwrap().to_owned();
                let version = FileOps::get_max_version(&path_buf);
                res.push((crate_name.to_str().unwrap().to_string()
                          , version));
            }
        }
    }
    res
}


fn external_unsafe() {
    let mut no_internal_unsafe = HashSet::new();
    for (crate_name, version) in crates {
        error!("Processing Crate {:?}", crate_name);
        let dir_name = ::get_full_analysis_dir();
        let file_ops = results::FileOps::new( crate_name, &version, &dir_name );
        if let Some (files) = file_ops.open_files(results::BLOCK_SUMMARY_BB) {
            if (files.is_empty()) {
                error!("No files for crate {:?}", crate_name);
                assert!(false);
            }
            for file in files.iter() {
                let mut reader = BufReader::new(file);
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
                            if block_summary.total == 0 {
                                no_internal_unsafe.insert(block_summary.crate_name);
                            }
                        }
                    }
                }
            }
        } else {
            error!("Block summary files missing for crate {:?}", crate_name);
        }
    }
}