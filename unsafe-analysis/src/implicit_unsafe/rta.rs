use syntax::ast::NodeId;
use rustc::hir;
use rustc::hir::def_id::DefId;
use rustc::mir::visit::Visitor;
use rustc::mir::{BasicBlock, Location, Operand, Terminator, TerminatorKind, Mir};
use rustc::ty::TyKind;
use rustc::ty::TypeFoldable;
use rustc::ty::subst::Substs;
use rustc::ty::subst::Subst;
use rustc::ty;
use rustc::lint::LateContext;



//use std::collections::HashMap;
use fxhash::FxHashMap;
use results::implicit::UnsafeResults;
use implicit_unsafe::UnsafeBlocksVisitorData;
use get_fn_path;

pub fn run_sources_analysis<'a, 'tcx>(cx: &LateContext<'a, 'tcx>
                                      , fns: &Vec<NodeId>, optimistic: bool)
                                      -> Vec<UnsafeResults> {
    let mut with_unsafe = FxHashMap::default();
    let mut call_graph = FxHashMap::default();
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
                    let mut info = UnsafeResults::Resolved(get_fn_path(cx,fn_def_id), true);
                    with_unsafe.insert(fn_def_id, info);
                } else {
                    let mir = &mut cx.tcx.optimized_mir(fn_def_id);
                    let mut calls_visitor = CallsVisitor::new(&cx,mir,fn_id);
                    calls_visitor.visit_mir(mir);
                    if calls_visitor.uses_fn_ptr && !optimistic {
                        let mut info = UnsafeResults::Resolved(get_fn_path(cx,fn_def_id), true);
                        with_unsafe.insert(fn_def_id, info);
                    } else {
                        call_graph.insert(fn_def_id, calls_visitor.calls);
                    }
                }
            }
        }
    }
    let rcg = resolve(cx, &call_graph);
    error!("Call Graph +++++++++++++++++++++++++++++++++++++++++++");
    dump_call_graph(cx,&call_graph);
    error!("Resolved Call Graph ++++++++++++++++++++++++++++++++++++");
    dump_call_graph(cx,&rcg);

    //TODO
    Vec::new()

}

fn dump_call_graph<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, call_graph: &FxHashMap<DefId,Vec<Call<'tcx>>>) {
    for (d,c) in call_graph.iter() {
        error!("{:?} : {:?}", d, c);
    }
}


