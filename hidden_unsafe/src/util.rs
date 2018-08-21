use syntax::ast::NodeId;

use rustc::hir;
use rustc::lint::LateContext;
use rustc::mir::Operand;
use rustc::ty::TypeVariants;
use rustc_target::spec::abi::Abi;

use syntax::codemap::Span;

use print::Print;
use std::fs::File;
use std::path::PathBuf;
use std::io::Write;
use print;
use std::fs::DirBuilder;
use chrono;
use std::fs::OpenOptions;
use std::path::Path;

pub enum FnCallInfo {
    Local(NodeId, Abi),
    External(hir::def_id::CrateNum, String, Abi),
}

impl Print for FnCallInfo {
    fn print<'a, 'tcx>(&self, cx: &LateContext<'a, 'tcx>, file: &mut File) -> () {
        match self {
            FnCallInfo::Local(node_id, abi) => {
                write!(file, "{:?} |abi: {:?}", cx.tcx.node_path_str(*node_id), abi);
            }
            FnCallInfo::External(krate, path_str, abi) => {
                write!(file,
                       "Crate: {:?} | Calee: {:?} | abi: {:?}",
                    cx.tcx.crate_name(*krate),
                    path_str,
                    abi
                );
            }
        }
    }
}

pub fn find_callee<'a, 'tcx>(
    cx: &LateContext<'a, 'tcx>,
    func: &Operand<'tcx>,
) -> Option<FnCallInfo> {
    if let Operand::Constant(constant) = func {
        if let TypeVariants::TyFnDef(callee_def_id, _) = constant.literal.ty.sty {
            let abi = cx.tcx.fn_sig(callee_def_id).abi();
            if callee_def_id.is_local() {
                if let Some(callee_node_id) = cx.tcx.hir.as_local_node_id(callee_def_id) {
                    Some(FnCallInfo::Local(callee_node_id, abi))
                } else {
                    println!("local node id NOT found {:?}", callee_def_id);
                    None
                }
            } else {
                let mut output = std::format!("{}", constant.literal.ty.sty);
                Some(FnCallInfo::External(callee_def_id.krate, output, abi))
            }
        } else {
            println!("TypeVariants NOT handled {:?}", constant.literal.ty.sty);
            None
        }
    } else {
        println!("find_callee::Operand Type NOT handled {:?}", func);
        None
    }
}


pub fn is_unsafe_fn<'a, 'tcx>(node_id: NodeId, cx: &LateContext<'a, 'tcx>) -> bool {
    let node = cx.tcx.hir.get(node_id);
    match node {
        hir::map::Node::NodeItem(item) => {
            if let hir::ItemKind::Fn(ref _fn_decl, ref fn_header, _, _) = item.node {
                if let hir::Unsafety::Normal = fn_header.unsafety {
                    false
                } else {
                    true
                }
            } else {
                //println!("Body owner node type NOT handled: {:?}", item);
                false
            }
        }
        _ => { false }
    }
}

pub fn is_unsafe_method<'a, 'tcx>(node_id: NodeId, cx: &LateContext<'a, 'tcx>) -> bool {
    let node = cx.tcx.hir.get(node_id);
    match node {
        hir::map::Node::NodeImplItem(ref impl_item) => {
            if let ::hir::ImplItemKind::Method(ref method_sig, ..) = impl_item.node {
                if let hir::Unsafety::Normal = method_sig.header.unsafety {
                    false
                } else {
                    true
                }
            } else {
                //println!("Impl Item Kind NOT handled {:?}", impl_item.node);
                false
            }
        }
        _ => { false }
    }
}

pub fn is_fn_or_method<'a, 'tcx>(node_id: NodeId, cx: &LateContext<'a, 'tcx>) -> bool {
    let node = cx.tcx.hir.get(node_id);
    match node {
        hir::map::Node::NodeItem(_item) => {true}
        hir::map::Node::NodeImplItem(ref _impl_item) => {true}
        hir::map::Node::NodeExpr(ref _expr) => {false} //closure
        hir::map::Node::NodeAnonConst(ref _anon_const) => {
            // nothing to do - this is not a stand alone function
            // any unsafe in this body will be processed by the enclosing function or method
            false
        }
        _ => {
            //println!("Body owner node NOT handled: {:?}", node);
            false
        }
    }
}

pub fn print_file_and_line<'a, 'tcx>( cx: &LateContext<'a, 'tcx>, span: Span, file: &mut File ) {
    let loc = cx.tcx.sess.codemap().lookup_char_pos(span.lo());
    let filename = &loc.file.name;
    write!(file,
           "file: {:?} line {:?} | ",
            filename,
            loc.line
    );
}

pub fn crate_name() -> String {
    let manifest_path = Path::new("./Cargo.toml");
    let features = cargo_metadata::CargoOpt::AllFeatures;
    let metadata =
        cargo_metadata::metadata_run(Some(manifest_path), false, Some(features)).unwrap();
    metadata.packages[0].name.clone()
}

pub fn open_file(analysis_name: &'static str) -> File {

    let local_crate = crate_name();

    // create directory if necessary
    let dir_path: PathBuf = [print::ROOT_DIR.to_string(), local_crate.to_string()].iter().collect();
    DirBuilder::new().recursive(true).create(dir_path).unwrap();

    let file_path: PathBuf = [print::ROOT_DIR.to_string()
                        , local_crate.to_string()
                        , analysis_name.to_string()].iter().collect();

    if file_path.as_path().exists() {
        // back-up old file if it exists
        let mut new_name = analysis_name.to_string();
        let dt = chrono::offset::utc::UTC::now();
        let newdate = dt.format("_%Y_%m_%d_%H_%M_%S");
        new_name.push_str(newdate.to_string().as_str());
        let new_path : PathBuf = [print::ROOT_DIR.to_string()
            , local_crate.to_string()
            , new_name].iter().collect();
        std::fs::rename(file_path.as_path(), new_path).unwrap();
    }

    // create new file
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true).open(file_path).unwrap()
}