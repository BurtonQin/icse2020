#![crate_name = "hidden_unsafe"]
#![crate_type = "dylib"]
#![feature(plugin_registrar)]
#![feature(rustc_private)]
#![feature(box_syntax)]
#![feature(macro_at_most_once_rep)]
#![feature(macro_vis_matcher)]
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

use fn_info::FnInfo;
use print::{EmptyPrinter, Print};
use rustc::hir;
use rustc::hir::Crate;
use rustc::lint::{LateContext, LateLintPass, LateLintPassObject, LintArray, LintPass};
use rustc_plugin::Registry;
use syntax::ast::NodeId;

mod analysis;
mod calls;
mod fn_info;
mod unsafety_sources;
mod print;
mod unsafe_blocks;
mod unsafe_traits;
mod unsafety;
mod util;


use unsafety_sources::UnsafeFnUsafetyAnalysis;
use unsafety_sources::UnsafeBlockUnsafetyAnalysis;
use unsafe_blocks::UnsafeInBody;
use unsafe_traits::UnsafeTraitSafeMethod;

use std::io::Write;
use std::path::PathBuf;
use std::fs::DirBuilder;
use std::fs::OpenOptions;

struct HiddenUnsafe {
    normal_functions: Vec<FnInfo>,
    unsafe_functions: Vec<FnInfo>,
}

impl HiddenUnsafe {
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

    pub fn print_results<'a, 'tcx, T: Print>(
        cx: &'a LateContext<'a, 'tcx>,
        result: &'a Vec<(&'a FnInfo, T)>,
        name: &'static str,
    ) {
        let mut file = util::open_file(name);
        for &(fn_info, ref res) in result.iter() {
            fn_info.print(cx, res, &mut file);
        }
    }

    pub fn print_graph<'a, 'tcx>(&self, cx: &'a LateContext<'a, 'tcx>) {
        let empty_printer = EmptyPrinter {};
        let mut safe_file = util::open_file("00_safe_functions");
        for ref fn_info in self.normal_functions.iter() {
            writeln!(safe_file, "+++++++++++++++++++++++++++++++++++++++++++++++++++");
            fn_info.print(cx, &empty_printer, &mut safe_file);
            fn_info.print_local_calls(cx, &mut safe_file);
            writeln!(safe_file, "--------------------------------------------------");
            fn_info.print_external_calls(cx, &mut safe_file);
        }
        let mut unsafe_file = util::open_file("01_unsafe_functions");
        for ref fn_info in self.unsafe_functions.iter() {
            fn_info.print(cx, &empty_printer, &mut unsafe_file);
        }
    }

    pub fn print_external_calls<'a, 'tcx>(&self, cx: &'a LateContext<'a, 'tcx>) {
        // delete the old data
        let local_crate = util::crate_name();
        let dir_path: PathBuf = [print::ROOT_DIR.to_string()
                , local_crate.to_string()
                , "external_calls".to_string()].iter().collect();
        DirBuilder::new().recursive(true).create(&dir_path).unwrap();
        std::fs::remove_dir_all(&dir_path).unwrap();
        DirBuilder::new().recursive(true).create(&dir_path).unwrap();
        // create a file for each crate

        // collect crates and external calls
        let mut external_crates = Vec::new();
        let mut external_calls = Vec::new();

        for ref fn_info in self.normal_functions.iter() {
            fn_info.external_calls().iter().for_each(|elt| {
                if !external_crates.iter().any(|crate_num| *crate_num == elt.0) {
                    external_crates.push(elt.0);
                }
                if !external_calls.iter().any(|x: &(hir::def_id::CrateNum, String)| x.0 == elt.0 && x.1 == elt.1) {
                    external_calls.push(elt.clone());
                }
            });
        }

        external_crates.iter().for_each(|krate| {
            let file_path: PathBuf = [print::ROOT_DIR.to_string()
                , local_crate.to_string()
                , "external_calls".to_string()
                , cx.tcx.crate_name(*krate).to_string() ].iter().collect();
            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true).open(file_path).unwrap();
            external_calls
                .iter()
                .filter(|elt| elt.0 == *krate)
                .for_each(|elt| {
                    writeln!(file, "{:?}", elt.1);
                });
        });
    }
}

declare_lint!(pub HIDDEN_UNSAFE, Allow, "Functions using hidden unsafe");

impl<'a, 'tcx> LintPass for HiddenUnsafe {
    fn get_lints(&self) -> LintArray {
        lint_array!(HIDDEN_UNSAFE)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for HiddenUnsafe {
    fn check_crate_post(&mut self, cx: &LateContext<'a, 'tcx>, _: &'tcx Crate) {
        calls::build_call_graph(&mut self.normal_functions, cx);
        self.print_graph(cx);
        // the information collected in check_body is available at this point
        // collect unsafe blocks information for each function
        // and propagates it

        let res1: Vec<(&FnInfo, UnsafeInBody)> = analysis::run_all(cx, &self.normal_functions, true);
        HiddenUnsafe::print_results(cx, &res1, "10_unsafe_in_call_tree");

        let res2: Vec<(&FnInfo, UnsafeTraitSafeMethod)> = analysis::run_all(cx, &self.normal_functions, true);
        HiddenUnsafe::print_results(cx, &res2, "20_unsafe_trait_safe_method_in_call_tree");

        let unsafe_fn_info: Vec<(&FnInfo, UnsafeFnUsafetyAnalysis)> =
            analysis::run_all(cx, &self.unsafe_functions, false);
        HiddenUnsafe::print_results(cx, &unsafe_fn_info, "30_unsafe_fn");

        let safe_fn_info: Vec<(&FnInfo, UnsafeBlockUnsafetyAnalysis)> =
            analysis::run_all(cx, &self.normal_functions, false);
        HiddenUnsafe::print_results(cx, &safe_fn_info, "40_unsafe_blocks");

        self.print_external_calls(cx);
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
    reg.register_late_lint_pass(box HiddenUnsafe::new() as LateLintPassObject);
}
