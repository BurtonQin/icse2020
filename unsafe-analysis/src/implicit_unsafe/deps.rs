use std::collections::HashMap;

use cargo::core::Workspace;
use cargo::ops;
use cargo::Config;
use results::implicit::UnsafeInBody;
use rustc::hir;
use syntax::ast::NodeId;
use rustc::hir::def_id::DefId;
use rustc::lint::LateContext;
use std::env;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;
use std::collections::HashSet;
use std::path::Path;

pub fn load<'a, 'tcx>( cx: &'a LateContext<'a, 'tcx>, calls: &HashMap<String,DefId>, optimistic: bool)
    -> HashMap<DefId,UnsafeInBody> {
    let mut result = HashMap::new();
    let crates = load_dependencies( get_all_used_crates(cx,calls) );
    for crate_info in crates.values() {
        let mut analysis = load_analysis(cx, crate_info, calls, optimistic, &mut result);
        if let Ok(()) = analysis {
        } else {
            error!("Error processing crate {:?}", crate_info.name);
        }
    }
    for (fn_name,def_id) in calls.iter() {
        if is_excluded_crate( &cx.tcx.crate_name(def_id.krate).to_string() ) {
            //info!("Call {:?} from excluded crate", fn_name);
            result.insert(*def_id, UnsafeInBody::new(fn_name.clone(), false, fn_name.to_string()));
        } else {
            if !result.contains_key(def_id) {
                //info!("Call {:?} not found", fn_name);
                result.insert(*def_id, UnsafeInBody::new(fn_name.clone(), !optimistic, fn_name.to_string()));
            }
        }
    }
    result
}

fn get_all_used_crates<'a, 'tcx>( cx: &'a LateContext<'a, 'tcx>, calls: &HashMap<String,DefId>)-> HashSet<String> {
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

pub fn load_dependencies(used_crates:HashSet<String>) -> HashMap<String,CrateInfo> {
    let mut manifest_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    manifest_path.push("Cargo.toml");

    info!("manifest path {:?}", manifest_path);
    let mut result = HashMap::new();

    if manifest_path.as_path().exists() {
        match Config::default() {
            Ok(cargo_config) => {
                match Workspace::new(&manifest_path.as_path(), &cargo_config) {
                    Ok(workspace) => {
                        let resolve_res = ops::resolve_ws(&workspace);
                        if let Ok((packages, _resolve)) = resolve_res {
                            for package_id in packages.package_ids() {
                                if let Ok(package) = packages.get(package_id) {
                                    let crate_name = package.name().to_string().replace("-", "_");
                                    if let None = used_crates.get(&crate_name) {
                                        //info!("Crate not used {:?}", crate_name);
                                    } else {
                                        result.insert(package.name().to_string(), CrateInfo::new(
                                            crate_name,
                                            package.version().to_string(),
                                        ));
                                    }
                                } else {
                                    error!("Can't get package {:?}", package_id);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error loading workspace {:?}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to create default configuration {:?}", e);
            }
        }
    } else {
        error!("Cargo file does not exists! {:?}", manifest_path);
    }
    result

}


fn load_analysis<'a, 'tcx>(
    cx: &'a LateContext<'a, 'tcx>,
    crate_info: &CrateInfo,
    calls: &HashMap<String,DefId>,
    optimistic: bool,
    result: &mut HashMap<DefId,UnsafeInBody>
) -> Result<(), &'static str> {
    //filter external calls to this crate
    info!("Processing crate: {:?}", crate_info);
    let root_dir = ::get_root_dir();
    let dir_path: PathBuf = [&root_dir,&crate_info.name].iter().collect();
    //check if directory with crate name exists
    // if not try to replace _ with -
    let crate_name =
        if !Path::new(&dir_path).exists() {
            crate_info.name.replace("_", "-")
        } else {
            crate_info.name.clone()
        };

    let file_ops = results::FileOps::new(&crate_name, &crate_info.version, &root_dir);
    let file =
        if optimistic {
            file_ops.get_implicit_unsafe_coarse_opt_file(false)
        } else {
            file_ops.get_implicit_unsafe_coarse_pes_file(false)
        };
    info!("Processsing file {:?}", file_ops.get_root_path_components());
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
            info!("Processsing line {:?}", trimmed_line);
            let ub: UnsafeInBody = serde_json::from_str(&trimmed_line).unwrap();
            let def_path = ub.def_path;
            if let Some(def_id) = calls.get(&def_path) {
                info!("Call {:?} found", &def_path);
                result.insert(*def_id,UnsafeInBody::new(def_path,ub.has_unsafe,ub.name));
            }
        }
    }
    Ok(())
}

pub fn is_excluded_crate(crate_name: &String) -> bool {
    crate_name.as_str() == "alloc" || crate_name.as_str() == "std" || crate_name.as_str() == "core" || crate_name.as_str() == "proc_macro"
    //false
}
