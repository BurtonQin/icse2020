
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::fs::File;
use std::fs::DirBuilder;

pub static ROOT_DIR: &'static str = "/tmp/unsafe_analysis/analysis_results/";

pub struct FileOps {
    crate_name: String,
    crate_version: String,
    analysis_name: &'static str,
}

impl FileOps {
    pub fn new(crate_name: String,
               crate_version: String,
               analysis_name: &'static str) -> Self {
        FileOps { crate_name, crate_version, analysis_name }
    }

    pub fn ope_file(&self) -> File {
        let file_path = self.get_path(self.analysis_name.to_string());

        if file_path.as_path().exists() {
            // back-up old file if it exists
            let mut new_name = self.analysis_name.to_string();
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
        //        let (local_crate,version) = crate_name_and_version();
        //        [ROOT_DIR.to_string(), local_crate.to_string()
        //            , version.to_string()]
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
}
