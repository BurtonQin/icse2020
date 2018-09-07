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
mod util;
mod deps;


use results::functions::UnsafeFnUsafetySources;
use results::blocks::BlockUnsafetyAnalysisSources;
use results::implicit::UnsafeInBody;
use results::implicit::UnsafeTraitSafeMethodInBody;
use implicit_analysis::propagate_external;

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
        let mut safe_file = results::functions::get_safe_functions_file(cnv.0.clone()
                                                                        , cnv.1.clone()).open_file(true);
        for ref fn_info in self.normal_functions.iter() {
            let long_form = fn_info.build_long_fn_info(cx);
            writeln!(safe_file, "{}", serde_json::to_string(&long_form).unwrap());
        }
        // unsafe functions
        let mut unsafe_file = results::functions::get_unsafe_functions_file(cnv.0.clone()
                                                                            , cnv.1.clone()).open_file(true);
        for ref fn_info in self.unsafe_functions.iter() {
            let long_form = fn_info.build_long_fn_info(cx);
            writeln!(unsafe_file, "{}", serde_json::to_string(&long_form).unwrap());
        }
        // summary
        let mut summary_file = results::functions::get_unsafe_functions_file(cnv.0.clone()
                                                                             , cnv.1.clone()).open_file(true);
        writeln!(summary_file, "{}",
                 serde_json::to_string( &results::functions::Summary::new(
                      self.unsafe_functions.len(),
                      self.unsafe_functions.len() + self.normal_functions.len(),
                 )).unwrap()
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

        let cnv = util::local_crate_name_and_version();

        // implicit unsafe analysis
        let implicit_external = deps::load_all_analyses(cx
                                                      , &external_crates
                                                      , &mut self.normal_functions);
// TODO fix save_results
        let mut res1: Vec<(&FnInfo, UnsafeInBody)> = analysis::run_all(cx, &self.normal_functions, true);
        propagate_external( cx, &mut res1, &implicit_external);
        analysis::save_analysis::<UnsafeInBody>(res1 as Vec<(&FnInfo, UnsafeInBody)>,
                                          &mut results::implicit::get_implicit_unsafe_file(cnv.0.clone(), cnv.1.clone()).open_file(true));

        // unsafe traits analysis
        let res2: Vec<(&FnInfo, UnsafeTraitSafeMethodInBody)> = analysis::run_all(cx, &self.normal_functions, true);
        analysis::save_analysis(res2,
                                          &mut results::implicit::get_implicit_trait_unsafe_file(cnv.0.clone(), cnv.1.clone()).open_file(true)
        );

        let unsafe_fn_info: Vec<(&FnInfo, UnsafeFnUsafetySources)> =
            analysis::run_all(cx, &self.unsafe_functions, false);
        analysis::save_analysis(unsafe_fn_info,
            &mut results::functions::get_fn_unsafety_sources_file(cnv.0.clone(), cnv.1.clone()).open_file(true)
        );

        let safe_fn_info: Vec<(&FnInfo, BlockUnsafetyAnalysisSources)> =
            analysis::run_all(cx, &self.normal_functions, false);
        analysis::save_analysis(safe_fn_info,
                &mut results::blocks::get_blocks_unsafety_sources_file(cnv.0.clone(), cnv.1.clone()).open_file(true)
        );

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
