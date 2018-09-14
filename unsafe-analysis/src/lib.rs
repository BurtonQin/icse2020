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
extern crate serde_derive;
extern crate serde_json;

use rustc::hir;
use rustc::hir::Crate;
use rustc::lint::{LateContext, LateLintPass, LateLintPassObject, LintArray, LintPass};
use rustc_plugin::Registry;
use syntax::ast::NodeId;
use std::io::Write;
use std::fs::File;
use std::path::Path;

mod blocks;
mod traits;

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
        let cnv = local_crate_name_and_version();
        // safe functions
        let file_ops = results::FileOps::new(&cnv.0, &cnv.1);

        // blocks summary
        let bb_summary: results::blocks::BlockSummary = blocks::run_analysis(cx);
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

pub fn local_crate_name_and_version() -> (String, String) {
    let manifest_path = Path::new("./Cargo.toml");
    let features = cargo_metadata::CargoOpt::AllFeatures;
    let metadata =
        cargo_metadata::metadata_run(Some(manifest_path), false, Some(features)).unwrap();

    //println!("Crate {:?} Version {:?}", metadata.packages[0].name.clone(),metadata.packages[0].version.clone());

    (
        metadata.packages[0].name.clone(),
        metadata.packages[0].version.clone(),
    )
}

fn get_node_name<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, node_id: NodeId) -> String {
    cx.tcx.node_path_str(node_id)
}