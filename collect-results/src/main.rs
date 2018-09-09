#![feature(rustc_private)]
#![feature(extern_prelude)]

#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate serde;
extern crate serde_json;

extern crate results;

mod rq1;

use log::Level;
use std::fs::{File, DirEntry};
use std::path::Path;
use std::error::Error;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::fs::DirBuilder;

static OUTPUT_DIR: &'static str = "/tmp/unsafe_analysis/research_questions";

fn main() {
    // create ouput dir if it does not exists
    let dir_path: PathBuf = [OUTPUT_DIR].iter().collect();
    DirBuilder::new().recursive(true).create(dir_path).unwrap();
    // logger
    env_logger::init();
    // consider only the most recent version of each crate
    let crates = get_crates_recent_versions();
    rq1::process_rq1(&crates);

}

pub fn get_output_file( name: &'static str ) -> File {
    let path : PathBuf = [OUTPUT_DIR,name].iter().collect();
    debug!("{:?}", path);
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .unwrap()
}

fn get_crates_recent_versions() -> Vec<(String,String)> {
    let mut res = Vec::new();
    let root_dir = std::fs::read_dir(results::ROOT_DIR).unwrap();
    for dir_result in root_dir {
        let d = dir_result.unwrap();
        let path_buf = d.path();
        let crate_name = path_buf.file_name().unwrap().to_owned();
        let version = std::fs::read_dir(d.path()).unwrap().filter_map(
            |dir_result| {
                let dd = dir_result.unwrap();
                let pb = &dd.path();
                if let Some (name) = pb.file_name() {
                    Some (name.to_os_string())
                } else {
                    None
                }

            }
        ).max();
        res.push((crate_name.to_str().unwrap().to_string()
                  ,version.unwrap().to_str().unwrap().to_string()));
    }
    res
}
