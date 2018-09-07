use util;
use unsafety_sources::Source;

static SAFE_FUNCTIONS_FILENAME: &'static str = "00_safe_functions";
static UNSAFE_FUNCTIONS_FILENAME: &'static str = "01_unsafe_functions";
static SUMMARY_FUNCTIONS_FILE_NAME: &'static str = "02_summary_functions";
static FN_UNSAFETY_SOURCES_FILE_NAME: &'static str = "30_unsafe_fn";

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
    unsafe_no: usize,
    total: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnsafeFnUsafetySources {
    name: String,
    from_trait: bool,
    arguments: Vec<Argument>,
    sources: Vec<Source>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Argument {
    ty_node_id: String,
    kind: ArgumentKind,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ArgumentKind {
    RawPointer,
    UnsafeFunction,
}

impl UnsafeFnUsafetySources {
    pub fn new(name: String) -> Self {
        UnsafeFnUsafetySources {
            name,
            from_trait: false,
            arguments: Vec::new(),
            sources: Vec::new(),
        }
    }

    pub fn add_argument(&mut self, arg: Argument) {
        self.arguments.push(arg);
    }

    fn add_source(&mut self, source: Source) {
        self.sources.push(source);
    }
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
    util::FileOps::new( crate_name, crate_version, SUMMARY_FUNCTIONS_FILE_NAME)
}

pub fn get_fn_unsafety_sources_file(crate_name: String,
                                  crate_version: String) -> util::FileOps {
    util::FileOps::new( crate_name, crate_version, FN_UNSAFETY_SOURCES_FILE_NAME)
}