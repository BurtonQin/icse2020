use syntax::ast::NodeId;
use rustc::hir;
use rustc::hir::def_id::DefId;
use rustc::mir::visit::Visitor;
use rustc::mir::{BasicBlock, Location, Operand, Terminator, TerminatorKind, Mir};
use rustc::ty::TyKind;
use rustc::ty;
use rustc::lint::LateContext;

use std::collections::HashMap;
use results::implicit::UnsafeResults;
use implicit_unsafe::UnsafeBlocksVisitorData;
use get_fn_path;

pub fn run_sources_analysis<'a, 'tcx>(cx: &LateContext<'a, 'tcx>
                                      , fns: &Vec<NodeId>, optimistic: bool)
                                      -> Vec<UnsafeResults> {
    let mut with_unsafe = HashMap::new();
    //let mut call_graph = HashMap::new();
    for &fn_id in fns {
        let fn_def_id = cx.tcx.hir.local_def_id(fn_id);
        match cx.tcx.fn_sig(fn_def_id).unsafety() {
            hir::Unsafety::Unsafe => {} //ignore it
            hir::Unsafety::Normal => {
                error!("Processing function {:?}", fn_def_id);
                let mut body_visitor = UnsafeBlocksVisitorData {
                    hir: &cx.tcx.hir,
                    has_unsafe: false,
                };
                let body_id = cx.tcx.hir.body_owned_by(fn_id);
                let body = cx.tcx.hir.body(body_id);
                hir::intravisit::walk_body(&mut body_visitor, body);
                if body_visitor.has_unsafe {
                    let mut info = UnsafeResults::Resolved(get_fn_path(cx,fn_def_id), true);
                    with_unsafe.insert(fn_def_id, info);
                } else {
                    let mir = &mut cx.tcx.optimized_mir(fn_def_id);
                    let mut calls_visitor = CallsVisitor::new(&cx,mir,fn_id);
                    calls_visitor.visit_mir(mir);
                }
            }
        }
    }

    //TODO
    Vec::new()

}

enum CallTypes {
    External (DefId),
    Local (DefId),
    TraitObject(),
    FnPtr(),
    //method called
    SelfCall(DefId),
    ParametricCall(),
}

struct CallsVisitor<'a, 'tcx: 'a> {
    cx: &'a LateContext<'a, 'tcx>,
    mir: &'tcx Mir<'tcx>,
    fn_id: NodeId,
    calls: Vec<CallTypes>,
    uses_fn_ptr: bool,
}

impl <'a, 'tcx> CallsVisitor<'a, 'tcx> {
    fn is_associated_method_with_self(self: &Self) -> bool {
        //TODO should return true if this is default method of a trait with self
        let node = self.cx.tcx.hir.get(self.fn_id);
        match node {
            hir::Node::TraitItem(ref trait_item) => {
                match trait_item.node {
                    hir::TraitItemKind::Method(ref method_sig, ref trait_method) => {
                        match trait_method {
                            hir::TraitMethod::Provided(_) => {
                                let fn_def_id = self.cx.tcx.hir.local_def_id(self.fn_id);
                                self.cx.tcx.associated_item(fn_def_id).method_has_self_argument
                            }
                            _ => {false}
                        }
                    }
                    _ => {false}
                }
            }
            _ => false
        }
    }

    fn is_method_same_trait(self: &Self, def_id: DefId) -> bool {
        if let Some (trait_id) = self.cx.tcx.trait_of_item(def_id) {
            let fn_def_id = self.cx.tcx.hir.local_def_id(self.fn_id);
            if let Some (fn_trait_id) = self.cx.tcx.trait_of_item(fn_def_id) {
                if fn_trait_id == trait_id {
                    self.cx.tcx.associated_item(def_id).method_has_self_argument
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }

}

impl<'a, 'tcx> CallsVisitor<'a, 'tcx> {
    fn new(cx: &'a LateContext<'a, 'tcx>, mir: &'tcx Mir<'tcx>, fn_id: NodeId) -> Self {
        CallsVisitor { cx, mir, fn_id, calls: Vec::new(),  uses_fn_ptr: false}
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
                        let fn_def_id = self.cx.tcx.hir.local_def_id(self.fn_id);
                        let param_env = self.cx.tcx.param_env(fn_def_id);
                        if let Some(instance) = ty::Instance::resolve(self.cx.tcx, param_env, callee_def_id, substs) {
                            match instance.def {
                                ty::InstanceDef::Item(def_id)
                                | ty::InstanceDef::Intrinsic(def_id)
                                | ty::InstanceDef::Virtual(def_id, _)
                                | ty::InstanceDef::CloneShim(def_id,_) => {
                                    if def_id.is_local() {
                                        self.calls.push(CallTypes::Local(def_id));
                                    } else {
                                        self.calls.push(CallTypes::External(def_id));
                                    }
                                }
                                _ => error!("ty::InstanceDef:: NOT handled {:?}", instance.def),
                            }
                        } else {
                            let mut resolved = false;
                            // default trait method (func), self type (param to callee)
                            if self.is_associated_method_with_self() {
                                // need to check is callee_def_id is method in trait
                                if self.is_method_same_trait(callee_def_id) {
                                    self.calls.push( CallTypes::SelfCall(callee_def_id) );
                                    resolved = true;
                                }
                            }
                            // method of generic type parameter (generic from method or trait defns)
                            // self.cx.tcx.generics_of(fn_def_id)

                            error!("substs {:?}", substs);
                            error!("generics of method {:?}", self.cx.tcx.generics_of(fn_def_id));
                            error!("generics of calee {:?}", self.cx.tcx.generics_of(callee_def_id));

                            // function pointer
                            if self.cx.tcx.is_closure(callee_def_id) {
                                error!("is closure {:?}", callee_def_id);
                                resolved = true; // nothing to do; the closure's body is parsed in the enclosing function
                            }

                            if !resolved {
                                error!("Type not resolved for call {:?}", func);
                                error!("calee def id {:?}", callee_def_id);
                            }
                        }
                    }
                _ => {
                    error!("func not handled ")
                }
            }
        }
    }
}
