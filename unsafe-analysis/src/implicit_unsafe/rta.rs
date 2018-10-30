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

use fxhash::{FxHashMap,FxHashSet};
use results::implicit::UnsafeResults;
use implicit_unsafe::UnsafeBlocksVisitorData;
use get_fn_path;
use std::hash::Hash;
use std::cmp::Eq;
use std::cmp::PartialEq;

#[derive(PartialEq,Eq,Hash,Debug,Clone,Copy)]
struct CallContext<'tcx> {
    def_id: DefId,
    substs:  Option<&'tcx ty::subst::Substs<'tcx>>,
}

#[derive(Debug)]
struct CallData<'tcx> {
    call_type: CallType,
    calls: Option<Vec<CallContext<'tcx>>>,
}

static MAX_DEPTH: usize = 32;

#[derive(Debug,PartialEq,Clone,Copy)]
enum CallType {
    NotSafe,
    Safe,
    Processing,
    ProcessingRecursion,
    Dependent,
}

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
                                      -> Vec<UnsafeResults> {
    let mut call_graph = FxHashMap::default();
    // couldn't figure out what lifetimes to use to make this an argument of CallType::Processing
    let mut recursive_calls = FxHashSet::default();
    for &fn_id in fns {
        let fn_def_id = cx.tcx.hir.local_def_id(fn_id);
        match cx.tcx.fn_sig(fn_def_id).unsafety() {
            hir::Unsafety::Unsafe => {} //ignore it
            hir::Unsafety::Normal => {
                let cc = CallContext {
                    def_id: fn_def_id,
                    substs: None,
                };
                // process only if it was not done so already
                if let None = call_graph.get(&cc) {
                    if has_unsafe_block(cx,fn_def_id) {
                        call_graph.insert(
                            CallContext{
                                def_id: fn_def_id,
                                substs: None,
                            },
                            CallData{
                                call_type: CallType::NotSafe,
                                calls: None,
                            }
                        );
                    } else {
                        let mut cd = CallData { call_type: CallType::Processing, calls: None };
                        call_graph.insert(CallContext {
                            def_id: fn_def_id,
                            substs: None,
                        }, cd);
                        let mir = &mut cx.tcx.optimized_mir(fn_def_id);
                        let mut calls_visitor = CallsVisitor::new(&cx, mir, &cc,
                                                                  &mut call_graph,
                                                                  &mut recursive_calls,
                                                                  optimistic);
                        calls_visitor.visit_mir(mir);
                    }
                }
            }
        }
    }

    error!("Call Graph +++++++++++++++++++++++++++++++++++++++++++");
    dump_call_graph(cx,&call_graph);

    //TODO
    Vec::new()

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
    recursive_calls: &'b mut FxHashSet<CallContext<'tcx>>,
    optimistic: bool,
}


impl<'a, 'b, 'tcx:'a+'b>  CallsVisitor<'a, 'b, 'tcx> {

    fn new(cx: &'a LateContext<'a, 'tcx>,
           mir: &'tcx Mir<'tcx>,
           fn_ctx: &'b CallContext<'tcx>,
           call_graph: &'b mut FxHashMap<CallContext<'tcx>,CallData<'tcx>>,
           recursive_calls: &'b mut FxHashSet<CallContext<'tcx>>,
           optimistic: bool,
    ) -> Self {
        CallsVisitor { cx, mir, fn_ctx, call_graph, recursive_calls, optimistic}
    }

    fn reset_recursive_calls(&mut self, call_type: CallType) {
        for cc in self.recursive_calls.iter() {
            if let Some (cd) = self.call_graph.get_mut(cc) {
                cd.call_type = call_type.clone();
            } else {assert!(false);}
        }
        self.recursive_calls.clear();
    }

