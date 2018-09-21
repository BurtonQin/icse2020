#![crate_name = "unsafe_analysis"]
#![crate_type = "dylib"]
#![feature(plugin_registrar)]
#![feature(rustc_private)]
#![feature(box_syntax)]
#![feature(macro_at_most_once_rep)]
#![feature(extern_prelude)]
#![feature(box_patterns)]

#[macro_use] extern crate log;
extern crate env_logger;

#[macro_use] extern crate rustc;
extern crate cargo;
extern crate cargo_metadata;
extern crate chrono;
extern crate results;
extern crate rustc_data_structures;
extern crate rustc_mir;
extern crate rustc_plugin;
extern crate rustc_target;
extern crate syntax;
extern crate syntax_pos;

extern crate serde;
//extern crate serde_derive;
extern crate serde_json;

use rustc::hir;
use rustc::hir::Crate;
use rustc::lint::{LateContext, LateLintPass, LateLintPassObject, LintArray, LintPass};
use rustc_plugin::Registry;
use syntax::ast::NodeId;
use syntax_pos::Span;
use std::io::Write;
use std::fs::File;
use std::path::Path;
use std::fmt::Write as FmtWrite;
use std::env;

mod blocks;
mod traits;
mod unsafety_sources;
mod functions;

declare_lint!(pub HIDDEN_UNSAFE, Allow, "Unsafe analysis");

struct Functions {
    normal_functions: Vec<NodeId>,
    unsafe_functions: Vec<NodeId>,
}

impl Functions {

    pub fn new() -> Self {
        Self {
            normal_functions: Vec::new(),
            unsafe_functions: Vec::new(),
        }
    }

    fn add(&mut self, node_id: NodeId, unsafety: hir::Unsafety) {
        match unsafety {
            hir::Unsafety::Normal => {self.normal_functions.push(node_id);}
            hir::Unsafety::Unsafe => {self.unsafe_functions.push(node_id);}
        }
    }
}

impl<'a, 'tcx> LintPass for Functions {
    fn get_lints(&self) -> LintArray {
        lint_array!(HIDDEN_UNSAFE)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for Functions {

    fn check_crate_post(&mut self, cx: &LateContext<'a, 'tcx>, _: &'tcx Crate) {

        let root_dir = match std::env::var("UNSAFE_ANALYSIS_DIR") {
            Ok (val) => {val.to_string()}
            Err (_) => {"/home/ans5k/unsafe_analysis/analysis_data".to_string()}
        };

        let cnv = local_crate_name_and_version();
        // safe functions
        let file_ops = results::FileOps::new(&cnv.0, &cnv.1, &root_dir);

        // blocks summary
        let bb_summary: results::blocks::BlockSummary = blocks::run_summary_analysis(cx);
        save_summary_analysis(bb_summary, &mut file_ops.get_blocks_summary_file(true));
        // unsafe functions summary
        let mut fn_summary_file = file_ops.get_summary_functions_file(true);
        save_summary_analysis(
            results::functions::Summary::new(
                self.unsafe_functions.len(),
                self.unsafe_functions.len() + self.normal_functions.len(),
            ),
            &mut fn_summary_file,
        );
        // unsafe traits
        let mut traits_file = file_ops.get_unsafe_traits_file(true);
        let unsafe_traits = traits::run_analysis(cx);
        save_summary_analysis(unsafe_traits, &mut traits_file);
        //unsafety sources in blocks
        let mut bus_file = file_ops.get_blocks_unsafety_sources_file(true);
        let bus_res = blocks::run_unsafety_sources_analysis(cx,&self.normal_functions);
        save_analysis(bus_res, &mut bus_file);
        //unsafety in sources
        let (fn_unsafety,no_reason) = functions::run_sources_analysis(cx,&self.unsafe_functions);
        save_analysis(fn_unsafety,&mut file_ops.get_fn_unsafety_sources_file(true));
        save_analysis(no_reason,&mut file_ops.get_no_reason_for_unsafety_file(true));
    }

    fn check_body(&mut self, cx: &LateContext<'a, 'tcx>, body: &'tcx hir::Body) {
        //need to find fn/method declaration of this body
        let owner_def_id = cx.tcx.hir.body_owner_def_id(body.id());
        if let Some(owner_node_id) = cx.tcx.hir.as_local_node_id(owner_def_id) {
            let node = cx.tcx.hir.get(owner_node_id);
            match node {
                hir::Node::Item(item) => {
                    if let hir::ItemKind::Fn(ref _fn_decl, ref fn_header, _, _) = item.node {
                        self.add(owner_node_id, fn_header.unsafety);
                    }
                },
                hir::Node::ImplItem(ref impl_item) => {
                    if let hir::ImplItemKind::Method(ref sig, _) = impl_item.node {
                        self.add(owner_node_id, sig.header.unsafety);
                    }
                }
                hir::Node::Expr(ref _expr) => {}//closure nothing to do
                hir::Node::AnonConst(ref _anon_const) => {
                    // nothing to do - this is not a stand alone function
                    // any unsafe in this body will be processed by the enclosing function or method
                }
                hir::Node::TraitItem(ref trait_item) => {
                    match trait_item.node {
                        hir::TraitItemKind::Const(..)
                        | hir::TraitItemKind::Type(..) => { }
                        hir::TraitItemKind::Method(ref sig, ref _trait_method) => {
                            self.add(owner_node_id, sig.header.unsafety);
                        }
                    }
                }
                _ => {
                    error!("Not handled {:?} ", node);
                }
            }
        }
    }
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_late_lint_pass(box Functions::new() as LateLintPassObject);
}

pub fn save_summary_analysis<T>(analysis_results: T, file: &mut File)
    where
        T: serde::ser::Serialize,
{
    let serialized = serde_json::to_string(&analysis_results).unwrap();
    writeln!(file, "{}", serialized);
}

pub fn save_analysis<T>(analysis_results: Vec<T>, file: &mut File)
    where
        T: serde::ser::Serialize,
{
    for res in analysis_results {
        save_summary_analysis(res,file);
    }
}

pub fn local_crate_name_and_version() -> (String, String) {
    let pkg = env::var("CARGO_PKG_NAME").unwrap();
    let version = env::var("CARGO_PKG_VERSION").unwrap();

//    error!("Local Package {:?} {:?}", pkg, version);

    (pkg,version)

//    let manifest_path = Path::new("./Cargo.toml");
//    let features = cargo_metadata::CargoOpt::AllFeatures;
//    match cargo_metadata::metadata_run(Some(manifest_path)
//                                       , false, Some(features)) {
//        Ok (metadata) => {
//            (metadata.packages[0].name.clone(), metadata.packages[0].version.clone())
//        }
//        Err (e) => {
//            error!("{:?}", e);
//            panic!("");
//        }
//    }
}

fn get_node_name<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, node_id: NodeId) -> String {
    cx.tcx.node_path_str(node_id)
}

pub fn get_file_and_line<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, span: Span) -> String {
    let mut result = String::new();
    let loc = cx.tcx.sess.source_map().lookup_char_pos(span.lo());
    let filename = &loc.file.name;
    write!(result, "file: {:?} line {:?}", filename, loc.line);
    result
}