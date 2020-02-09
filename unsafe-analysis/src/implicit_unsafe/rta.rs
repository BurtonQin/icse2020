use syntax::ast::NodeId;
use rustc::hir::def_id::DefId;
use rustc::mir::visit::Visitor;
use rustc::mir::{BasicBlock, Location, Terminator, TerminatorKind, Mir};
use rustc::ty::TyKind;
use rustc::ty::subst::Substs;
use rustc::ty;
use rustc::lint::LateContext;
use rustc::hir;
use fxhash::{FxHashMap,FxHashSet};
use implicit_unsafe::UnsafeBlocksVisitorData;
use get_fn_path;
use implicit_unsafe;
use results::implicit::UnsafeInBody;
use results::implicit::FnType;
use implicit_unsafe::deps;

#[derive(PartialEq,Eq,Hash,Debug,Clone,Copy)]
struct CallContext<'tcx> {
    def_id: DefId,
    substs: &'tcx ty::subst::Substs<'tcx>,
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

        // TODO if the call is external clear substs
        let new_cc =
            if cc.def_id.is_local() {
                cc
            } else {
                CallContext{
                    def_id: cc.def_id,
                    //substs: ty::List::empty(),
                    substs: cc.substs, //TODO
                }
            };

        if let Some(ref mut v) = self.calls {
             v.push(new_cc);
        } else {
            let mut v = Vec::new();
            v.push(new_cc);
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
    for &fn_id in fns { // TODO change to
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
                if let TyKind::FnDef(_, _) = ty.sty {
                    let cc = CallContext {
                        def_id: fn_def_id,
                        substs: Substs::identity_for_item(cx.tcx,fn_def_id),
                    };
                    // process only if it was not done so already
                    if let None = call_graph.get(&cc) {
                        let mir = &mut cx.tcx.optimized_mir(fn_def_id);
                        let mut calls_visitor =
                            CallsVisitor::new(&cx, mir, &cc,
                                              &mut call_graph, &mut with_unsafe,
                                              &mut external_calls,
                                              optimistic, 0);
                        info!("+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++");
                        calls_visitor.visit_mir(mir);
                    }
                }
            }
        }
    }

//    error!("Call Graph +++++++++++++++++++++++++++++++++++++++++++");
//    dump_call_graph(cx,&call_graph);

    //load external calls info
    let implicit_external: FxHashMap<DefId,UnsafeInBody> =
        deps::load(cx, &external_calls, optimistic);

    for cc in call_graph.keys() {
        if !cc.def_id.is_local() || (cc.def_id.is_local() && !cx.tcx.mir_keys(hir::def_id::LOCAL_CRATE).contains(&cc.def_id) ){

            info!("Searching for {:?}", cc.def_id);

            if let Some(ub) = implicit_external.get(&cc.def_id) {

                info!("Found {:?}", ub);

                match ub.fn_type {
                    FnType::Safe => {},
                    FnType::Unsafe => {
                        with_unsafe.insert(cc.clone());
                    },
                    FnType::NormalNotSafe => {
                        with_unsafe.insert(cc.clone());
                    },
                    FnType::Parametric => {
                        if !optimistic {
                            with_unsafe.insert(cc.clone());
                        }
                    },
                }
            } else {
                info!("NOT found {:?}", cc.def_id);
                if !optimistic {
                    with_unsafe.insert(cc.clone());
                }
            }
        }
    }

