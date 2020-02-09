#![crate_name = "unsafe_analysis"]
#![crate_type = "dylib"]
#![feature(plugin_registrar)]
#![feature(rustc_private)]
#![feature(box_syntax)]
#![feature(macro_at_most_once_rep)]
#![feature(extern_prelude)]
#![feature(box_patterns)]
#![feature(drain_filter)]

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
extern crate fxhash;

extern crate serde;
//extern crate serde_derive;
extern crate serde_json;

use rustc::hir;
use rustc::hir::Crate;
use rustc::hir::def_id::DefId;
use rustc::lint::{LateContext, LateLintPass, LateLintPassObject, LintArray, LintPass};
use rustc_plugin::Registry;
use syntax::ast::NodeId;
use syntax_pos::Span;
use std::io::Write;
use std::fs::File;
use std::fmt::Write as FmtWrite;
use std::env;

mod blocks;
mod traits;
mod unsafety_sources;
mod functions;
mod calls;
mod implicit_unsafe;
mod restricted_unsafe;
mod deps;

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

    fn check_crate(&mut self, _: &LateContext<'a, 'tcx>, _: &'tcx Crate) {
    }


    fn check_crate_post(&mut self, cx: &LateContext<'a, 'tcx>, _: &'tcx Crate) {
        let root_dir = get_root_dir();
        let cnv = local_crate_name_and_version();
        let file_ops = results::FileOps::new(&cnv.0, &cnv.1, &root_dir);
        // blocks summary
        let bb_summary: results::blocks::BlockSummary = blocks::run_summary_analysis(cx);
        let mut file = file_ops.create_file (results::BLOCK_SUMMARY_BB);
        save_summary_analysis(bb_summary, &mut file);
        // unsafe functions summary
        let mut fn_summary_file = file_ops.create_file (results::SUMMARY_FUNCTIONS_FILE_NAME);
        save_summary_analysis(
            results::functions::Summary::new(
                self.unsafe_functions.len(),
                self.unsafe_functions.len() + self.normal_functions.len(),
            ),
            &mut fn_summary_file,
        );
        // unsafe functions restricted
        let mut file = file_ops.create_file (results::UNSAFE_CALLS_RESTRICTED);
        let actualy_unsafe = functions::run_restricted_unsafe_analysis(cx,&self.unsafe_functions);
        save_summary_analysis(
            results::functions::Summary::new(actualy_unsafe.len(),
                                             self.unsafe_functions.len() + self.normal_functions.len()),&mut file);
        //unsafety in functions
        let (fn_unsafety,no_reason) = functions::run_sources_analysis(cx,&self.unsafe_functions);
        let mut file = file_ops.create_file (results::FN_UNSAFETY_SOURCES_FILE_NAME);
        save_analysis(fn_unsafety,&mut file);
        // unsafe traits
        let mut impls_file = file_ops.create_file (results::UNSAFE_TRAITS_IMPLS);
        let mut traits_file = file_ops.create_file (results::UNSAFE_TRAITS);
        let result = traits::run_analysis(cx);
        save_analysis(result.unsafe_traits_impls, &mut impls_file);
        save_analysis(result.unsafe_traits, &mut traits_file);
        //unsafety sources in blocks
        let mut bus_file = file_ops.create_file (results::BLOCK_UNSAFETY_SOURCES_FILE_NAME);
        let bus_res = blocks::run_unsafety_sources_analysis(cx,&self.normal_functions);
        save_analysis(bus_res, &mut bus_file);
        //unsafe function calls
        let unsafe_calls = calls::run_analysis(cx);
        let mut file = file_ops.create_file (results::UNSAFE_CALLS);
        save_analysis(unsafe_calls, &mut file);


        let mut all_fn_ids = Vec::new();
        for fn_id in self.normal_functions.iter() {
            all_fn_ids.push(*fn_id)
        }
        for fn_id in self.unsafe_functions.iter() {
            all_fn_ids.push(*fn_id)
        }
        let mut file = file_ops.create_file (results::IMPLICIT_RTA_OPTIMISTIC_FILENAME);
        let opt_rta_impl_unsafe = implicit_unsafe::run_sources_analysis(cx,&all_fn_ids,
                                                                             true, false);
        save_analysis(opt_rta_impl_unsafe, &mut file);
        file.flush();

        let mut file = file_ops.create_file (results::IMPLICIT_RTA_PESSIMISTIC_FILENAME);
        let pes_rta_impl_unsafe = implicit_unsafe::run_sources_analysis(cx,
                                                                                &self.normal_functions,
                                                                                false, false);
        save_analysis(pes_rta_impl_unsafe, &mut file);
        file.flush();
        // restricted unsafe
        let mut file = file_ops.create_file (results::RESTRICTED_RTA_OPTIMISTIC_FILENAME);
        let results = implicit_unsafe::run_sources_analysis(cx,&all_fn_ids,
                                                                             true, true);
        save_analysis(results, &mut file);
        drop(file);
        let mut file = file_ops.create_file (results::RESTRICTED_RTA_PESSIMISTIC_FILENAME);
        let results = implicit_unsafe::run_sources_analysis(cx,
                   &self.normal_functions, false, true);
        save_analysis(results, &mut file);
        drop(file);
    }

    fn check_body(&mut self, cx: &LateContext<'a, 'tcx>, body: &'tcx hir::Body) {
        //need to find fn/method declaration of this body
        let owner_def_id = cx.tcx.hir.body_owner_def_id(body.id());
        if let Some(owner_node_id) = cx.tcx.hir.as_local_node_id(owner_def_id) {
            let node = cx.tcx.hir.get(owner_node_id);
            match node {
                hir::Node::Item(item) => {
                    // functions
                    if let hir::ItemKind::Fn(ref _fn_decl, ref fn_header, _, _) = item.node {
                        self.add(owner_node_id, fn_header.unsafety);
                    }
                },
                hir::Node::ImplItem(ref impl_item) => {
                    // method implementations
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
                    // associated methods (functions in impl blocks, not of traits)
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
    file.sync_all();
}

pub fn save_analysis<T>(analysis_results: Vec<T>, file: &mut File)
    where
        T: serde::ser::Serialize,
{
    for res in analysis_results {
        let serialized = serde_json::to_string(&res).unwrap();
        writeln!(file, "{}", serialized);
    }
    file.flush();
    file.sync_all();
}

pub fn local_crate_name_and_version() -> (String, String) {
    let pkg = env::var("CARGO_PKG_NAME").unwrap();
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    (pkg,version)
}

pub fn local_crate_name() -> String {
    let pkg = env::var("CARGO_PKG_NAME").unwrap();
    pkg
}

fn get_node_name<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, def_id: DefId) -> String {
    cx.tcx.absolute_item_path_str(def_id)
}

pub fn get_file_and_line<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, span: Span) -> String {
    let mut result = String::new();
    let loc = cx.tcx.sess.source_map().lookup_char_pos(span.lo());
    let filename = &loc.file.name;
    write!(result, "file: {:?} line {:?}", filename, loc.line);
    result
}

pub fn get_root_dir() -> String {
     match std::env::var("FULL_ANALYSIS_DIR") {
        Ok (val) => {val.to_string()}
        Err (_) => {"/home/ans5k/unsafe_analysis/analysis-data/full-analysis".to_string()}
    }
}

fn convert_abi(abi: rustc_target::spec::abi::Abi) -> results::Abi {
    match abi {
        rustc_target::spec::abi::Abi::Cdecl => results::Abi::Cdecl,
        rustc_target::spec::abi::Abi::Stdcall => results::Abi::Stdcall,
        rustc_target::spec::abi::Abi::Fastcall => results::Abi::Fastcall,
        rustc_target::spec::abi::Abi::Vectorcall => results::Abi::Vectorcall,
        rustc_target::spec::abi::Abi::Thiscall => results::Abi::Thiscall,
        rustc_target::spec::abi::Abi::SysV64 => results::Abi::SysV64,
        rustc_target::spec::abi::Abi::PtxKernel => results::Abi::PtxKernel,
        rustc_target::spec::abi::Abi::Msp430Interrupt => {
            results::Abi::Msp430Interrupt
        }
        rustc_target::spec::abi::Abi::X86Interrupt => results::Abi::X86Interrupt,
        rustc_target::spec::abi::Abi::AmdGpuKernel => results::Abi::AmdGpuKernel,
        rustc_target::spec::abi::Abi::Rust => results::Abi::Rust,
        rustc_target::spec::abi::Abi::C => results::Abi::C,
        rustc_target::spec::abi::Abi::System => results::Abi::System,
        rustc_target::spec::abi::Abi::RustIntrinsic => {
            results::Abi::RustIntrinsic
        }
        rustc_target::spec::abi::Abi::RustCall => results::Abi::RustCall,
        rustc_target::spec::abi::Abi::PlatformIntrinsic => {
            results::Abi::PlatformIntrinsic
        }
        rustc_target::spec::abi::Abi::Unadjusted => results::Abi::Unadjusted,
        rustc_target::spec::abi::Abi::Aapcs => results::Abi::Aapcs,
        rustc_target::spec::abi::Abi::Win64 => results::Abi::Win64,
    }
}

fn get_fn_path<'a, 'tcx>(cx: &'a LateContext<'a, 'tcx>, def_id:DefId) -> String {
    let mut out = String::new();
    write!(&mut out,"{:?}", cx.tcx.def_path(def_id).data);
    out
}
