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
use std::collections::HashMap;
use std::io::Write;


declare_lint!(pub HIDDEN_UNSAFE, Allow, "Unsafe analysis");

impl<'a, 'tcx> LintPass for ImplicitUnsafe {
    fn get_lints(&self) -> LintArray {
        lint_array!(HIDDEN_UNSAFE)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for ImplicitUnsafe {

    fn check_crate_post(&mut self, cx: &LateContext<'a, 'tcx>, _: &'tcx Crate) {
        let external_crates = deps::load_dependencies();
        // list of all normal and unsafe functions are available here
        calls::build_call_graph(&mut self.normal_functions, cx);
        calls::build_call_graph(&mut self.unsafe_functions, cx);
        // the call graph info is available here
        self.save_results(cx); // saves normal and unsafe functions info, and summary for RQ2

        let cnv = util::local_crate_name_and_version();
        let file_ops = results::FileOps::new(&cnv.0, &cnv.1);

        // implicit unsafe analysis
        let implicit_external =
            deps::load_all_analyses(cx, &external_crates, &mut self.normal_functions);
        let mut res1: Vec<(&FnInfo, UnsafeInBody)> =
            analysis::run_all(cx, &self.normal_functions, true);
        propagate_external(cx, &mut res1, &implicit_external);
        analysis::save_analysis::<UnsafeInBody>(
            &res1,
            &mut file_ops.get_implicit_unsafe_file(true),
        );

        // implicit unsafe from traits analysis
        let res2: Vec<(&FnInfo, UnsafeTraitSafeMethodInBody)> =
            analysis::run_all(cx, &self.normal_functions, true);
        analysis::save_analysis(&res2, &mut file_ops.get_implicit_trait_unsafe_file(true));

        // unsafety sources in unsafe functions
        let unsafe_fn_info: Vec<(&FnInfo, UnsafeFnUsafetySources)> =
            analysis::run_all(cx, &self.unsafe_functions, false);
        analysis::save_analysis(
            &unsafe_fn_info,
            &mut file_ops.get_fn_unsafety_sources_file(true),
        );
        // no reason unsafe functions
        let no_reason = unsafety_sources::collect_no_reason(cx, &unsafe_fn_info);
        analysis::save_analysis(&no_reason, &mut file_ops.get_no_reason_for_unsafety_file(true));

        // unsafety sources in unsafe blocks
        let safe_fn_info: Vec<(&FnInfo, BlockUnsafetySourcesAnalysis)> =
            analysis::run_all(cx, &self.normal_functions, false);

        debug!("Result: {:?}", safe_fn_info);

        // TODO fix this: the function name is saved twice
        analysis::save_analysis_with_fn_info(
            cx,
            &safe_fn_info,
            &mut file_ops.get_blocks_unsafety_sources_file(true),
        );

        // blocks summary
        let bb_summary: results::blocks::BlockSummary =
            block_summary::collect(analysis::run_all(cx, &self.normal_functions, false));
        analysis::save_summary_analysis(bb_summary, &mut file_ops.get_blocks_summary_file(true));
    }

    fn check_body(&mut self, cx: &LateContext<'a, 'tcx>, body: &'tcx hir::Body) {
        //need to find fn/method declaration of this body
        let owner_def_id = cx.tcx.hir.body_owner_def_id(body.id());
        if let Some(owner_node_id) = cx.tcx.hir.as_local_node_id(owner_def_id) {
            if is_fn_or_method(owner_node_id, cx) {
                if is_unsafe_fn(owner_node_id, cx) {
                    self.push_unsafe_fn_info(owner_node_id);
                } else {
                    if util::is_unsafe_method(owner_node_id, cx) {
                        self.push_unsafe_fn_info(owner_node_id);
                    } else {
                        self.push_normal_fn_info(owner_node_id);
                    }
                }
            }
        }
    }
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_late_lint_pass(box ImplicitUnsafe::new() as LateLintPassObject);
}

pub fn is_unsafe_fn<'a, 'tcx>(node_id: NodeId, cx: &LateContext<'a, 'tcx>) -> bool {
    let node = cx.tcx.hir.get(node_id);
    match node {
        hir::Node::Item(item) => {
            if let hir::ItemKind::Fn(ref _fn_decl, ref fn_header, _, _) = item.node {
                if let hir::Unsafety::Normal = fn_header.unsafety {
                    false
                } else {
                    true
                }
            } else {
                false
            }
        }
        _ => false,
    }
}

pub fn is_fn_or_method<'a, 'tcx>(node_id: NodeId, cx: &LateContext<'a, 'tcx>) -> bool {
    let node = cx.tcx.hir.get(node_id);
    match node {
        hir::Node::Item(_item) => true,
        hir::Node::ImplItem(ref _impl_item) => true,
        hir::Node::Expr(ref _expr) => false, //closure
        hir::Node::AnonConst(ref _anon_const) => {
            // nothing to do - this is not a stand alone function
            // any unsafe in this body will be processed by the enclosing function or method
            false
        }
        hir::Node::TraitItem(ref trait_item) => {
            match trait_item.node {
                hir::TraitItemKind::Const(..)
                | hir::TraitItemKind::Type(..) => { info!("Not handled {:?}", node); false }
                hir::TraitItemKind::Method(ref _sig, ref _trait_method) => {
                    true
                }
            }
        }
        _ => {
            error!("Not handled {:?} ", node);
            false
        }
    }
}