//    for (&def_id, ref ub) in implicit_external.iter() {
//
//        info!("Checking {:?}: {:?}", def_id, ub);
//
//        if let FnType::NormalNotSafe = ub.fn_type {
//
//            info!("{:?} is NormalNotSafe", def_id);
//
//            // find all call contexts with same def id
//            for cc in call_graph.keys() {
//
//                info!("Processing {:?}", cc);
//
//                if cc.def_id == def_id {
//
//                    info!("Marking {:?} as unsafe", cc);
//
//                    with_unsafe.insert(
//                        cc.clone()
//                    );
//                }
//            }
//        }
//    }


    info!("external calls +++++++++++++++++++++++++++++++++++++++++++");
    for (def_path, def_id) in external_calls.iter() {
        info!("{:?}: {:?}", def_path, def_id);
    }

    info!("implicit external +++++++++++++++++++++++++++++++++++++++++++");
    for (def_id, ub) in implicit_external.iter() {
        info!("{:?} {:?}", def_id, ub);
    }


    info!("With Unsafe +++++++++++++++++++++++++++++++++++++++++++");
    for def_id in with_unsafe.iter() {
        info!("{:?}", def_id);
    }


    //reverse call graph

    let mut reverse_call_graph =  FxHashMap::default();

    for (caller_ctxt,call_data) in call_graph.iter() {
        match call_data.call_type {
            CallType::Processing => {
                error!("Ctxt still processing: {:?} data {:?} name: {:?}", caller_ctxt, call_data, ::get_node_name(cx, caller_ctxt.def_id));
                assert!(false);
            }
            CallType::Parametric => {
                //info!("Parametric call : call {:?} call_data {:?}", caller_ctxt, call_data);
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

    info!("Start propagation");
    while !worklist.is_empty() {
        if let Some (ctxt) = worklist.pop() {
            if let Some(call_data) = reverse_call_graph.get(&ctxt) {
                if let Some (ref calls) = call_data.calls {
                    for c in calls.iter() {
                        let visited =
                            if let None = with_unsafe.get(c) {
                                false
                            } else {
                                true
                            };
                        if !visited {
                            with_unsafe.insert(c.clone());
                            worklist.push(c.clone());
                        }
                    }
                }
            } // Nobody calls this function
        } else {assert!(false);}
    }
    info!("End propagation");

    // compile results
    for (ctxt,call_data) in call_graph.iter() {
        if ctxt.def_id.is_local() {
            match call_data.call_type {
                CallType::Resolved => {
                    if ctxt.substs == Substs::identity_for_item(cx.tcx,ctxt.def_id) {
                        // add only if it is the declared function
                        // if it is a specialization with given substs ignore
                        if with_unsafe.contains(ctxt) {
                            result.push(UnsafeInBody::new(get_fn_path(cx, ctxt.def_id),
                                                          FnType::NormalNotSafe,
                                                          ::get_node_name(cx, ctxt.def_id)));
                        } else {
                            result.push(UnsafeInBody::new(get_fn_path(cx, ctxt.def_id),
                                                          FnType::Safe,
                                                          ::get_node_name(cx, ctxt.def_id)));
                        }
                    }
                }
                CallType::Processing => { assert!(false); }
                CallType::Parametric => {
                    result.push(UnsafeInBody::new(
                        get_fn_path(cx, ctxt.def_id),
                        FnType::Parametric,
                        ::get_node_name(cx, ctxt.def_id)
                    ));
                }
            }
        }
    }

    info!("End building results");

    result

}

fn has_unsafe_block<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, fn_id: DefId) -> bool {
    let mut body_visitor = UnsafeBlocksVisitorData {
        hir: &cx.tcx.hir,
        has_unsafe: false,
    };
    if let Some (fn_node_id) = cx.tcx.hir.as_local_node_id(fn_id) {
        let body_id_opt = cx.tcx.hir.maybe_body_owned_by(fn_node_id);
        match body_id_opt {
            Some(body_id) => {
                let body = cx.tcx.hir.body(body_id);
                hir::intravisit::walk_body(&mut body_visitor, body);
                body_visitor.has_unsafe
            }
            None => false
        }
    } else {
        false
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
           call_graph: &'b mut FxHashMap<CallContext<'tcx>,CallData<'tcx>>,
           with_unsafe: &'b mut FxHashSet<CallContext<'tcx>>,
           external_calls: &'b mut FxHashMap<String,DefId>,
           optimistic: bool,
           depth: usize,
    ) -> Self {
        CallsVisitor { cx, mir, fn_ctx, call_graph, with_unsafe, external_calls, optimistic, depth}
    }

    fn has_parametric_call(&self, cd: &CallData<'tcx>) -> bool {
        match cd.calls {
            None => {false}
            Some (ref calls) => {
                for cc in calls.iter() {
                    if let Some (c) = self.call_graph.get(cc) {
                        if let CallType::Parametric = c.call_type {
                            return true
                        }
                    }
                }
                false
            }
        }
    }

    // mutate in place the call data
    fn resolve(&mut self, ctxt: CallContext<'tcx>) {
        self.depth += 1;
        info!("Resolve {:?} {:?} is_local {:?}", ctxt.def_id, ctxt.substs, ctxt.def_id.is_local());
        if !ctxt.def_id.is_local() {
            info!("Resolve {:?} external call", ctxt.def_id);
            // external call
            self.external_calls.insert(get_fn_path(self.cx, ctxt.def_id), ctxt.def_id);
            self.call_graph.insert(ctxt, CallData{
                call_type: CallType::Resolved,
                calls: None
            });
        } else { // if !ctxt.def_id.is_local()
            info!("Resolve {:?} local call", ctxt.def_id);
            // get calls for the method with no substitutions
            let no_substs_ctx = CallContext {
                def_id: ctxt.def_id,
                substs: Substs::identity_for_item(self.cx.tcx, ctxt.def_id),
            };
            //check if a node exists for def_id with no substs
            let mut not_in_call_graph_no_subts = false; // for borrow checker
            if let None = self.call_graph.get(&no_substs_ctx) {
                not_in_call_graph_no_subts = true;
            }
            // call visitor on the no substitutions node if it does not exist in call graph
            // and if this is the node without substs that has not been processed yet
            if not_in_call_graph_no_subts {
                //check if it has an MIR associated
                if self.cx.tcx.mir_keys(hir::def_id::LOCAL_CRATE).contains(&no_substs_ctx.def_id) {
                    // Did not process yet this function
                    let mir = &mut self.cx.tcx.optimized_mir(no_substs_ctx.def_id);
                    let mut calls_visitor =
                        CallsVisitor::new(&self.cx, mir,
                                          &no_substs_ctx,
                                          self.call_graph,
                                          self.with_unsafe,
                                          self.external_calls,
                                          self.optimistic,
                                          self.depth);
                    calls_visitor.visit_mir(mir);
                } else { // Can't find it, sometimes it is still external
                    info!("Resolve {:?} external call", ctxt.def_id);
                    // external call
                    self.external_calls.insert(get_fn_path(self.cx, ctxt.def_id), ctxt.def_id);
                    self.call_graph.insert(ctxt, CallData{
                        call_type: CallType::Resolved,
                        calls: None
                    });
                }
            }

            // insert node in call graph for this context if one does not exist
            if let None = self.call_graph.get(&ctxt) {
                self.call_graph.insert(ctxt, CallData {
                    call_type: CallType::Processing,
                    calls: None,
                });
            }

            if self.with_unsafe.contains(&no_substs_ctx) {
                self.with_unsafe.insert(ctxt.clone());
                if let Some(call_data) = self.call_graph.get_mut(&ctxt) {
                    call_data.call_type = CallType::Resolved;
                    call_data.calls = None;
                }
            } else {
                let mut call_data = CallData {
                    call_type: CallType::Processing,
                    calls: None,
                };
                let calls_no_substs_opt =
                    if let Some(cd) = self.call_graph.get(&no_substs_ctx) {
                        cd.calls.clone()
                    } else {
                        None
                    };
                if let Some(calls) = calls_no_substs_opt { // there are calls
                    for callee_ctxt in calls.iter() {
                        let mut cco = None;
                        let mut unresolved_type = false;
                        let param_env = self.cx.tcx.param_env(ctxt.def_id);
                        if let Some(instance) = ty::Instance::resolve(self.cx.tcx,
                                                                      param_env,
                                                                      callee_ctxt.def_id,
                                                                      callee_ctxt.substs) {
                            match instance.def {
                                ty::InstanceDef::Item(def_id)
                                | ty::InstanceDef::Intrinsic(def_id)
                                | ty::InstanceDef::CloneShim(def_id, _) => {
                                    if self.cx.tcx.is_closure(def_id) {
                                        //do nothing
                                        info!("closure {:?}", instance.def);
                                    } else {
                                        cco = Some(CallContext {
                                            def_id: def_id,
                                            substs: instance.substs,
                                        });
                                    }
                                }
                                _ => {
                                    error!("ty::InstanceDef:: NOT handled {:?}", instance.def);
                                    assert!(false);
                                }
                            }
                        } else {
                            // unresolved type, still trait method
                            cco = Some(
                                CallContext {
                                    def_id: callee_ctxt.def_id,
                                    substs: callee_ctxt.substs,
                                }
                            );
                            unresolved_type = true;
                        }

                        if unresolved_type {
                            call_data.call_type = CallType::Parametric
                        } else {
                            if let Some(cc) = cco {
                                let mut needs_resolve =
                                    if let None = self.call_graph.get(&cc) {
                                        true
                                    } else { false };
                                if self.depth < MAX_DEPTH {
                                    if needs_resolve {
                                        let cc = CallContext {
                                            def_id: cc.def_id,
                                            substs: cc.substs,
                                        };
                                        self.resolve(cc);
                                    }
                                    call_data.push(cc);
                                } else {}  // ignore the call
                            }
                        }

                    } // for
                    //if any callee is still parametric, this call is also parametric
                    if self.has_parametric_call(&call_data) {
                        call_data.call_type = CallType::Parametric
                    } else {
                        call_data.call_type = CallType::Resolved
                    }
                } else {// if let Some(calls) = calls_no_substs_opt
                    // no_subst had no calls
                    call_data.call_type = CallType::Resolved;
                    call_data.calls = None;
                }
                self.call_graph.insert(ctxt, call_data);
                //error!("Resolved {:?} calls: {:?}", ctxt.def_id, self.call_graph.get(&ctxt));
            }
        }
        self.depth -= 1;
        info!("Resolve END {:?} {:?} {:?}", ctxt.def_id, ctxt.substs, self.call_graph.get(&ctxt));
    }
}

impl<'a, 'b, 'tcx:'a+'b> Visitor<'tcx> for CallsVisitor<'a,'b,'tcx> {

    fn visit_mir(&mut self, mir: &Mir<'tcx>) {

        info!("visit_mir {:?}", self.fn_ctx);

        let mut must_process = true; // For borrow checker

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
                match func.ty(&self.mir.local_decls, self.cx.tcx).sty {
                    TyKind::FnDef(callee_def_id, callee_substs) => {
                        if implicit_unsafe::is_library_crate(&self.cx.tcx.crate_name(callee_def_id.krate).to_string()) {
                            // do nothing
                        } else {
                            // find actual method call
                            let param_env = self.cx.tcx.param_env(self.fn_ctx.def_id);
                            if let Some(instance) = ty::Instance::resolve(self.cx.tcx,
                                                                          param_env,
                                                                          callee_def_id,
                                                                          callee_substs) {
                                // Have a type for the function call
                                match instance.def {
                                    ty::InstanceDef::Item(def_id)
                                    | ty::InstanceDef::Intrinsic(def_id)
                                    | ty::InstanceDef::CloneShim(def_id, _) => {
                                        if self.cx.tcx.is_closure(def_id) {
                                            //do nothing
                                            info!("closure {:?}", instance.def);
                                        } else {
                                            cco = Some(CallContext {
                                                def_id: def_id,
                                                substs: instance.substs,
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
                                        substs: callee_substs,
                                    }
                                );
                                unresolved_type = true;
                            }
                        }
                    }
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
                        if let Some (mut cc) = cco {
                            let mut not_in_call_graph = None == self.call_graph.get(&cc);
                            if not_in_call_graph {
                                  self.resolve(CallContext {
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
            }
        }
    }
}


//////////////////////////////////////////////////////


fn dump_call_graph<'a, 'tcx>(cx: &LateContext<'a, 'tcx>,
                             call_graph: &FxHashMap<CallContext<'tcx>,CallData<'tcx>>) {
    error!("============================================================================================");
    for (d,c) in call_graph.iter() {
        error!("{:?} : {:?}", d, c);
    }
    error!("============================================================================================");
}


