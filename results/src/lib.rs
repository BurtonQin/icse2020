#![feature(extern_prelude)]

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate chrono;

pub mod implicit;
pub mod functions;
pub mod unsafety_sources;
pub mod blocks;

use std::path::PathBuf;
use std::fs::OpenOptions;
use std::fs::File;
use std::fs::DirBuilder;

static ROOT_DIR: &'static str = "/tmp/unsafe_analysis/analysis_results/";
static IMPLICIT_FILENAME: &'static str = "10_unsafe_in_call_tree";
static IMPLICIT_TRAIT_FILENAME: &'static str = "11_unsafe_trait_safe_method_in_call_tree";
static SAFE_FUNCTIONS_FILENAME: &'static str = "00_safe_functions";
static UNSAFE_FUNCTIONS_FILENAME: &'static str = "01_unsafe_functions";
static SUMMARY_FUNCTIONS_FILE_NAME: &'static str = "02_summary_functions";
static FN_UNSAFETY_SOURCES_FILE_NAME: &'static str = "30_unsafe_fn";
static EXTERNAL_CALLS_SUMMARY: &'static str = "03_external_calls_summary";
static BLOCK_UNSAFETY_SOURCES_FILE_NAME: &'static str = "40_unsafe_blocks";
static BLOCK_SUMMARY_BB: &'static str = "41_blocks_summary";

pub struct FileOps<'a> {
    crate_name: &'a String,
    crate_version: &'a String,
}

impl <'a> FileOps<'a> {
    pub fn new(crate_name: &'a String,
               crate_version: &'a String) -> Self {
        FileOps { crate_name, crate_version }
    }


    pub fn open_file(&self, analysis_name: &'static str, save_old: bool) -> File {
        let file_path = self.get_path(analysis_name.to_string());

        if file_path.as_path().exists() && save_old {
            // back-up old file if it exists
            let mut new_name = analysis_name.to_string();
            let dt = chrono::offset::utc::UTC::now();
            let newdate = dt.format("_%Y_%m_%d_%H_%M_%S");
            new_name.push_str(newdate.to_string().as_str());
            let new_path: PathBuf = self.get_path(new_name );
            std::fs::rename(file_path.as_path(), new_path).unwrap();
        }

        // create new file
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true).open(file_path).unwrap()
    }

    pub fn get_root_path_components(&self) -> [String; 3] {
        [ROOT_DIR.to_string()
            , self.crate_name.clone()
            , self.crate_version.clone()]
    }

    pub fn get_analysis_path_components(&self, filename: String) -> [String; 4] {
        [ROOT_DIR.to_string()
            , self.crate_name.clone()
            , self.crate_version.clone()
            , filename]
    }

    fn get_path(&self, filename: String) -> PathBuf {
        // create directory if necessary
        let dir_path: PathBuf = self.get_root_path_components().iter().collect();
        DirBuilder::new().recursive(true).create(dir_path).unwrap();

        let file_path: PathBuf = self.get_analysis_path_components(filename).iter().collect();
        file_path
    }

    pub fn get_implicit_unsafe_file(&self, save_old: bool) -> File {
        self.open_file(IMPLICIT_FILENAME, save_old)
    }

    pub fn get_implicit_trait_unsafe_file(&self) -> File {
        self.open_file(IMPLICIT_TRAIT_FILENAME, true)
    }

    pub fn get_safe_functions_file(&self) -> File {
        self.open_file(SAFE_FUNCTIONS_FILENAME, true)
    }

    pub fn get_unsafe_functions_file(&self) -> File {
        self.open_file(UNSAFE_FUNCTIONS_FILENAME, true)
    }

    pub fn get_summary_functions_file(&self) -> File {
        self.open_file(SUMMARY_FUNCTIONS_FILE_NAME, true)
    }

    pub fn get_fn_unsafety_sources_file(&self) -> File {
        self.open_file(FN_UNSAFETY_SOURCES_FILE_NAME, true)
    }

    pub fn get_external_calls_summary_file(&self) -> File {
        self.open_file(EXTERNAL_CALLS_SUMMARY, true)
    }

    pub fn get_blocks_unsafety_sources_file(&self) -> File {
        self.open_file(BLOCK_UNSAFETY_SOURCES_FILE_NAME, true)
    }

    pub fn get_blocks_summary_file(&self) -> File {
        self.open_file(BLOCK_SUMMARY_BB, true)
    }
}

