use std::collections::HashMap;
use std::fmt::Write;

use syntax::ast::NodeId;
use rustc::hir;
use rustc::hir::def_id::DefId;
use rustc::mir::visit::Visitor;
use rustc::mir::{BasicBlock, Location, Operand, Terminator, TerminatorKind, Mir};
use rustc::ty::TyKind;
use rustc::ty;

use results::implicit::UnsafeInBody;
use rustc::lint::LateContext;
use implicit_unsafe::deps;
use get_fn_path;
use implicit_unsafe::UnsafeBlocksVisitorData;

enum Call {
    Static(DefId),
    Dynamic
}

pub fn run_sources_analysis<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, fns: &Vec<NodeId>, optimistic: bool)
                                      -> Vec<UnsafeInBody> {

    let mut with_unsafe = HashMap::new();
    let mut call_graph = HashMap::new();
    for &fn_id in fns {
        let fn_def_id = cx.tcx.hir.local_def_id(fn_id);
        match cx.tcx.fn_sig(fn_def_id).unsafety() {
            hir::Unsafety::Unsafe => {} //ignore it
            hir::Unsafety::Normal => {
                let mut body_visitor = UnsafeBlocksVisitorData {
                    hir: &cx.tcx.hir,
                    has_unsafe: false,
                };
                let body_id = cx.tcx.hir.body_owned_by(fn_id);
                let body = cx.tcx.hir.body(body_id);
                hir::intravisit::walk_body(&mut body_visitor, body);
                if body_visitor.has_unsafe {
                    let mut info = UnsafeInBody::new(get_fn_path(cx,fn_def_id), true, ::get_node_name(cx,fn_id));
                    with_unsafe.insert(fn_def_id, info);
                } else {
                    // collect calls
                    let mir = &mut cx.tcx.optimized_mir(fn_def_id);
                    let mut calls_visitor = CallsVisitor::new(&cx,mir,fn_def_id);
                    calls_visitor.visit_mir(mir);
                    if !optimistic && (calls_visitor.uses_fn_ptr || calls_visitor.uses_unresolved_calls) {
                        let mut info = UnsafeInBody::new(get_fn_path(cx,fn_def_id), true, ::get_node_name(cx,fn_id));
                        with_unsafe.insert(fn_def_id, info);
                    } else {
                        call_graph.insert(fn_def_id, calls_visitor.calls);
                    }
                }
            }
        }
    }
    // propagate external unsafety
    let mut external_calls : HashMap<String,DefId> = HashMap::new();
    for (def_id, calls) in call_graph.iter() {
        if let None = with_unsafe.get(def_id) {
            for call in calls.iter() {
                if let Call::Static(calee_def_id) = call
                {
                    if !calee_def_id.is_local() {
                        info!("external call def id: {:?}", calee_def_id);

                        external_calls.insert(get_fn_path(cx, *calee_def_id), *calee_def_id);
                    }
                }
            }
        }
    }
    let implicit_external =
        deps::load(cx, &external_calls, optimistic);
    for (def_id, calls) in call_graph.iter() {
        if let None = with_unsafe.get(def_id) {
            for call in calls.iter() {

                if let Call::Static(calee_def_id) = call {
                    if !calee_def_id.is_local() {
                        if let Some(ub) = implicit_external.get(calee_def_id) {
                            if ub.has_unsafe {
                                let fn_name =
                                    if let Some(fn_id) = cx.tcx.hir.as_local_node_id(*def_id) {
                                        ::get_node_name(cx, fn_id)
                                    } else {
                                        "ERROR!".to_string()
                                    };
                                with_unsafe.insert(*def_id,
                                                   UnsafeInBody::new(
                                                       get_fn_path(cx, *def_id),
                                                       true,
                                                       fn_name
                                                   ));
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    // propagate crate local unsafety
    let mut changes = true;
    while changes {
        changes = false;
        for (def_id, calls) in call_graph.iter() {
            let fn_name =
                if let Some(fn_id) = cx.tcx.hir.as_local_node_id(*def_id) {
                    ::get_node_name(cx, fn_id)
                } else {
                    "ERROR!".to_string()
                };
            if let None = with_unsafe.get(def_id) {
                // Dynamic Calls
                if calls.iter().any(|call|{
                    if let Call::Dynamic = *call {
                        true
                    } else {
                        false
                    }
                }) {
                    let info = UnsafeInBody::new(get_fn_path(cx, *def_id), true, fn_name);
                    with_unsafe.insert(*def_id, info);
                    changes = true;
                } else {
                    // No Dynamic Calls
                    // get local calls
                    let local_calls = calls.iter().filter(
                        |call| {
                            if let Call::Static(call_id) = *call {
                                call_id.is_local()
                            } else {
                                false
                            }
                        });
                    if local_calls.into_iter()
                        .any(|call| {
                            if let Call::Static(call_id) = *call {
                                if let None = with_unsafe.get(&call_id) {
                                    false
                                } else {
                                    true
                                }
                            } else {
                                // this case handled above
                                true
                            }
                        }) {
                        let fn_name =
                            if let Some(fn_id) = cx.tcx.hir.as_local_node_id(*def_id) {
                                ::get_node_name(cx, fn_id)
                            } else {
                                "ERROR!".to_string()
                            };
                        let info = UnsafeInBody::new(get_fn_path(cx, *def_id), true, fn_name);
                        with_unsafe.insert(*def_id, info);
                        changes = true;
                    }
                }
            }
        }
    }
    let mut result = Vec::new();
    for &fn_id in fns.iter() {
        let fn_def_id = cx.tcx.hir.local_def_id(fn_id);
        if let Some(elt) = with_unsafe.get(&fn_def_id) {
            let mut ub = UnsafeInBody::new(elt.def_path.clone(), elt.has_unsafe,
                                           ::get_node_name(cx,fn_id));
            result.push(ub);
        } else {
            result.push(UnsafeInBody::new(get_fn_path(cx,fn_def_id), false,
                                          ::get_node_name(cx,fn_id)));
        }
    }
    result
}


struct CallsVisitor<'a, 'tcx: 'a> {
    cx: &'a LateContext<'a, 'tcx>,
    mir: &'tcx Mir<'tcx>,
    fn_def_id: DefId,
    calls: Vec<Call>,
    uses_fn_ptr: bool,
    uses_unresolved_calls: bool,
}

impl<'a, 'tcx> CallsVisitor<'a, 'tcx> {
    fn new(cx: &'a LateContext<'a, 'tcx>, mir: &'tcx Mir<'tcx>, fn_def_id: DefId) -> Self {
        CallsVisitor { cx, mir, fn_def_id,
            calls: Vec::new(),
            uses_fn_ptr: false,
            uses_unresolved_calls: false}
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
            match func {
                Operand::Constant(constant) =>
                    if let TyKind::FnDef(callee_def_id, substs) = constant.literal.ty.sty {
                        let param_env = self.cx.tcx.param_env(self.fn_def_id);
                        if let Some(instance) = ty::Instance::resolve(self.cx.tcx, param_env, callee_def_id, substs) {

                            info!("func {:?} has type {:?}", func, instance);

                            match instance.def {
                                ty::InstanceDef::Item(def_id)
                                | ty::InstanceDef::Intrinsic(def_id)
                                | ty::InstanceDef::CloneShim(def_id,_) => {
                                    self.calls.push(Call::Static(def_id));
                                }
                                | ty::InstanceDef::Virtual(def_id, _) => {
                                    self.calls.push(Call::Dynamic);
                                }
                                _ => error!("ty::InstanceDef:: NOT handled {:?}", instance.def),
                            }
                        } else {
                            info!("no type for func: {:?}", func);
                            self.uses_unresolved_calls = true;
                        }
                    }
                _ => {
                    self.uses_fn_ptr = true;
                }
            }
//            match func.ty(&self.mir.local_decls,self.cx.tcx).sty {
//                TyKind::FnDef(callee_def_id, _substs) => {
//                    match self.cx.tcx.fn_sig(callee_def_id).unsafety() {
//                        hir::Unsafety::Unsafe => {} // do nothing; there must be a surrounding unsafe block
//                        hir::Unsafety::Normal => { self.calls.push(callee_def_id); }
//                    };
//                }
//                TyKind::FnPtr(ref poly_sig) => {
//                    self.uses_fn_ptr = true;
//                }
//                _ => {
//                    error ! ("TypeVariants NOT handled {:?}", func.ty(&self.mir.local_decls,self.cx.tcx).sty);
//                }
//            }

        }
    }
}



