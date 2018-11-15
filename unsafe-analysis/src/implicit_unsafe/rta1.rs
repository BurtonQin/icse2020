use syntax::ast::NodeId;
use rustc::hir;
use rustc::hir::def_id::DefId;
use rustc::mir::visit::Visitor;
use rustc::mir;
use rustc::mir::{BasicBlock, Location, Operand, Terminator, TerminatorKind, Mir};
use rustc::ty::TyKind;
use rustc::ty::TypeFoldable;
use rustc::ty::subst::Substs;
use rustc::ty::subst::Subst;
use rustc::ty;
use rustc::lint::LateContext;

use fxhash::{FxHashMap,FxHashSet};
use implicit_unsafe::UnsafeBlocksVisitorData;
use get_fn_path;
use std::hash::Hash;
use std::cmp::Eq;
use std::cmp::PartialEq;
use implicit_unsafe;
use results::implicit::UnsafeInBody;
use results::implicit::FnType;
use implicit_unsafe::deps;

#[derive(PartialEq,Eq,Hash,Debug,Clone,Copy)]
struct CallContext<'tcx> {
    def_id: DefId,
    substs:  Option<&'tcx ty::subst::Substs<'tcx>>,
}

#[derive(Debug,PartialEq,Clone,Copy)]
enum CallType {
    Parametric,
    Processing,
    Resolved,
}

#[derive(PartialEq,Debug)]
struct CallData<'tcx> {
    call_type: CallType,
    calls: Option<Vec<CallContext<'tcx>>>,
}

static MAX_DEPTH: usize = 1032;

impl<'tcx> CallData<'tcx> {
    fn push(&mut self, cc: CallContext<'tcx>) {
        if let Some(ref mut v) = self.calls {
            v.push(cc);
        } else {
            let mut v = Vec::new();
            v.push(cc);
            self.calls = Some (v);
        }
    }
}

