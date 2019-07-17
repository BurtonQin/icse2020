use rustc::hir;
use syntax::ast::NodeId;
use rustc::lint::LateContext;
use rustc::mir::SourceInfo;
use rustc::mir::visit::Visitor;
use rustc::hir::HirId;

use std::fmt::Write;

use unsafety_sources::{UnsafetySourcesVisitor,UnsafetySourceCollector};
use results::functions::UnsafeFnUsafetySources;
use results::unsafety_sources::SourceKind;
use results::unsafety_sources::Source;
use results::functions::Argument;
use results::functions::ArgumentKind;
use results::functions::ShortFnInfo;
use std::ops::Deref;

pub fn run_sources_analysis<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, fns: &Vec<HirId>, user_defined_only: bool)
        -> (Vec<UnsafeFnUsafetySources>,Vec<ShortFnInfo>) {

    let mut sources = Vec::new();
    let mut no_reason = Vec::new();

    for &fn_id in fns {
        let node_id = cx.tcx.hir().hir_to_node_id(fn_id);
        let fn_def_id = cx.tcx.hir().local_def_id(node_id);
        let mut data = process_fn_decl(cx, node_id);
        let mir = &mut cx.tcx.optimized_mir(fn_def_id);
        let mut success = false;
        if let Some(mut body_visitor) = UnsafetySourcesVisitor::new(cx, mir, &mut data, fn_def_id)  {
            body_visitor.visit_body(mir);
            success = true;
        }
        if success {
            if data.arguments().is_empty() && data.sources().is_empty() && !data.from_trait() {
                let node_id = cx.tcx.hir().hir_to_node_id(fn_id);
                no_reason.push(build_short_fn_info(cx,node_id));
            }
            sources.push(data);
        }
    }
    (sources,no_reason)
}

fn build_short_fn_info<'a, 'tcx>( cx: &LateContext<'a, 'tcx>, decl_id: NodeId) -> results::functions::ShortFnInfo {
    let def_id = cx.tcx.hir().local_def_id(decl_id);
    let name = cx.tcx.def_path_str(def_id);
    let node_id = decl_id.to_string();
    let hir_id = cx.tcx.hir().node_to_hir_id(decl_id);
    let span = cx.tcx.hir().span(hir_id);
    let location = ::get_file_and_line(cx, span);
    results::functions::ShortFnInfo::new(name, node_id, location)
}


fn process_fn_decl<'a, 'tcx>( cx: &LateContext<'a, 'tcx>, decl_id: NodeId) -> UnsafeFnUsafetySources {
    let from_trait = is_unsafe_method(decl_id, cx);
    let def_id = cx.tcx.hir().local_def_id(decl_id);
    let name = cx.tcx.def_path_str(def_id);
    let mut res = UnsafeFnUsafetySources::new(name, from_trait);
    let hir_id = cx.tcx.hir().node_to_hir_id(decl_id);
    if let Some(fn_decl) = cx.tcx.hir().fn_decl_by_hir_id(hir_id) {
        let rustc::hir::FnDecl { inputs, output, c_variadic: _ , implicit_self: _} = fn_decl.deref();
        for ref input in inputs {
            if let Some(reason) = process_type(cx, &input) {
                //TODO record some information about the argument
                res.add_argument(reason);
            }
        }
    } else {
        error!("Decl NOT found {:?}", decl_id);
    }
    res
}

fn is_unsafe_method<'a, 'tcx>(node_id: NodeId, cx: &LateContext<'a, 'tcx>) -> bool {
    let hir_id = cx.tcx.hir().node_to_hir_id(node_id);
    let node = cx.tcx.hir().get(hir_id);
    match node {
        hir::Node::ImplItem(ref impl_item) => {
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
        _ => false,
    }
}

// returns true is a raw ptr is somewhere in the type
fn process_type<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, ty: &hir::Ty) -> Option<Argument> {
    match ty.node {
        hir::TyKind::Slice(ref sty) | hir::TyKind::Array(ref sty, _) => process_type(cx, &sty),

        hir::TyKind::Ptr(_) => {
            let mut buff = String::new();
            write!(buff,"{:?}", ty);
            Some(Argument::new(
                buff,
                ArgumentKind::RawPointer,
            ))
        },

        hir::TyKind::Rptr(_, _) => None, //TODO check:I think this is a Rust reference

        hir::TyKind::BareFn(ref bare_fn) => {
            if let hir::Unsafety::Unsafe = bare_fn.unsafety {
                let mut type_info = String::new();
                write!(type_info, "{:?}", ty);
                Some(Argument::new(
                    type_info,
                    ArgumentKind::UnsafeFunction,
                ))
            } else {
                process_ty_array(cx, &bare_fn.decl.inputs)
            }
        }

        hir::TyKind::Tup(ref vty) => process_ty_array(cx, &vty),

        hir::TyKind::Path(ref qpath) => match qpath {
            hir::QPath::Resolved(oty, _) => {
                if let Some(sty) = oty {
                    process_type(cx, sty)
                } else {
                    None
                }
            }
            hir::QPath::TypeRelative(pty, _) => process_type(cx, pty),
        },

        hir::TyKind::TraitObject(ref _poly_ref, _) => None, //TODO

        _ => None,
    }
}

fn process_ty_array<'a, 'tcx>(
    cx: &LateContext<'a, 'tcx>,
    array: &hir::HirVec<hir::Ty>,
) -> Option<Argument> {
    let mut iter = array.iter();
    let mut done = false;
    let mut res = None;
    while !done {
        if let Some(elt) = iter.next() {
            let arg_res = process_type(cx, elt);
            if let Some(_) = arg_res {
                res = arg_res;
                done = true;
            }
        } else {
            done = true;
        }
    }
    res
}

impl UnsafetySourceCollector for UnsafeFnUsafetySources {
    fn add_unsafety_source<'a, 'tcx>(
        &mut self,
        cx: &LateContext<'a, 'tcx>,
        kind: SourceKind,
        source_info: SourceInfo,
        _node_id: NodeId,
        user_provided: bool,
    ) {
        let source = Source {
            kind,
            loc: ::get_file_and_line(cx, source_info.span),
            user_provided,
        };
        self.add_source(source);
    }
}

