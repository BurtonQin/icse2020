use fxhash::FxHashMap;
use cargo::core::Workspace;
use cargo::ops;
use cargo::Config;
use results::implicit::UnsafeInBody;
use rustc::hir::def_id::DefId;
use rustc::lint::LateContext;
use std::env;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;
use std::collections::HashSet;
use std::path::Path;
use implicit_unsafe::is_library_crate;
use results::implicit::FnType;
use results::FileOps;

pub fn load<'a, 'tcx>( cx: &'a LateContext<'a, 'tcx>, calls: &FxHashMap<String,DefId>, optimistic: bool)
    -> FxHashMap<DefId,UnsafeInBody> {
    let mut result = FxHashMap::default();
    let crates = load_dependencies( get_all_used_crates(cx,calls) );
    for crate_info in crates.values() {
        let mut analysis = load_analysis(cx, crate_info, calls, optimistic, &mut result);
        let mut analysis = load_analysis(cx, crate_info, calls, optimistic, &mut result);
        if let Ok(()) = analysis {
        } else {
            error!("Error processing crate {:?}", crate_info.name);
        }
    }
    let mut not_found = 0;
    for (fn_name,def_id) in calls.iter() {
        if is_library_crate( &cx.tcx.crate_name(def_id.krate).to_string() ) {
            //info!("Call {:?} from excluded crate", fn_name);
            result.insert(*def_id, UnsafeInBody::new(fn_name.clone(), FnType::Safe, fn_name.to_string()));
        } else {
            if !result.contains_key(def_id) {
                not_found += 1;
                //info!("Call {:?} not found", fn_name);
                result.insert(*def_id, UnsafeInBody::new(fn_name.clone(),
                                                         if optimistic {
                                                             FnType::Safe
                                                         } else {
                                                             FnType::NormalNotSafe
                                                         }
                                                         , fn_name.to_string()));
            }
        }
    }
    info!("External Calls {:?} NOT Found {:?}", calls.len(), not_found);
    result
}

fn get_all_used_crates<'a, 'tcx>( cx: &'a LateContext<'a, 'tcx>, calls: &FxHashMap<String,DefId>)-> HashSet<String> {
    let mut result = HashSet::new();
    for &def_id in calls.values() {
        let crate_num = def_id.krate;
        let crate_name = cx.tcx.crate_name(crate_num);
        result.insert(crate_name.to_string());
    }
    result
}

#[derive(Clone,Debug)]
pub struct CrateInfo {
    name: String,
    version: String,
}

impl CrateInfo {
    pub fn new(name: String, version: String) -> Self {
        CrateInfo { name, version }
    }
}

pub fn load_dependencies(used_crates:HashSet<String>) -> FxHashMap<String,CrateInfo> {
    let mut manifest_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    manifest_path.push("Cargo.toml");
    let mut result = FxHashMap::default();

    if manifest_path.as_path().exists() {
        match  cargo_metadata::metadata(Some(&manifest_path)) {
            Err(s) => error!("cargo_metadata::metadata" + s),
            Ok(metadata) => {
                for package in metadata.packages {
                    let crate_name = package.name.to_string().replace("-", "_");
                    if let None = used_crates.get(&crate_name) {
                        info!("Crate not used {:?}", crate_name);
                    } else {
                        result.insert(package.name.to_string(), CrateInfo::new(
                            crate_name,
                            package.version.to_string(),
                        ));
                    }
                }
            },
        }
    } else {
        error!("Cargo file does not exists! {:?}", manifest_path);
    }
    result

}


fn load_analysis<'a, 'tcx>(
    cx: &'a LateContext<'a, 'tcx>,
    crate_info: &CrateInfo,
    calls: &FxHashMap<String,DefId>,
    optimistic: bool,
    result: &mut FxHashMap<DefId,UnsafeInBody>
) -> Result<(), &'static str> {
    //filter external calls to this crate
//    info!("Processing crate: {:?}", crate_info);
    let root_dir = ::get_root_dir();
    let dir_path: PathBuf = [&root_dir,&crate_info.name].iter().collect();
    let crate_name =
        if !Path::new(&dir_path).exists() {
            // TODO look for directory that matches crate name modulo _ and -
            let crate_comps: Vec<&str> = crate_info.name.split(|c| c=='_' || c=='-').collect();
            let mut result = None;
            if let Ok(dir_entries) = std::fs::read_dir(&root_dir) {
                for dir in dir_entries {
                    if let Ok (dir) = dir {
                        let dir_name = dir.file_name().into_string().unwrap();
                        let comps: Vec<&str> = dir_name.split(|c| c=='_' || c=='-').collect();
                        if comps == crate_comps {
                            result = Some (dir.file_name().into_string().unwrap());
                            break
                        }
                    } else {
                        assert!(false, "Error dir entry {:?}", dir);
                    }
                }
                result
            } else {
                error!("Can not read root dir {:}", root_dir);
                None
            }
        } else {
            Some (crate_info.name.clone())
        };

    if let Some (crate_name) = crate_name {
        let crate_path: PathBuf = [&root_dir, &crate_name].iter().collect();
        let version_path: PathBuf = [&root_dir, &crate_name, &crate_info.version].iter().collect();
        let version = FileOps::get_max_version(&crate_path); // here to satisfy lifetime

        info!("Want version {:?} max version {:?}", crate_info.version, version);

        // always use max version
        let file_ops = results::FileOps::new(&crate_name, &version, &root_dir);
//            if Path::new(&version_path).exists() {
//                results::FileOps::new(&crate_name, &crate_info.version, &root_dir)
//            } else {
//                error!("Dir does not exists {:?}, using version {:?}", crate_path, version);
//                results::FileOps::new(&crate_name, &version, &root_dir)
//            };
        let files =
                if optimistic {
                    file_ops.open_files(results::IMPLICIT_RTA_OPTIMISTIC_FILENAME)
                } else {
                    file_ops.open_files(results::IMPLICIT_RTA_PESSIMISTIC_FILENAME)
                };

        if let Some(files) = files {
            for file in files.iter() {
                info!("Processsing file {:?}", file);
                let mut reader = BufReader::new(file);
                //read line by line
                loop {
                    let mut line = String::new();
                    let len = reader.read_line(&mut line).expect("Error reading file");
                    if len == 0 {
                        //EOF reached
                        break;
                    } else {
                        //process line
                        let trimmed_line = line.trim_right();
                        //info!("Processsing line {:?}", trimmed_line);
                        let res: serde_json::Result<UnsafeInBody> = serde_json::from_str(&trimmed_line);
                        match res {
                            Ok(ub) => {
                                let def_path = ub.def_path;
                                if let Some(def_id) = calls.get(&def_path) {
                                    //info!("Call {:?} found", &def_path);
                                    result.insert(*def_id, UnsafeInBody::new(def_path, ub.fn_type, ub.name));
                                } else {
                                    //info!("Call {:?} NOT found", &def_path);
                                }
                            }
                            Err(e) => {
                                error!("Error processing line {:?} file: {:?}", trimmed_line, &file_ops.get_root_path_components());
                                assert!(false); // I want to detect the corrupt files
                            }
                        }
                    }
                }
            }
        } else {
            error!("Dir not found for crate {:?}", &crate_name);
        }
        Ok(())
    } else {
        assert!(false, "Directory not found for crate {:}", crate_info.name);
        Err("Not Found")
    }
}

