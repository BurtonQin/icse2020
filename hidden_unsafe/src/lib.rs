#![crate_name = "hidden_unsafe"]
#![crate_type = "dylib"]
#![feature(plugin_registrar)]
#![feature(rustc_private)]
#![feature(box_syntax)]
#![feature(macro_at_most_once_rep)]
#![feature(extern_prelude)]
#![feature(box_patterns)]

#[macro_use]
extern crate rustc;
extern crate rustc_mir;
extern crate rustc_plugin;
extern crate rustc_target;
extern crate rustc_data_structures;
extern crate syntax;
extern crate syntax_pos;
extern crate chrono;
extern crate cargo_metadata;
extern crate cargo;
extern crate results;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;


use fn_info::FnInfo;
use rustc::hir;
use rustc::hir::Crate;
use rustc::lint::{LateContext, LateLintPass, LateLintPassObject, LintArray, LintPass};
use rustc_plugin::Registry;
use syntax::ast::NodeId;

mod analysis;
mod calls;
mod fn_info;
mod unsafety_sources;
mod implicit_analysis;
mod unsafe_traits;
mod unsafety;
mod util;
mod deps;


use unsafety_sources::UnsafeFnUsafetyAnalysis;
use unsafety_sources::UnsafeBlockUnsafetyAnalysis;
use results::implicit::UnsafeInBody;
use results::implicit::UnsafeTraitSafeMethodInBody;

use std::io::Write;
use implicit_analysis::propagate_external;


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
        let mut safe_file = results::functions::get_safe_functions_file(cnv.0, cnv.1).open_file();
        for ref fn_info in self.normal_functions.iter() {
            let long_form = fn_info.build_long_fn_info(cx);
            writeln!(safe_file, "{}", serde_json::to_string(long_form).unwrap());
        }
        // unsafe functions
        let mut unsafe_file = results::functions::get_unsafe_functions_file(cnv.0, cnv.1).open_file();
        for ref fn_info in self.unsafe_functions.iter() {
            let long_form = fn_info.build_long_fn_info(cx);
            writeln!(safe_file, "{}", serde_json::to_string(long_form).unwrap());
        }
        // summary
        let mut summary_file = results::functions::get_unsafe_functions_file(cnv.0, cnv.1).open_file();
        writeln!(summary_file, "{}",
                 serde_json::to_string( results::functions::Summary{
                     unsafe_no : self.unsafe_functions.len(),
                     total: self.unsafe_functions.len() + self.safe_functions.len(),
                 })
        );
    }
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
        // the call graph info is available here
        self.save_results(cx); // saves normal and unsafe functions info, and summary for RQ2

        // implicit unsafe analysis
        let implicit_external = deps::load_all_analyses(cx
                                                      , &external_crates
                                                      , &mut self.normal_functions);
        let mut res1: Vec<(&FnInfo, UnsafeInBody)> = analysis::run_all(cx, &self.normal_functions, true);
        propagate_external( cx, &mut res1, &hidden_external);
        UnsafeInBody::save_analysis(res1);

        // unsafe traits analysis
        let res2: Vec<(&FnInfo, UnsafeTraitSafeMethodInBody)> = analysis::run_all(cx, &self.normal_functions, true);
        UnsafeTraitSafeMethodInBody::save_analysis(res2);

        let unsafe_fn_info: Vec<(&FnInfo, UnsafeFnUsafetyAnalysis)> =
            analysis::run_all(cx, &self.unsafe_functions, false);
        ImplicitUnsafe::print_results(cx, &unsafe_fn_info, "30_unsafe_fn");

        let safe_fn_info: Vec<(&FnInfo, UnsafeBlockUnsafetyAnalysis)> =
            analysis::run_all(cx, &self.normal_functions, false);
        ImplicitUnsafe::print_results(cx, &safe_fn_info, "40_unsafe_blocks");

//        self.print_external_calls(cx);
    }

    fn check_body(&mut self, cx: &LateContext<'a, 'tcx>, body: &'tcx hir::Body) {
        //need to find fn/method declaration of this body
        let owner_def_id = cx.tcx.hir.body_owner_def_id(body.id());
        if let Some(owner_node_id) = cx.tcx.hir.as_local_node_id(owner_def_id) {
            if util::is_fn_or_method(owner_node_id,cx) {
                if util::is_unsafe_fn(owner_node_id, cx) {
                    self.push_unsafe_fn_info(owner_node_id);
                } else {
                    if util::is_unsafe_method(owner_node_id,cx) {
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
