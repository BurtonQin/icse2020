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

use fn_info::FnInfo;
use rustc::hir;
use rustc::hir::Crate;
use rustc::lint::{LateContext, LateLintPass, LateLintPassObject, LintArray, LintPass};
use rustc_plugin::Registry;
use syntax::ast::NodeId;

mod analysis;
mod block_summary;
mod calls;
mod deps;
mod fn_info;
mod implicit_analysis;
mod traits;
mod unsafety_sources;
mod util;

use implicit_analysis::propagate_external;
use results::blocks::BlockUnsafetySourcesAnalysis;
use results::functions::UnsafeFnUsafetySources;
use results::implicit::UnsafeInBody;
use results::implicit::UnsafeTraitSafeMethodInBody;

use std::collections::HashMap;
use std::io::Write;

struct ImplicitUnsafe {
    normal_functions: Vec<FnInfo>,
    unsafe_functions: Vec<FnInfo>,
}

impl ImplicitUnsafe {
    pub fn new() -> Self {
        Self {
            normal_functions: Vec::new(),
            unsafe_functions: Vec::new(),
        }
    }

    pub fn push_normal_fn_info<'a, 'tcx>(&mut self, node_id: NodeId) {
        let fn_info = FnInfo::new(node_id);
        self.normal_functions.push(fn_info);
    }

    pub fn push_unsafe_fn_info<'a, 'tcx>(&mut self, node_id: NodeId) {
        let fn_info = FnInfo::new(node_id);
        self.unsafe_functions.push(fn_info);
    }

    pub fn save_results<'a, 'tcx>(&self, cx: &'a LateContext<'a, 'tcx>) {
        let cnv = util::local_crate_name_and_version();
        // safe functions
        let file_ops = results::FileOps::new(&cnv.0, &cnv.1);
        let mut safe_file = file_ops.get_safe_functions_file(true);
        for ref fn_info in self.normal_functions.iter() {
            let long_form = fn_info.build_long_fn_info(cx);
            writeln!(safe_file, "{}", serde_json::to_string(&long_form).unwrap());
        }
        // unsafe functions
        let mut unsafe_file = file_ops.get_unsafe_functions_file(true);
        for ref fn_info in self.unsafe_functions.iter() {
            let long_form = fn_info.build_long_fn_info(cx);
            writeln!(
                unsafe_file,
                "{}",
                serde_json::to_string(&long_form).unwrap()
            );
        }
        // summary
        let mut summary_file = file_ops.get_summary_functions_file(true);
        analysis::save_summary_analysis(
            results::functions::Summary::new(
                self.unsafe_functions.len(),
                self.unsafe_functions.len() + self.normal_functions.len(),
            ),
            &mut summary_file,
        );
        // external calls summary
        let mut external_calls_summary_file = file_ops.get_external_calls_summary_file(true);
        let summary = self.collect_external_unsafe_calls(cx);
        analysis::save_summary_analysis(summary, &mut external_calls_summary_file);
        // unsafe traits
        let mut traits_file = file_ops.get_unsafe_traits_file(true);
        let unsafe_traits = traits::run_analysis(cx);
        analysis::save_summary_analysis(unsafe_traits, &mut traits_file);
    }

    fn collect_external_unsafe_calls<'a, 'tcx>(&self, cx: &LateContext<'a, 'tcx>) -> results::functions::ExternalCallsSummary {

        let mut res = results::functions::ExternalCallsSummary::new();
        let mut map = HashMap::new();
        for fn_info in self.normal_functions.iter() {
            for (crate_num, ext_call, safety) in fn_info.external_calls().iter() {
                if let fn_info::Safety::Unsafe = safety {
                    //prepend crate name to avoid name collisions
                    let key = get_full_path(cx, crate_num, ext_call);
                    let count = map.entry(key).or_insert(0 as usize);
                    *count = *count + 1;
                }
            }
        }
        for fn_info in self.unsafe_functions.iter() {
            for (crate_num, ext_call, safety) in fn_info.external_calls().iter() {
                if let fn_info::Safety::Unsafe = safety {
                    let count = map.entry(get_full_path(cx, crate_num, ext_call)).or_insert(0 as usize);
                    *count = *count + 1;
                }
            }
        }
        for (call, count) in map.iter() {
            res.push(call.to_string(), *count);
        }
        res
    }
}

fn get_full_path<'a, 'tcx>( cx: &LateContext<'a, 'tcx>, crate_num: &hir::def_id::CrateNum, fn_call: &String ) -> String {
    let mut key = String::new();
    key.push_str(&cx.tcx.crate_name(*crate_num).to_string());
    key.push_str("::");
    key.push_str(fn_call);
    key.clone()
}

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