struct CallContext<'tcx> {
    def_id: DefId,
    substs: &'tcx ty::subst::Substs<'tcx>,
}
fn resolve<'a, 'tcx>(cx: &LateContext<'a, 'tcx>
                     , call_graph: &FxHashMap<DefId,Vec<Call<'tcx>>>)
            -> FxHashMap<Key,Vec<Call<'tcx>>>  {
    let mut new_call_graph = FxHashMap::default();
    //propagate known types
    for (fn_def_id,calls) in call_graph.iter() {
        error!("Processing function {:?}", fn_def_id);
        let mut new_calls: Vec<Call<'tcx>> = Vec::new();
        let mut wl: Vec<(DefId, &Substs)> = Vec::new();
        for c in calls {
            match c {
                Call::Static(def_id, substs) => {
                    if let Some(calls1) = call_graph.get(&def_id) {
                        if calls1.is_empty() {
                            new_calls.push(c.clone());
                        } else {
                            error!("Add to wl {:?}", def_id);
                            wl.push((*def_id, substs));
                        }
                    } else {
                            error!("def id not in call graph {:?}", def_id);
                            wl.push((*def_id, substs));
                        }
                }
                Call::Virtual(..) => {
                    //TODO
                }
            }
        }
        while !wl.is_empty() {
            if let Some((def_id, substs)) = wl.pop() {
                error!("Removed from wl {:?}", def_id);
                error!("substs {:?}", substs);
                if let Some(calls) = call_graph.get(&def_id) {
                    if calls.is_empty() {
                        new_calls.push(Call::Static(def_id,substs));
                    } else {
                        for c in calls {
                            match c {
                                Call::Static(c_def_id, c_substs) => {
                                    error!("Call {:?}", c_def_id);
                                    error!("c_substs {:?}", c_substs);
//                                    for s in c_substs.iter() {
//                                        if let ty::subst::UnpackedKind::Type(t) = s.unpack() {
//                                            error!("{:?}", t.sty);
//                                            if let ty::TyKind::Param(param_ty) = t.sty {
//                                                error!("param_ty.to_ty {:?}",param_ty.to_ty(cx.tcx));
//                                            }
//                                            error!("fold_with {:?}", c_substs.fold_with(&mut t));
//                                        }
//                                    }

                                    let new_substs = c_substs.subst(cx.tcx,
                                        substs
                                    );

                                    error!("new_substs {:?}", new_substs);

                                        // need to merge substs and c_substs!!!!!!!!!
                                    //
                                    let  param_env = cx.tcx.param_env(def_id);
                                    if let Some(instance) = ty::Instance::resolve(
                                            cx.tcx,
                                            param_env,
                                            // Which substs do I care substs, c_substs????
                                            // neither replace with new_substs
                                            *c_def_id, new_substs) {
                                        match instance.def {
                                            ty::InstanceDef::Item(def_id)
                                            | ty::InstanceDef::Intrinsic(def_id)
                                            | ty::InstanceDef::CloneShim(def_id, _) => {
                                                // Which substs do I care substs, c_substs, ty_subts????
                                                let new_substs = instance.subst(cx.tcx,&c_substs).substs;
                                                error!("Add to wl {:?}", def_id);
                                                wl.push((*c_def_id, new_substs));
                                            }
                                            | ty::InstanceDef::Virtual(def_id, _) => {
                                                new_calls.push(Call::Virtual(def_id));
                                            }
                                            _ => error!("ty::InstanceDef:: NOT handled {:?}", instance.def),
                                        }

                                    } else {
                                        info!("STILL no type for func: {:?}", c_def_id);
                                        new_calls.push(Call::Static(def_id, substs));
                                    }
                                }
                                Call::Virtual(_) => {
                                    //TODO
                                }
                            }
                        }
                    }
                } else {
                    error!("wl def id not in call graph {:?}", def_id);
                }
            }
        }
        new_call_graph.insert(*fn_def_id, new_calls);
    }
    new_call_graph
}

#[derive(Clone,Debug)]
enum Call<'tcx> {
    Static (DefId, &'tcx ty::subst::Substs<'tcx>),
    Virtual(DefId), // def id of the trait method called
    FnPtr,
}


struct CallsVisitor<'a, 'tcx: 'a> {
    cx: &'a LateContext<'a, 'tcx>,
    mir: &'tcx Mir<'tcx>,
    fn_id: NodeId,
    calls: Vec<Call<'tcx>>,
    uses_fn_ptr: bool,
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
                Operand::Constant(constant) => {
                    if let TyKind::FnDef(callee_def_id, substs) = constant.literal.ty.sty {
                        let fn_def_id = self.cx.tcx.hir.local_def_id(self.fn_id);
                        let param_env = self.cx.tcx.param_env(fn_def_id);
                        if let Some(instance) = ty::Instance::resolve(self.cx.tcx, param_env, callee_def_id, substs) {
                            match instance.def {
                                ty::InstanceDef::Item(def_id)
                                | ty::InstanceDef::Intrinsic(def_id)
                                | ty::InstanceDef::CloneShim(def_id, _) => {
                                    self.calls.push(Call::Static(def_id, substs));
                                }
                                | ty::InstanceDef::Virtual(def_id, _) => {
                                    self.calls.push(Call::Virtual(def_id));
                                }
                                _ => error!("ty::InstanceDef:: NOT handled {:?}", instance.def),
                            }
                        } else {
                            error!("no type for func: {:?}", func);
                            self.calls.push(Call::Static(callee_def_id, substs));
                        }
                    } else {
                        error!("Constant: type NOT handled {:?}", constant.literal.ty.sty);
                    }
                }
                Operand::Copy (place)
                | Operand::Move (place) => {
                    if let TyKind::FnPtr(ref poly_sig) = constant.literal.ty.sty {
                        self.calls.push(Call::FnPtr);
                    } else {
                        error!("Others: type NOT handled {:?} place{:?}", self.fn_id, place);
                    }
                }
            }
        }
    }
}