pub fn run_sources_analysis<'a, 'tcx>(cx: &LateContext<'a, 'tcx>
                                      , fns: &Vec<NodeId>, optimistic: bool)
                                      -> Vec<UnsafeInBody> {
    let mut call_graph = FxHashMap::default();
    let mut with_unsafe = FxHashSet::default();
    let mut external_calls : FxHashMap<String,DefId> = FxHashMap::default();
    let mut result = Vec::new();

    // build call graph
    for &fn_id in fns {
        let fn_def_id = cx.tcx.hir.local_def_id(fn_id);
        match cx.tcx.fn_sig(fn_def_id).unsafety() {
            hir::Unsafety::Unsafe => {
                // call graph not needed for unsafe functions
                result.push(UnsafeInBody::new(get_fn_path(cx,fn_def_id),
                                              FnType::Unsafe,
                                              ::get_node_name(cx,fn_def_id)));
            }
            hir::Unsafety::Normal => {

                let ty = cx.tcx.type_of(fn_def_id);
                if let TyKind::FnDef(new_def_id, substs) = ty.sty {
                    info!("fn_def_id {:?} new_def_id {:?} substs {:?}", fn_def_id, new_def_id, substs);
                    // TODO figure out whe I might want None for substs
                    let cc = CallContext {
                        def_id: fn_def_id,
                        substs: Some (substs),
                    };
                    // process only if it was not done so already
                    if let None = call_graph.get(&cc) {
                        let mir = &mut cx.tcx.optimized_mir(fn_def_id);
                        let mut calls_visitor =
                            CallsVisitor::new(&cx, mir, &cc,
                                              &mut call_graph, &mut with_unsafe,
                                              &mut external_calls,
                                              optimistic, 0);
                        calls_visitor.visit_mir(mir);
                    }
                } else {
                    error!("fn_def_id {:?} not TyKind::FnDef", fn_def_id);
                }

            }
        }
    }

    info!("Call Graph +++++++++++++++++++++++++++++++++++++++++++");
    dump_call_graph(cx,&call_graph);

    //load external calls info
    let implicit_external: FxHashMap<DefId,UnsafeInBody> =
        deps::load(cx, &external_calls, optimistic, false);

    for (&def_id, ref ub) in implicit_external.iter() {
        if let FnType::NormalNotSafe = ub.fn_type {
            with_unsafe.insert(
                CallContext {
                    def_id: def_id,
                    substs: None,
                },
            );
        }
    }

//    error!("external calls +++++++++++++++++++++++++++++++++++++++++++");
//    for (def_id, ub) in external_calls.iter() {
//        error!("{:?} {:?}", def_id, ub);
//    }
//
//    error!("external +++++++++++++++++++++++++++++++++++++++++++");
//    for (def_id, ub) in implicit_external.iter() {
//        error!("{:?} {:?}", def_id, ub);
//    }
//
//
//    error!("With Unsafe +++++++++++++++++++++++++++++++++++++++++++");
//    for def_id in with_unsafe.iter() {
//        error!("{:?}", def_id);
//    }


    //reverse call graph

    let mut reverse_call_graph =  FxHashMap::default();

    for (caller_ctxt,call_data) in call_graph.iter() {
        match call_data.call_type {
            CallType::Processing => {
                error!("Ctxt still processing: {:?} data {:?} name: {:?}", caller_ctxt, call_data, ::get_node_name(cx, caller_ctxt.def_id));
                assert!(false);
            }
            CallType::Parametric => {
                info!("Parametric call : call {:?} call_data {:?}", caller_ctxt, call_data);
            }
            CallType::Resolved => {
                if !reverse_call_graph.contains_key(caller_ctxt) {
                    reverse_call_graph.insert(
                        caller_ctxt, CallData {
                            call_type: CallType::Processing,
                            calls: None,
                        }
                    );
                }
                match call_data.calls {
                    None => {}
                    Some (ref calls) => {
                        for callee_ctxt in calls.iter() {
                            let mut must_process = true; // for borrow checker
                            if let Some (call_data) = reverse_call_graph.get_mut(callee_ctxt) {
                                call_data.push(caller_ctxt.clone());
                                must_process = false;
                            }
                            if must_process {
                                let mut new_calls = Vec::new();
                                new_calls.push(caller_ctxt.clone());
                                let call_data = CallData{
                                    call_type: CallType::Resolved,
                                    calls: Some (new_calls),
                                };
                                reverse_call_graph.insert(callee_ctxt, call_data);
                            }
                        }
                    }
                }
            }
        }
    }


    // propagate unsafety

    let mut worklist = Vec::new();
    for ctxt in with_unsafe.iter() {
        worklist.push(ctxt.clone());
    }
    while !worklist.is_empty() {
        if let Some (ctxt) = worklist.pop() {
            if let Some(call_data) = reverse_call_graph.get(&ctxt) {
                if let Some (ref calls) = call_data.calls {
                    for c in calls.iter() {
                        with_unsafe.insert(c.clone());
                        worklist.push(c.clone());
                    }
                }
            } // Nobody calls this function
        } else {assert!(false);}
    }


    // compile results
    for (ctxt,call_data) in call_graph.iter() {
        // print only if it does not have substitutions
        if let None = ctxt.substs {
            match call_data.call_type {
                CallType::Resolved => {
                    if ctxt.def_id.is_local() {
                        if with_unsafe.contains(ctxt) {
                            result.push(UnsafeInBody::new(get_fn_path(cx, ctxt.def_id), FnType::NormalNotSafe, ::get_node_name(cx, ctxt.def_id)));
                        } else {
                            result.push(UnsafeInBody::new(get_fn_path(cx, ctxt.def_id), FnType::Safe, ::get_node_name(cx, ctxt.def_id)));
                        }
                    }
                }
                CallType::Processing => { assert!(false); }
                CallType::Parametric => {
                    if ctxt.def_id.is_local() {
                        result.push(UnsafeInBody::new(
                            get_fn_path(cx, ctxt.def_id),
                            FnType::Parametric,
                            ::get_node_name(cx, ctxt.def_id)
                        ));
                    }
                }
            }
        }
    }

    result

}

fn has_unsafe_block<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, fn_id: DefId) -> bool {
    let mut body_visitor = UnsafeBlocksVisitorData {
        hir: &cx.tcx.hir,
        has_unsafe: false,
    };
    if let Some (fn_node_id) = cx.tcx.hir.as_local_node_id(fn_id) {
        let body_id = cx.tcx.hir.body_owned_by(fn_node_id);
        let body = cx.tcx.hir.body(body_id);
        hir::intravisit::walk_body(&mut body_visitor, body);
        body_visitor.has_unsafe
    } else {
        // this should not happen
        assert!(true);
        true
    }
}

//////////////////////////////////////////////////////////////
struct CallsVisitor<'a, 'b, 'tcx:'a+'b> {
    cx: &'a LateContext<'a, 'tcx>,
    mir: &'tcx Mir<'tcx>,
    fn_ctx: &'b CallContext<'tcx>,
    call_graph: &'b mut FxHashMap<CallContext<'tcx>,CallData<'tcx>>,
    with_unsafe: &'b mut FxHashSet<CallContext<'tcx>>,
    external_calls: &'b mut FxHashMap<String,DefId>,
    optimistic: bool,
    depth: usize,
}


