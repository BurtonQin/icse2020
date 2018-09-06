use util;

static SAFE_FUNCTIONS_FILENAME: &'static str = "00_safe_functions";
static UNSAFE_FUNCTIONS_FILENAME: &'static str = "01_unsafe_functions";
static SUMMARY_FRUNCTIONS_FILE_NAME: &'static str = "02_summary_functions";

#[derive(Serialize, Deserialize, Debug)]
pub struct LongFnInfo {
    name: String,
    node_id: String,
    location: String,
    // pairs (name,node_id)
    local_calls: Vec<(String,String)>,
    external_calls: Vec<(String,Vec<String>)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShortFnInfo {
    name: String,
    node_id: String,
    location: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Summary{
    unsafe_no: u32,
    total: u32,
}

pub fn get_safe_functions_file(crate_name: String,
                                crate_version: String) -> util::FileOps {
    util::FileOps::new( crate_name, crate_version, SAFE_FUNCTIONS_FILENAME)
}

pub fn get_unsafe_functions_file(crate_name: String,
                               crate_version: String) -> util::FileOps {
    util::FileOps::new( crate_name, crate_version, UNSAFE_FUNCTIONS_FILENAME)
}

pub fn get_summary_functions_file(crate_name: String,
                                 crate_version: String) -> util::FileOps {
    util::FileOps::new( crate_name, crate_version, SUMMARY_FRUNCTIONS_FILE_NAME)
}