use rustc::hir;
use syntax::ast::NodeId;
use rustc::lint::LateContext;
use rustc::mir::SourceInfo;
use rustc::mir::visit::Visitor;

use std::fmt::Write;

use unsafety_sources::{UnsafetySourcesVisitor,UnsafetySourceCollector};
use results::functions::UnsafeFnUsafetySources;
use results::unsafety_sources::SourceKind;
use results::unsafety_sources::Source;
use results::functions::Argument;
use results::functions::ArgumentKind;
use results::functions::ShortFnInfo;

use restricted_unsafe::RestrictedUnsafeVisitor;

pub fn run_sources_analysis<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, fns: &Vec<NodeId>)
        -> (Vec<UnsafeFnUsafetySources>,Vec<ShortFnInfo>) {

    let mut sources = Vec::new();
    let mut no_reason = Vec::new();

    for &fn_id in fns {
        let fn_def_id = cx.tcx.hir.local_def_id(fn_id);
        let mut data = process_fn_decl(cx, fn_id);
        let mir = &mut cx.tcx.optimized_mir(fn_def_id);
        let mut success = false;
        if let Some(mut body_visitor) = UnsafetySourcesVisitor::new(cx, mir, &mut data, fn_def_id)  {
            body_visitor.visit_mir(mir);
            success = true;
        }
        if success {
            if data.arguments().is_empty() && data.sources().is_empty() && !data.from_trait() {
                no_reason.push(build_short_fn_info(cx,fn_id));
            }
            sources.push(data);
        }
    }
    (sources,no_reason)
}

fn build_short_fn_info<'a, 'tcx>( cx: &LateContext<'a, 'tcx>, decl_id: NodeId) -> results::functions::ShortFnInfo {
    let name = cx.tcx.node_path_str(decl_id);
    let node_id = decl_id.to_string();
    let span = cx.tcx.hir.span(decl_id);
    let location = ::get_file_and_line(cx, span);
    results::functions::ShortFnInfo::new(name, node_id, location)
}


fn process_fn_decl<'a, 'tcx>( cx: &LateContext<'a, 'tcx>, decl_id: NodeId) -> UnsafeFnUsafetySources {
    let from_trait = is_unsafe_method(decl_id, cx);
    let mut res = UnsafeFnUsafetySources::new(cx.tcx.node_path_str(decl_id), from_trait);
    if let Some(fn_decl) = cx.tcx.hir.fn_decl(decl_id) {
        for input in fn_decl.inputs {
            if let Some(reason) = process_type(cx, &input) {
                res.add_argument(reason);
            }
        }
    } else {
        error!("Decl NOT found {:?}", decl_id);
    }
    res
}

fn is_unsafe_method<'a, 'tcx>(node_id: NodeId, cx: &LateContext<'a, 'tcx>) -> bool {
    let node = cx.tcx.hir.get(node_id);
    match node {
        hir::Node::ImplItem(ref impl_item) => {
            if let ::hir::ImplItemKind::Method(ref method_sig, ..) = impl_item.node {
                if let hir::Unsafety::Normal = method_sig.header.unsafety {
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

// returns true is a raw ptr is somewhere in the type
fn process_type<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, ty: &hir::Ty) -> Option<Argument> {
    match ty.node {
        hir::TyKind::Slice(ref sty) | hir::TyKind::Array(ref sty, _) => process_type(cx, &sty),

        hir::TyKind::Ptr(_) => {
            let mut buff = String::new();
            Some(Argument::new(
                buff,
                ArgumentKind::RawPointer,
            ))
        },

        hir::TyKind::Rptr(_, _) => None,

        hir::TyKind::BareFn(ref bare_fn) => {
            if let hir::Unsafety::Unsafe = bare_fn.unsafety {
                let mut type_info = String::new();
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

        hir::TyKind::TraitObject(ref _poly_ref, _) => None,

        hir::TyKind::Never | hir::TyKind::Typeof(_) | hir::TyKind::Infer | hir::TyKind::Err => None,
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

////////////////////// New Analysis for Camera Ready
// A function is unsafe but has no user defined unsafe operations
// nor an argument of type pointer
pub fn run_restricted_unsafe_analysis<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, fns: &Vec<NodeId>)
        -> (Vec<ShortFnInfo>) {
    let mut sources = Vec::new();
    for &fn_id in fns {
        let fn_def_id = cx.tcx.hir.local_def_id(fn_id);
        let mir = &mut cx.tcx.optimized_mir(fn_def_id);
        if let Some(mut body_visitor) = RestrictedUnsafeVisitor::new(cx, mir, fn_def_id)  {
            body_visitor.visit_mir(mir);
            if body_visitor.has_unsafe() {
                sources.push(build_short_fn_info(cx,fn_id));
            }
        }

    }
    (sources)
}
