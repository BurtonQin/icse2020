#![feature(extern_prelude)]

#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate serde;
extern crate serde_json;

pub mod blocks;
pub mod functions;
pub mod implicit;
pub mod traits;
pub mod unsafety_sources;

use std::fs::DirBuilder;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::PathBuf;


//static SAFE_FUNCTIONS_FILENAME: &'static str = "00_safe_functions";
//static UNSAFE_FUNCTIONS_FILENAME: &'static str = "01_unsafe_functions";

static SUMMARY_FUNCTIONS_FILE_NAME: &'static str = "02_summary_functions";

static UNSAFE_CALLS: &'static str = "03_unsafe_calls";

static IMPLICIT_COARSE_OPTIMISTIC_FILENAME: &'static str = "10_coarse_opt_unsafe_in_call_tree";
static IMPLICIT_COARSE_PESSIMISTIC_FILENAME: &'static str = "10_coarse_pes_unsafe_in_call_tree";
static IMPLICIT_TRAIT_FILENAME: &'static str = "11_unsafe_trait_safe_method_in_call_tree";

static FN_UNSAFETY_SOURCES_FILE_NAME: &'static str = "30_unsafe_fn";
static NO_REASON_FOR_UNSAFE: &'static str = "31_no_reason";

static BLOCK_UNSAFETY_SOURCES_FILE_NAME: &'static str = "40_unsafe_blocks";
static BLOCK_SUMMARY_BB: &'static str = "41_blocks_summary";

static UNSAFE_TRAITS: &'static str = "50_unsafe_traits";

pub struct FileOps<'a,'b> {
    crate_name: &'a String,
    crate_version: &'a String,
    root_dir: &'b String
}

impl<'a, 'b> FileOps<'a, 'b> {
    pub fn new(crate_name: &'a String, crate_version: &'a String, root_dir: &'b String) -> Self {
        FileOps {
            crate_name,
            crate_version,
            root_dir,
        }
    }

    pub fn open_file(&self, analysis_name: &'static str, overwrite: bool) -> File {
        let file_path = self.get_path(analysis_name.to_string());
        // create new file
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(overwrite) // if true overwrites the old file
            .open(file_path)
            .unwrap()
    }

    pub fn get_root_path_components(&self) -> [String; 3] {
        [
            self.root_dir.to_string(),
            self.crate_name.clone(),
            self.crate_version.clone(),
        ]
    }

    pub fn get_analysis_path_components(&self, filename: String) -> [String; 4] {
        [
            self.root_dir.to_string(),
            self.crate_name.clone(),
            self.crate_version.clone(),
            filename,
        ]
    }

    fn get_path(&self, filename: String) -> PathBuf {
        // create directory if necessary
        let dir_path: PathBuf = self.get_root_path_components().iter().collect();
        DirBuilder::new().recursive(true).create(dir_path).unwrap();

        let file_path: PathBuf = self.get_analysis_path_components(filename).iter().collect();
        file_path
    }

    pub fn get_implicit_unsafe_coarse_opt_file(&self, save_old: bool) -> File {
        self.open_file(IMPLICIT_COARSE_OPTIMISTIC_FILENAME, save_old)
    }

    pub fn get_implicit_unsafe_coarse_pes_file(&self, save_old: bool) -> File {
        self.open_file(IMPLICIT_COARSE_PESSIMISTIC_FILENAME, save_old)
    }

    pub fn get_implicit_trait_unsafe_file(&self, save: bool) -> File {
        self.open_file(IMPLICIT_TRAIT_FILENAME, save)
    }

//    pub fn get_safe_functions_file(&self, save: bool) -> File {
//        self.open_file(SAFE_FUNCTIONS_FILENAME, save)
//    }
//
//    pub fn get_unsafe_functions_file(&self, save: bool) -> File {
//        self.open_file(UNSAFE_FUNCTIONS_FILENAME, save)
//    }

    pub fn get_summary_functions_file(&self, save: bool) -> File {
        self.open_file(SUMMARY_FUNCTIONS_FILE_NAME, save)
    }

    pub fn get_fn_unsafety_sources_file(&self, save: bool) -> File {
        self.open_file(FN_UNSAFETY_SOURCES_FILE_NAME, save)
    }

    pub fn get_unsafe_calls_file(&self, save: bool) -> File {
        self.open_file(UNSAFE_CALLS, save)
    }

    pub fn get_blocks_unsafety_sources_file(&self, save: bool) -> File {
        self.open_file(BLOCK_UNSAFETY_SOURCES_FILE_NAME, save)
    }

    pub fn get_blocks_summary_file(&self, save: bool) -> File {
        self.open_file(BLOCK_SUMMARY_BB, save)
    }

    pub fn get_no_reason_for_unsafety_file(&self, save: bool) -> File {
        self.open_file(NO_REASON_FOR_UNSAFE, save)
    }

    pub fn get_unsafe_traits_file(&self, save: bool) -> File {
        self.open_file(UNSAFE_TRAITS, save)
    }
}