impl<'a, 'b, 'tcx:'a+'b>  CallsVisitor<'a, 'b, 'tcx> {
    fn new(cx: &'a LateContext<'a, 'tcx>,
           mir: &'tcx Mir<'tcx>,
           fn_ctx: &'b CallContext<'tcx>,
           call_graph: &'b mut FxHashMap<CallContext<'tcx>, CallData<'tcx>>,
           with_unsafe: &'b mut FxHashSet<CallContext<'tcx>>,
           external_calls: &'b mut FxHashMap<String, DefId>,
           optimistic: bool,
           depth: usize,
    ) -> Self {
        CallsVisitor { cx, mir, fn_ctx, call_graph, with_unsafe, external_calls, optimistic, depth }
    }

    fn has_parametric_call(&self, cd: &CallData<'tcx>) -> bool {
        match cd.calls {
            None => { false }
            Some(ref calls) => {
                for cc in calls.iter() {
                    if let Some(c) = self.call_graph.get(cc) {
                        if let CallType::Parametric = c.call_type {
                            return true
                        }
                    }
                }
                false
            }
        }
    }
}

impl<'a, 'b, 'tcx:'a+'b> Visitor<'tcx> for CallsVisitor<'a,'b,'tcx> {

    fn visit_mir(&mut self, mir: &Mir<'tcx>) {

        info!("visit_mir {:?}", self.fn_ctx);

        let mut must_process = true;

        if self.fn_ctx.def_id.is_local() {
            // don't process functions that have an unsafe block
            if has_unsafe_block(self.cx, self.fn_ctx.def_id) {
                self.call_graph.insert(
                    self.fn_ctx.clone(),
                    CallData {
                        call_type: CallType::Resolved,
                        calls: None,
                    }
                );
                self.with_unsafe.insert(self.fn_ctx.clone());
                must_process = false;
            }
        } else {
            self.external_calls.insert(get_fn_path(self.cx, self.fn_ctx.def_id), self.fn_ctx.def_id);
        }

        if must_process {
            self.call_graph.insert(
                self.fn_ctx.clone(),
                CallData {
                    call_type: CallType::Processing,
                    calls: None });

            self.super_mir(mir);

            let parametric =
                if let Some(call_data) = self.call_graph.get( self.fn_ctx ) {
                    if self.has_parametric_call(&call_data) {
                        true
                    } else {false}
                } else {false};
            if let Some(call_data) = self.call_graph.get_mut( self.fn_ctx ) {
                if call_data.call_type == CallType::Processing {
                    if parametric {
                        call_data.call_type = CallType::Parametric
                    } else {
                        call_data.call_type = CallType::Resolved
                    }
                }
            } else {assert!(false);}

            //error!("Done Processing {:?} call_data: {:?}", self.fn_ctx.def_id, self.call_graph.get(self.fn_ctx));
        }

        //dump_call_graph(self.cx, self.call_graph);

//        error!("visit_mir ended {:?} call_data: {:?}", self.fn_ctx, self.call_graph.get(self.fn_ctx));

    }


    fn visit_terminator( &mut self, _: BasicBlock, terminator: &Terminator<'tcx>, _: Location, ) {
        if let TerminatorKind::Call {ref func, args: _, destination: _, cleanup: _} = terminator.kind {
            if !self.with_unsafe.contains(self.fn_ctx) {
                let mut not_safe = false;
                let mut unresolved_type = false;
                let mut cco = None;
                match func {
                    Operand::Constant(constant) => {
                        // Function Call
                        if let TyKind::FnDef(callee_def_id, callee_subst) = constant.literal.ty.sty {
                            if implicit_unsafe::is_library_crate(&self.cx.tcx.crate_name(callee_def_id.krate).to_string()) {
                                // do nothing
                            } else {
                                let callee_id_substs = Substs::identity_for_item(self.cx.tcx,callee_def_id);
                                info!("callee_id_substs {:?}", callee_id_substs);
                                // combine callee substs

                                let s1 =
                                    if let Some (caller_substs) = self.fn_ctx.substs {
                                        callee_id_substs.subst(self.cx.tcx, callee_subst)
                                    } else {
                                        callee_id_substs
                                    };
                                let new_substs = s1.subst(self.cx.tcx, callee_subst);

                                info!("new_substs {:?}", new_substs);
                                // find actual method call
                                let param_env = self.cx.tcx.param_env(self.fn_ctx.def_id);
                                if let Some(instance) = ty::Instance::resolve(self.cx.tcx,
                                                                              param_env,
                                                                              callee_def_id,
                                                                              new_substs) {
                                    // Have a type for the function call
                                    match instance.def {
                                        ty::InstanceDef::Item(def_id)
                                        | ty::InstanceDef::Intrinsic(def_id)
                                        | ty::InstanceDef::CloneShim(def_id, _) => {
                                            if self.cx.tcx.is_closure(def_id) {
                                                //do nothing
                                                error!("closure {:?}", instance.def);
                                            } else {
                                                cco = Some(CallContext {
                                                    def_id: def_id,
                                                    substs: if new_substs.len() == 0 {
                                                        None
                                                    } else {
                                                        Some(new_substs)
                                                    },
                                                });
                                            }
                                        }
                                        | ty::InstanceDef::Virtual(def_id, _) => {
                                            if !self.optimistic { // virtual call, uses dynamic dispatch
                                                not_safe = true;
                                            }
                                        }
                                        _ => {
                                            error!("ty::InstanceDef:: NOT handled {:?}", instance.def);
                                            assert!(false);
                                        }
                                    }
                                } else { // if let Some(instance) = ty::Instance::resolve
                                    // unresolved type, still trait method
                                    //error!("no type for func: {:?}", func);
                                    cco = Some(
                                        CallContext {
                                            def_id: callee_def_id,
                                            substs: Some(new_substs),
                                        }
                                    );
                                    unresolved_type = true;
                                }
                            }
                        } else { // if let TyKind::FnDef(callee_def_id, substs) = constant.literal.ty.sty
                            error!("Constant: type NOT handled {:?}", constant.literal.ty.sty);
                            assert!(false);
                        }
                    }
                    Operand::Copy(place)
                    | Operand::Move(place) => {
                        match func.ty(&self.mir.local_decls, self.cx.tcx).sty {
                            TyKind::FnPtr(ref poly_sig) => {
                                if !self.optimistic {
                                    not_safe = true;
                                }
                            }
                            _ => {
                                error!("TyKind{:?}", func.ty(&self.mir.local_decls, self.cx.tcx).sty);
                                assert!(false);
                            }
                        }
                    }
                }

                if not_safe { // virtual call or function pointer and !optimistic
                    self.with_unsafe.insert(self.fn_ctx.clone());
                    if let Some(call_data) = self.call_graph.get_mut(self.fn_ctx) {
                        call_data.calls = None;
                        call_data.call_type = CallType::Resolved;
                    } else { assert!(false); };
                } else {
                    if unresolved_type {
                        if let Some(call_data) = self.call_graph.get_mut(self.fn_ctx) {
                            if let Some (cc) = cco {
                                call_data.call_type = CallType::Parametric;
                                call_data.push(cc);
                            } else { assert!(false); };
                        } else { assert!(false); };
                    } else {
                        if let Some (cc) = cco {
//                            error!(" visit_terminator call {:?} {:?}", cco, self.call_graph.get(&cc));
                            if cc.substs != None {
                                let mut not_in_call_graph = None == self.call_graph.get(&cc);
                                if not_in_call_graph {
                                    if self.cx.tcx.mir_keys(hir::def_id::LOCAL_CRATE).contains(&cc.def_id) {
                                        // Did not process yet this function
                                        let mir = &mut self.cx.tcx.optimized_mir(cc.def_id);
                                        let mut calls_visitor =
                                            CallsVisitor::new(&self.cx, mir,
                                                              &cc,
                                                              self.call_graph,
                                                              self.with_unsafe,
                                                              self.external_calls,
                                                              self.optimistic,
                                                              self.depth);
                                        calls_visitor.visit_mir(mir);
                                    } else {
                                        error!("No MIR for {:?}", &cc);
                                        return;
                                    }
                                }
                            }
                            if let Some(call_data) = self.call_graph.get_mut(self.fn_ctx) {
                                call_data.push(cc);
                            } else { assert!(false); };
                        }
                    }
                }
            } else { assert!(false); }
        }
    }
}


//////////////////////////////////////////////////////


fn dump_call_graph<'a, 'tcx>(cx: &LateContext<'a, 'tcx>,
                             call_graph: &FxHashMap<CallContext<'tcx>,CallData<'tcx>>) {
    info!("============================================================================================");
    for (d,c) in call_graph.iter() {
        info!("{:?} : {:?}", d, c);
    }
    info!("============================================================================================");
}

