use cargo::Config;
use cargo::core::shell::Shell;
use cargo::core::Workspace;
use cargo::ops;
use std::env;
use rustc::hir;
use rustc::lint::LateContext;
use cargo::core::registry::PackageRegistry;
use std::path::{Path, PathBuf};
use std::fs::OpenOptions;
use unsafe_blocks::UnsafeInBody;
use print;
use fn_info::FnInfo;

pub struct CrateInfo {
    name: String,
    version: String,
}

impl CrateInfo {
    pub fn new( name: String, version: String ) -> Self {
        CrateInfo{ name, version }
    }
}

pub fn load_dependencies() -> Vec<CrateInfo> {
    println!("CARGO_MANIFEST_DIR {:?}", env::var("CARGO_MANIFEST_DIR"));
    let mut manifest_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    manifest_path.push("Cargo.toml");

    let mut cargo_config = Config::default().unwrap();


    let workspace = Workspace::new( &manifest_path.as_path(), &cargo_config ).unwrap();
    let (packages, resolve) = ops::resolve_ws(&workspace).unwrap();

    let mut result = Vec::new();

    for package_id in packages.package_ids() {
        let package = packages.get(package_id).unwrap();
        result.push( CrateInfo::new(package.name().to_string(), package.version().to_string()) );
    }
    result
}

pub fn load_analysis<'a, 'tcx>( cx: &'a LateContext<'a, 'tcx>, crate_info: &CrateInfo, data: &mut Vec<FnInfo> ) -> Vec<UnsafeInBody> {
    let mut external_calls = Vec::new();
    for ref fn_info in data.iter() {
        for &(crate_num,fn_name) in fn_info.external_calls()
                .iter().filter(|x: &(hir::def_id::CrateNum, String)| {
            let crate_name = cx.tcx.crate_name(x.0);
            crate_name == crate_info.name
        }) {
            external_calls.push(fn_name);
        }
    }

    println!("Crate {:?} calls {:?}", crate_info.name, external_calls);

    let mut result = Vec::new();
    let path_comp = [print::ROOT_DIR.to_string()
        , crate_info.name.clone()
        , crate_info.version.clone()
        , UnsafeInBody::get_output_filename().to_string()
    ];
    let file_path: PathBuf = path_comp.iter().collect();
    let mut file =  OpenOptions::new()
                    .read(true).open(file_path).unwrap();
    //read line by line

    result
}