    fn get_call_data_type( &self, call_data_opt: Option<&CallData<'tcx>> ) -> CallType {

        //error!("get_call_data_type {:?}", call_data_opt);

        let mut call_type = CallType::Processing;
        let calls_opt =
            if let Some(ref call_data) = call_data_opt {
                call_data.calls.clone()
            } else {
                assert!(false);
                None
            };
        if let Some(ref calls) = calls_opt {
            for cc in calls.iter() {
                if let Some(callee_call_data) = self.call_graph.get(cc) {

                    //error!("get_call_data_type callee_call_data {:?}", callee_call_data);

                    match callee_call_data.call_type {
                        CallType::NotSafe => {
                            call_type = CallType::NotSafe;
                            break;
                        }
                        CallType::Safe => { /* do nothing*/ }
                        CallType::Dependent => { // could not fully resolve types
                            call_type = CallType::Dependent;
                            break;
                        }
                        CallType::Processing => {
                            call_type = CallType::ProcessingRecursion;
                        }
                        CallType::ProcessingRecursion => {
                            // check if there is only one still processing
                            // if yes the cycle is complete
                            // if no keep ProcessingRecursion type
                            call_type = CallType::ProcessingRecursion;
                        }
                    }
                } else {
                    assert!(false)
                }
            }
        }
        if let Some(call_data) = call_data_opt {
            if call_type == CallType::Processing {
                //error!("Chaging from Processing to Safe for {:?}", self.fn_ctx);
                call_type = CallType::Safe;
            }
        }

        //error!("get_call_data_type call_type {:?}", call_type);

        call_type
    }


