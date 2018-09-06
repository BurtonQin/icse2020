use cargo::Config;
use cargo::core::Workspace;
use cargo::ops;
use std::env;
use rustc::hir;
use rustc::lint::LateContext;
use std::path::PathBuf;
use std::fs::OpenOptions;
use unsafe_blocks::UnsafeInBody;
use print;
use fn_info::FnInfo;
use std::io::BufReader;
use std::io::BufRead;

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
    println!("CARGO_MANIFEST_DIR {:?}", env::var("CARGO_MANIFEST_DIR"));
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
                                , data: &mut Vec<FnInfo> ) -> Vec<UnsafeInBody> {
    let mut result = Vec::new();
    for crate_info in external_crates.iter() {
        let mut analysis = load_analysis(cx, crate_info, data);
        println!("Crate {:?} result {:?}", crate_info.name, analysis);
        result.append( &mut analysis );
    }
    result
}

fn load_analysis<'a, 'tcx>( cx: &'a LateContext<'a, 'tcx>, crate_info: &CrateInfo, data: &mut Vec<FnInfo> ) -> Vec<UnsafeInBody> {
    let mut external_calls = Vec::new();
    for ref fn_info in data.iter() {
        let iter =
            (*fn_info.external_calls())
            .iter()
            .filter(|x: &&(hir::def_id::CrateNum, String) | {
                let crate_name = cx.tcx.crate_name(x.0);
                crate_name == crate_info.name
            });
        for (_crate_num,fn_name) in iter {
            external_calls.push(fn_name);
        }
    }

//    println!("Crate {:?} calls {:?}", crate_info.name, external_calls);

    let mut result = Vec::new();

    if external_calls.len() > 0 {
        let path_comp = [print::ROOT_DIR.to_string()
            , crate_info.name.clone()
            , crate_info.version.clone()
            , UnsafeInBody::get_output_filename().to_string()
        ];
        let file_path: PathBuf = path_comp.iter().collect();

        println!("path {:?}", file_path);

        let file = OpenOptions::new()
            .read(true).open(file_path).unwrap();
        let mut reader = BufReader::new(file);
        //read line by line
        loop {
            let mut line = String::new();
            let len = reader.read_line(&mut line).expect("Error reading file");

//            println!("Processing crate {:?} line {:?}", crate_info.name, line);

            if len == 0 {
                //EOF reached
                break
            } else {
                //process line
                let trimmed_line = line.trim_right();
                let crate_info: UnsafeInBody = serde_json::from_str(&trimmed_line).unwrap();
//                println!("external func :{:?}", crate_info.fn_info);
                let mut _found = false;
                for &call in external_calls.iter() {
                    println!("call {:?} fn_info {:?}", call, crate_info.fn_info);
                    if call.as_str().ends_with( crate_info.fn_info.as_str() ) {
                        result.push(UnsafeInBody{
                            fn_info: call.to_string(),
                            has_unsafe: crate_info.has_unsafe } );
                        _found = true;
                        break
                    }
                }
                // TODO do something about found
            }
        }
    }
    //TODO: check result: if there are external_calls not in result then error
//    println!("external unsafety {:?}", result);
    result
}

