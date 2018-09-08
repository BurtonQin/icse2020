use cargo::Config;
use cargo::core::Workspace;
use cargo::ops;
use std::env;
use rustc::hir;
use rustc::lint::LateContext;
use std::path::PathBuf;
use results::implicit::UnsafeInBody;
use fn_info::FnInfo;
use std::io::BufReader;
use std::io::BufRead;
use util;

#[derive(Clone)]
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
//    println!("CARGO_MANIFEST_DIR {:?}", env::var("CARGO_MANIFEST_DIR"));
    let mut manifest_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    manifest_path.push("Cargo.toml");

    let cargo_config = Config::default().unwrap();


    let workspace = Workspace::new( &manifest_path.as_path(), &cargo_config ).unwrap();
    let (packages, _resolve) = ops::resolve_ws(&workspace).unwrap();

    let mut result = Vec::new();

    for package_id in packages.package_ids() {
        let package = packages.get(package_id).unwrap();
        result.push( CrateInfo::new(package.name().to_string(), package.version().to_string()) );
    }
    result
}

pub fn load_all_analyses<'a, 'tcx>( cx: &'a LateContext<'a, 'tcx>
                                , external_crates: &Vec<CrateInfo>
                                , data: &mut Vec<FnInfo> )
    -> Vec<(hir::def_id::CrateNum, Vec<UnsafeInBody>)> {
    let mut result = Vec::new();
    for crate_info in external_crates.iter() {
        let mut analysis = load_analysis(cx, crate_info, data);
        if let Some (crate_res) =  analysis.unwrap() {
            println!("Loaded analysis for crate {:?}: {:?}", crate_info.name, crate_res);
            result.push(crate_res);
        } else {
            println!("Loaded analysis for crate {:?}: EMPTY", crate_info.name);
        }
    }
    result
}

fn load_analysis<'a, 'tcx>( cx: &'a LateContext<'a, 'tcx>
                            , crate_info: &CrateInfo, data: &mut Vec<FnInfo> )
    -> Result<Option<(hir::def_id::CrateNum, Vec<UnsafeInBody>)>, &'static str> {

    println!("Loading analysis for crate {:?}", crate_info.name);

    let mut external_calls = Vec::new();
    let mut result = Vec::new();
    //find crate_num by name
    if let Some (crate_num) = cx.tcx.crates().iter().find(
        |&x| {
            crate_info.name == cx.tcx.crate_name(*x).to_string()
        }
    ) {
        //filter external calls to this crate
        for ref fn_info in data.iter() {
            let iter =
                (*fn_info.external_calls())
                    .iter()
                    .filter(|x: &&(hir::def_id::CrateNum, String)| {
                        x.0 == *crate_num
                    });
            for (_crate_num, fn_name) in iter {
                external_calls.push(fn_name);
            }
        }

        println!("External calls to this crate: {:?}", external_calls);

        if external_calls.len() > 0 {
            let file_ops = results::FileOps::new(&crate_info.name, &crate_info.version);
            let file = file_ops.get_implicit_unsafe_file(false);
            println!("Processsing file {:?}", file_ops.get_root_path_components() );
            let mut reader = BufReader::new(file);
            //read line by line
            loop {
                let mut line = String::new();
                let len = reader.read_line(&mut line).expect("Error reading file");
                if len == 0 {
                    //EOF reached
                    break
                } else {
                    //process line
                    let trimmed_line = line.trim_right();
                    let ub: UnsafeInBody = serde_json::from_str(&trimmed_line).unwrap();
                    let mut _found = false;
                    for &call in external_calls.iter() {
                        if call.as_str().ends_with(ub.fn_name.as_str()) {
                            result.push(UnsafeInBody {
                                fn_name: call.to_string(),
                                has_unsafe: ub.has_unsafe
                            });
                            _found = true;
                            break
                        }
                    }
                    // TODO do something about found
                }
            }
        }
        //TODO: check result: if there are external_calls not in result then error
        Ok (Some ((*crate_num, result)) )
    } else {

        if !util::is_excluded_crate(&crate_info.name) {
            //println!("Error: crate id NOT found for {:?}", crate_info.name);
            Ok (None)
        } else {
            println!("Crate is excluded {:?}", crate_info.name);
            Ok (None)
        }
    }
}