    // mutate in place the call data
    fn resolve(&mut self, ctxt: CallContext<'tcx>) {
        error!("Resolve {:?} {:?}", ctxt.def_id, ctxt.substs);

        if has_unsafe_block(self.cx, ctxt.def_id) {
            self.call_graph.insert(
                CallContext {
                    def_id: ctxt.def_id,
                    substs: None, // TODO think about this one
                },
                CallData {
                    call_type: CallType::NotSafe,
                    calls: None,
                }
            );
        } else {
            self.call_graph.insert(ctxt, CallData {
                call_type: CallType::Processing,
                calls: None,
            });

            // get calls for the method with no substitutions

            let no_substs_ctx = CallContext {
                def_id: ctxt.def_id,
                substs: None,
            };

            error!("Resolve no_substs_ctx: {:?} call_graph: {:?}", no_substs_ctx, self.call_graph.get(&no_substs_ctx));

            //check if a node exists for def_id with no substs
            let mut not_in_call_graph_no_subts = false; // for borrow checker
            if let None = self.call_graph.get(&no_substs_ctx) {
                not_in_call_graph_no_subts = true;
            }
            // call visitor on the no substitutions node if it does not exist in call graph
            // and if this is the node without substs that has not been processed yet
            if not_in_call_graph_no_subts || ctxt.substs == None {
                // Did not process yet this function
                let mir = &mut self.cx.tcx.optimized_mir(no_substs_ctx.def_id);
                let mut calls_visitor =
                    CallsVisitor::new(&self.cx, mir,
                                      &ctxt,
                                      self.call_graph,
                                      self.recursive_calls,
                                      self.optimistic);
                calls_visitor.visit_mir(mir);
            }
            if let None = ctxt.substs {
                // no substitutions
            } else {
                let mut call_data = CallData {
                    call_type: CallType::Processing,
                    calls: None,
                };
                let call_data_no_substs_opt =
                    {
                        if let Some(call_data_no_substs) = self.call_graph.get(&no_substs_ctx) {

                            //                    error!("no_substs_ctx: {:?}, call_data_no_substs: {:?}",
                            //                           no_substs_ctx, call_data_no_substs);

                            match call_data_no_substs.call_type {
                                CallType::NotSafe => {
                                    call_data.call_type = CallType::NotSafe;
                                    None
                                }
                                CallType::Safe => {
                                    call_data.call_type = CallType::Safe;
                                    None
                                }
                                CallType::Processing => {
                                    if self.recursive_calls.len() >= MAX_DEPTH {
                                        if self.optimistic {
                                            call_data.call_type = CallType::Safe;
                                        } else {
                                            call_data.call_type = CallType::NotSafe;
                                        }
                                    } else {
                                        self.recursive_calls.insert(ctxt);
                                        call_data.call_type = CallType::ProcessingRecursion;
                                    }
                                    None
                                }
                                CallType::ProcessingRecursion => {
                                    // TODO Is this possible
                                    assert!(false);
                                    None
                                }
                                CallType::Dependent => {
                                    if let Some(ref calls) = call_data_no_substs.calls {
                                        Some(calls.clone())
                                    } else {
                                        assert!(false);
                                        None
                                    }
                                }
                            }
                        } else {
                            assert!(false);
                            None
                        }
                    };
                if let Some(calls) = call_data_no_substs_opt {
                    for callee_ctxt in calls.iter() {
                        let mut cco = None;
                        let mut unresolved_type = false;
                        if let Some(caller_substs) = ctxt.substs {
                            if let Some(callee_substs) = callee_ctxt.substs {
                                let new_substs = callee_substs.subst(self.cx.tcx, caller_substs);

                                //                        error!("caller substs {:?}", caller_substs);
                                //                        error!("callee substs {:?}", callee_substs);
                                //                        error!("new substs substs {:?}", new_substs);

                                let param_env = self.cx.tcx.param_env(ctxt.def_id);
                                if let Some(instance) = ty::Instance::resolve(self.cx.tcx,
                                                                              param_env,
                                                                              callee_ctxt.def_id,
                                                                              new_substs) {
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
                                            assert!(false);
                                        }
                                        _ => {
                                            error!("ty::InstanceDef:: NOT handled {:?}", instance.def);
                                            assert!(false);
                                        }
                                    }
                                } else {
                                    // unresolved type, still trait method
                                    error!("no type for func: {:?}", ctxt.def_id);
                                    cco = Some(
                                        CallContext {
                                            def_id: callee_ctxt.def_id,
                                            substs: Some(new_substs),
                                        }
                                    );
                                    unresolved_type = true;
                                }
                                if let Some(cc) = cco {
                                    let mut needs_resolve = false; // for borrow checker
                                    if let None = self.call_graph.get(&cc) {
                                        needs_resolve = true;
                                    }
                                    if needs_resolve {
                                        self.resolve(CallContext {
                                            def_id: cc.def_id,
                                            substs: cc.substs,
                                        });
                                    }
                                    call_data.push(cc);
                                }
                            }
                        }
                    }
                }
                match call_data.call_type {
                    CallType::NotSafe => {
                        // make all ProcessingRecursive unsafe
                        //self.reset_recursive_calls(CallType::NotSafe);
                    }
                    CallType::Dependent => {
                        // TODO think of combination with recursion
                    }
                    CallType::Safe => {}
                    CallType::Processing => {
                        let call_type = self.get_call_data_type(Some(&call_data));
                        call_data.call_type = call_type;
                    }
                    CallType::ProcessingRecursion => {

                        //                error!("CallType::ProcessingRecursion def_id: {:?}, self.recursive_calls: {:?}",
                        //                    ctxt.def_id, self.recursive_calls
                        //                );

                        //check if there is only one left with Processing status
                        if self.recursive_calls.iter().filter(
                            |&c| {
                                if let Some(cd) = self.call_graph.get(c) {
                                    cd.call_type == CallType::Processing
                                } else {
                                    assert!(false);
                                    false
                                }
                            }
                        ).count() > 1 {
                            // if no keep status
                            call_data.call_type = CallType::ProcessingRecursion;
                        } else {
                            // if yes make all safe
                            call_data.call_type = CallType::Safe;
                            self.reset_recursive_calls(CallType::Safe);
                        }
                    }
                }

//                match call_data.call_type {
//                    CallType::NotSafe
//                    | CallType::Safe
//                    | CallType::Dependent => {
//                        self.reset_recursive_calls(call_data.call_type);
//                    }
//                    CallType::ProcessingRecursion => { /* Do nothing */ }
//                    CallType::Processing => { assert!(false); }
//                }

                self.call_graph.insert(ctxt, call_data);
                error!("Resolved {:?} calls: {:?}", ctxt.def_id, self.call_graph.get(&ctxt));
            }
        }
    }
}

