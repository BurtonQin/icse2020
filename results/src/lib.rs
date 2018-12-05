#![feature(extern_prelude)]
#![feature(duration_as_u128)]
#![feature(rustc_private)]
#[macro_use] extern crate log;
extern crate env_logger;

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
pub mod calls;

use std::fs::DirBuilder;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::time::SystemTime;
use std::fs::read_dir;
use std::fmt::Write;
use std::path::Path;

pub static SUMMARY_FUNCTIONS_FILE_NAME: &'static str = "02_summary_functions";

pub static UNSAFE_CALLS: &'static str = "03_unsafe_calls";
pub static UNSAFE_CALLS_USER_ONLY: &'static str = "03_unsafe_calls_user_only";

pub static IMPLICIT_COARSE_OPTIMISTIC_FILENAME: &'static str = "10_coarse_opt_unsafe_in_call_tree";
pub static IMPLICIT_COARSE_PESSIMISTIC_FILENAME: &'static str = "10_coarse_pes_unsafe_in_call_tree";

pub static IMPLICIT_RTA_OPTIMISTIC_FILENAME: &'static str = "11_precise_opt_unsafe_in_call_tree";
pub static IMPLICIT_RTA_PESSIMISTIC_FILENAME: &'static str = "11_precise_pes_unsafe_in_call_tree";

//static IMPLICIT_TRAIT_FILENAME: &'static str = "11_unsafe_trait_safe_method_in_call_tree";

pub static FN_UNSAFETY_SOURCES_FILE_NAME: &'static str = "30_unsafe_fn";
pub static NO_REASON_FOR_UNSAFE: &'static str = "31_no_reason";

pub static BLOCK_UNSAFETY_SOURCES_FILE_NAME: &'static str = "40_unsafe_blocks";
pub static BLOCK_SUMMARY_BB: &'static str = "41_blocks_summary";

pub static UNSAFE_TRAITS_IMPLS: &'static str = "50_unsafe_traits_impls";
pub static UNSAFE_TRAITS: &'static str = "51_unsafe_traits";

#[derive(Serialize, Deserialize, Debug)]
pub enum Abi {
    Cdecl,
    Stdcall,
    Fastcall,
    Vectorcall,
    Thiscall,
    Aapcs,
    Win64,
    SysV64,
    PtxKernel,
    Msp430Interrupt,
    X86Interrupt,
    AmdGpuKernel,
    Rust,
    C,
    System,
    RustIntrinsic,
    RustCall,
    PlatformIntrinsic,
    Unadjusted,
}

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

    pub fn create_file(&self, analysis_name: &'static str) -> File {
        let mut filename = String::new();
        filename.push_str(analysis_name);
        write!(filename, "_{:?}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis());
        let file_path = self.get_path(filename);
        // create new file
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true) // if true overwrites the old file
            .open(file_path)
            .unwrap()
    }

    pub fn open_files(&self, analysis_name: &'static str) -> Vec<File> {
        let mut result = Vec::new();
        let dir_path: PathBuf = self.get_root_path_components().iter().collect();
        if let Ok(read_dir) = dir_path.read_dir() {
            for entry in read_dir {
                // check if entry is ./analysis_name_*
                if let Some(filename) = entry.unwrap().path().as_path().file_name() {
                    if filename.to_str().unwrap().to_string().starts_with(analysis_name) {
                        let file_path = dir_path.join(filename);
                        // create new file
                        result.push(OpenOptions::new()
                            .read(true)
                            .create(false)
                            .open(file_path)
                            .unwrap());
                    }
                }
            }
        } else {
            error!("Error reading dir {:?}", dir_path);
            //TODO search for a higher version
        }

        result
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

//    pub fn get_implicit_unsafe_coarse_opt_file(&self, save_old: bool) -> File {
//        self.open_file(IMPLICIT_COARSE_OPTIMISTIC_FILENAME, save_old)
//    }
//
//    pub fn get_implicit_unsafe_coarse_pes_file(&self, save_old: bool) -> File {
//        self.open_file(IMPLICIT_COARSE_PESSIMISTIC_FILENAME, save_old)
//    }
//
//    pub fn get_implicit_unsafe_precise_opt_file(&self, save_old: bool) -> File {
//        self.open_file(IMPLICIT_RTA_OPTIMISTIC_FILENAME, save_old)
//    }
//
//    pub fn get_implicit_unsafe_precise_pes_file(&self, save_old: bool) -> File {
//        self.open_file(IMPLICIT_RTA_PESSIMISTIC_FILENAME, save_old)
//    op

//    pub fn get_implicit_trait_unsafe_file(&self, save: bool) -> File {
//        self.open_file(IMPLICIT_TRAIT_FILENAME, save)
//    }
//
//    pub fn get_summary_functions_file(&self, save: bool) -> File {
//        self.open_file(SUMMARY_FUNCTIONS_FILE_NAME, save)
//    }
//
//    pub fn get_fn_unsafety_sources_file(&self, save: bool) -> File {
//        self.open_file(FN_UNSAFETY_SOURCES_FILE_NAME, save)
//    }
//
//    pub fn get_unsafe_calls_file(&self, save: bool) -> File {
//        self.open_file(UNSAFE_CALLS, save)
//    }
//
//    pub fn get_unsafe_calls_file_user_only(&self, save: bool) -> File {
//        self.open_file(UNSAFE_CALLS_USER_ONLY, save)
//    }
//
//    pub fn get_blocks_unsafety_sources_file(&self, save: bool) -> File {
//        self.open_file(BLOCK_UNSAFETY_SOURCES_FILE_NAME, save)
//    }
//
//    pub fn get_blocks_summary_file(&self, save: bool) -> File {
//        self.open_file(BLOCK_SUMMARY_BB, save)
//    }
//
//    pub fn get_no_reason_for_unsafety_file(&self, save: bool) -> File {
//        self.open_file(NO_REASON_FOR_UNSAFE, save)
//    }
//
//    pub fn get_unsafe_traits_impls_file(&self, save: bool) -> File {
//        self.open_file(UNSAFE_TRAITS_IMPLS, save)
//    }
//
//   pub fn get_unsafe_traits_file(&self, save: bool) -> File {
//        self.open_file(UNSAFE_TRAITS, save)
//    }
}
