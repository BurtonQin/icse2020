use std::collections::HashMap;

use syntax::ast::NodeId;
use rustc::hir;
use rustc::hir::def_id::DefId;
use rustc::mir::visit::Visitor;
use rustc::mir::{BasicBlock, Location, Operand, Terminator, TerminatorKind};
use rustc::ty::TyKind;

use results::implicit::UnsafeInBody;
use rustc::lint::LateContext;


pub fn run_sources_analysis<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, fns: &Vec<NodeId>)
                                      -> Vec<UnsafeInBody> {

    let mut with_unsafe = HashMap::new();
    let mut call_graph = HashMap::new();
    for &fn_id in fns {
        let fn_def_id = cx.tcx.hir.local_def_id(fn_id);
        let mir = &mut cx.tcx.optimized_mir(fn_def_id);
        let mut body_visitor = UnsafeBlocksVisitorData {
            hir: &cx.tcx.hir,
            has_unsafe: false,
        };
        let body_id = cx.tcx.hir.body_owned_by(fn_id);
        let body = cx.tcx.hir.body(body_id);
        hir::intravisit::walk_body(&mut body_visitor, body);

        let mut info = UnsafeInBody::new(::get_node_name(cx, fn_id));
        if body_visitor.has_unsafe {
            info.has_unsafe = true;
            with_unsafe.insert(fn_def_id, info);
        } else {
            // collect calls
            match cx.tcx.fn_sig(fn_def_id).unsafety() {
                hir::Unsafety::Unsafe => {} //ignore it
                hir::Unsafety::Normal => {
                    let mut calls_visitor = CallsVisitor::new(&cx);
                    calls_visitor.visit_mir(mir);
                    call_graph.insert(fn_def_id, calls_visitor.calls);
                }
            };
        }
    }
    // propagate crate local unsafety
    let mut changes = true;
    while changes {
        changes = false;
        for (def_id, calls) in call_graph.iter() {
            if let None = with_unsafe.get(def_id) {
                // get local calls
                let local_calls = calls.iter().filter(|call_id|{call_id.is_local()});
                if local_calls.into_iter()
                    .any(|call_id| {
                        if let None = with_unsafe.get(call_id) {
                            false
                        } else {
                            true
                        }
                    })
                    {
                        let info = UnsafeInBody::new(::get_node_name(cx,
                                                                     cx.tcx.hir.def_index_to_node_id(def_id.index)));
                        with_unsafe.insert(*def_id, info);
                        changes = true;
                    }
            }
        }
    }
    // propagate external unsafety
    //TODO
    Vec::new()
}


struct CallsVisitor<'a, 'tcx: 'a> {
    cx: &'a LateContext<'a, 'tcx>,
    calls: Vec<DefId>,
}

impl<'a, 'tcx> CallsVisitor<'a, 'tcx> {
    fn new(cx: &'a LateContext<'a, 'tcx>) -> Self {
        CallsVisitor { cx, calls: Vec::new() }
    }
}

impl<'a, 'tcx> Visitor<'tcx> for CallsVisitor<'a, 'tcx> {
    fn visit_terminator(
        &mut self,
        _block: BasicBlock,
        terminator: &Terminator<'tcx>,
        _location: Location,
    ) {
        if let TerminatorKind::Call {
            ref func,
            args: _,
            destination: _,
            cleanup: _,
        } = terminator.kind {
            if let Operand::Constant(constant) = func {
                if let TyKind::FnDef(callee_def_id, _substs) = constant.literal.ty.sty {
                    match self.cx.tcx.fn_sig(callee_def_id).unsafety() {
                        hir::Unsafety::Unsafe => {} // do nothing; there must be a surrounding unsafe block
                        hir::Unsafety::Normal => {self.calls.push(callee_def_id);}
                    };
                } else {
                    error!("TypeVariants NOT handled {:?}", constant.literal.ty.sty);
                }
            } else {
                error!("calls.rs::Operand Type NOT handled {:?} at {:?}"
                       , func, ::get_file_and_line(self.cx,terminator.source_info.span));
            }
        }
    }
}

struct UnsafeBlocksVisitorData<'tcx> {
    hir: &'tcx hir::map::Map<'tcx>,
    has_unsafe: bool,
}

impl<'a, 'tcx> hir::intravisit::Visitor<'tcx> for UnsafeBlocksVisitorData<'tcx> {
    fn visit_block(&mut self, b: &'tcx hir::Block) {
        match b.rules {
            hir::BlockCheckMode::DefaultBlock => {}
            hir::BlockCheckMode::UnsafeBlock(_unsafe_source) => {
                self.has_unsafe = true;
            }
            hir::BlockCheckMode::PushUnsafeBlock(unsafe_source) => {
                error!("hir::BlockCheckMode::PushUnsafeBlock {:?}", unsafe_source);
            }
            hir::BlockCheckMode::PopUnsafeBlock(unsafe_source) => {
                error!("hir::BlockCheckMode::PopUnsafeBlock {:?}", unsafe_source);
            }
        }
        hir::intravisit::walk_block(self, b);
    }

    fn nested_visit_map<'this>(&'this mut self) -> hir::intravisit::NestedVisitorMap<'this, 'tcx> {
        hir::intravisit::NestedVisitorMap::All(self.hir)
    }
}