impl<'a, 'b, 'tcx:'a+'b> Visitor<'tcx> for CallsVisitor<'a,'b,'tcx> {

    fn visit_mir(&mut self, mir: &Mir<'tcx>) {

        error!("Processing def_id:{:?}, substs: {:?}, call data: {:?}",
               self.fn_ctx.def_id, self.fn_ctx.substs, self.call_graph.get(self.fn_ctx));

        self.super_mir(mir);

        let mut call_type = self.get_call_data_type(self.call_graph.get(self.fn_ctx));

        error!("Call type after mir: {:?}", call_type);

        match call_type {
            CallType::NotSafe => {
                self.reset_recursive_calls(CallType::NotSafe)
            }
            CallType::Dependent => {
                self.reset_recursive_calls(CallType::Dependent)
            }
            CallType::Safe => {}
            CallType::Processing
            | CallType::ProcessingRecursion => {
                if !self.recursive_calls.iter().any(|c|{
                    if let Some (cd) = self.call_graph.get(c) {
                        cd.call_type == CallType::Processing
                    } else {false}
                }) {
                    call_type = CallType::Safe;
                    self.reset_recursive_calls(CallType::Safe);
                }
            }
        }

        error!("Done Processing {:?} call_data: {:?}", self.fn_ctx.def_id, self.call_graph.get(self.fn_ctx));

    }




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
            let must_process =
                if let Some(call_data) = self.call_graph.get(self.fn_ctx) {
                    if call_data.call_type != CallType::NotSafe {
                        true
                    } else {
                        false
                    }
                } else { assert!(false); false};
            if must_process {
                let mut not_safe = false;
                let mut cco = None;
                let mut unresolved_type = false;
                match func {
                    Operand::Constant(constant) => {
                        // Function Call
                        if let TyKind::FnDef(callee_def_id, substs) = constant.literal.ty.sty {
                            // combine substitutions
                            let new_substs =
                                if let Some(parent_substs) = self.fn_ctx.substs {
                                    substs.subst(self.cx.tcx, parent_substs)
                                } else {
                                    substs
                                };
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
                            } else {
                                // unresolved type, still trait method
                                error!("no type for func: {:?}", func);
                                cco = Some(
                                    CallContext {
                                        def_id: callee_def_id,
                                        substs: Some(new_substs),
                                    }
                                );
                                unresolved_type = true;
                            }
                        } else {
                            error!("Constant: type NOT handled {:?}", constant.literal.ty.sty);
                            assert!(false);
                        }
                    }
                    Operand::Copy(place)
                    | Operand::Move(place) => {
                        match func.ty(&self.mir.local_decls, self.cx.tcx).sty {
                            TyKind::FnPtr(ref poly_sig) => {
                                not_safe = true;
                            }
                            _ => {
                                error!("TyKind{:?}", func.ty(&self.mir.local_decls, self.cx.tcx).sty);
                                assert!(false);
                            }
                        }
                    }
                }

                if not_safe { // virtual call or function pointer and !optimistic
                    if let Some(call_data) = self.call_graph.get_mut(self.fn_ctx) {
                        call_data.call_type = CallType::NotSafe;
                        call_data.calls = None;
                    } else { assert!(false); };
                } else {
                    if unresolved_type {
                        if let Some(call_data) = self.call_graph.get_mut(self.fn_ctx) {
                            if let Some (cc) = cco {
                                call_data.call_type = CallType::Dependent;
                                call_data.push(cc);
                            } else { assert!(false); };
                        } else { assert!(false); };
                    } else {
                        error!("call {:?}", cco);
                        if let Some (cc) = cco {
                            let mut not_in_call_graph = false; // for borrow checker
                            if let None = self.call_graph.get(&cc) {
                                not_in_call_graph = true;
                            }
                            if not_in_call_graph {
                                self.resolve(CallContext{ //to please the borrow checker
                                    def_id: cc.def_id,
                                    substs: cc.substs
                                });
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
    for (d,c) in call_graph.iter() {
        error!("{:?} : {:?}", d, c);
    }
}


