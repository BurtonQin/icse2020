//use rustc::hir;
//use rustc::ty;
//use rustc::mir::visit::{Visitor, PlaceContext};
//use rustc::mir::{
//    AggregateKind, BasicBlock, ClearCrossCrate, Location, Mir, Place, Projection,
//    ProjectionElem, Rvalue, Safety, SourceInfo, SourceScope, SourceScopeLocalData, Statement,
//    StatementKind, Static, OUTERMOST_SOURCE_SCOPE,
//};
//use rustc_mir;
//use rustc_data_structures::indexed_vec::IndexVec;
//use syntax::ast::NodeId;
//use rustc::lint::LateContext;

mod deps;
pub mod rta;


//struct UnsafeBlocksVisitorData<'a, 'tcx: 'a>  {
//    cx: &'a LateContext<'a, 'tcx>,
//    fn_node_id: NodeId,
//    mir: &'a Mir<'tcx>,
//    param_env: ty::ParamEnv<'tcx>,
//    source_info: SourceInfo,
//    source_scope_local_data: &'a IndexVec<SourceScope, SourceScopeLocalData>,
//    has_unsafe: bool,
//}
//
//// change this to check for presence of unsafe operations
//impl<'a, 'tcx> Visitor<'tcx> for UnsafeBlocksVisitorData<'a, 'tcx> {
//    fn visit_statement(
//        &mut self,
//        block: BasicBlock,
//        statement: &Statement<'tcx>,
//        location: Location,
//    ) {
//        self.source_info = statement.source_info;
//        match statement.kind {
//            StatementKind::InlineAsm { .. } => {
//                self.has_unsafe = true;
//            }
//            _ => {}
//        }
//        self.super_statement(block, statement, location);
//    }
//
//    fn visit_rvalue(&mut self, rvalue: &Rvalue<'tcx>, location: Location) {
//        if let &Rvalue::Aggregate(box ref aggregate, _) = rvalue {
//            match aggregate {
//                &AggregateKind::Array(..) | &AggregateKind::Tuple | &AggregateKind::Adt(..) => {}
//                &AggregateKind::Closure(def_id, _) | &AggregateKind::Generator(def_id, _, _) => {
//                    let mir = &mut self.cx.tcx.optimized_mir(def_id);
//                    if !self.has_unsafe {
//                        let mut body_visitor = UnsafeBlocksVisitorData{
//                            cx: self.cx,
//                            fn_node_id: self.fn_node_id,
//                            mir,
//                            param_env: self.cx.tcx.param_env(def_id),
//                            source_info: self.source_info,
//                            source_scope_local_data: self.source_scope_local_data,
//                            has_unsafe: false,
//                        };
//                        body_visitor.visit_mir(mir);
//                    }
//                }
//            }
//        }
//        self.super_rvalue(rvalue, location);
//    }
//
//    fn visit_place(
//        &mut self,
//        place: &Place<'tcx>,
//        context: PlaceContext<'tcx>,
//        location: Location,
//    ) {
//        if let PlaceContext::Borrow { .. } = context {
//            if rustc_mir::util::is_disaligned(self.cx.tcx, self.mir, self.param_env, place) {
//                self.has_unsafe = true;
//            }
//        }
//
//        match place {
//            &Place::Projection(box Projection { ref base, ref elem }) => {
//                let old_source_info = self.source_info;
//                if let &Place::Local(local) = base {
//                    if self.mir.local_decls[local].internal {
//                        // Internal locals are used in the `move_val_init` desugaring.
//                        // We want to check unsafety against the source info of the
//                        // desugaring, rather than the source info of the RHS.
//                        self.source_info = self.mir.local_decls[local].source_info;
//                    }
//                }
//                let base_ty = base.ty(self.mir, self.cx.tcx).to_ty(self.cx.tcx);
//                match base_ty.sty {
//                    ty::TyKind::RawPtr(..) => {
//                        self.has_unsafe = true;
//                    }
//                    ty::TyKind::Adt(adt, _) => {
//                        if adt.is_union() {
//                            if context == PlaceContext::Store
//                                || context == PlaceContext::AsmOutput
//                                || context == PlaceContext::Drop
//                            {
//                                let elem_ty = match elem {
//                                    &ProjectionElem::Field(_, ty) => ty,
//                                    _ => span_bug!(
//                                            self.source_info.span,
//                                            "non-field projection {:?} from union?",
//                                            place
//                                        ),
//                                };
//                                if elem_ty.moves_by_default(
//                                    self.cx.tcx,
//                                    self.param_env,
//                                    self.source_info.span,
//                                ) {
//                                    self.has_unsafe = true;
//                                } else {
//                                    // write to non-move union, safe
//                                }
//                            } else {
//                                let source_info = self.source_info;
//                                self.has_unsafe = true;
//                            }
//                        }
//                    }
//                    _ => {}
//                }
//                self.source_info = old_source_info;
//            }
//            &Place::Local(..) => {
//                // locals are safe
//            }
//            &Place::Promoted(ref _p) => {
//                //TODO find out what this is
//            }
//            &Place::Static(box Static { def_id, ty: _ }) => {
//                let source_info = self.source_info;
//                if self.cx.tcx.is_static(def_id) == Some(hir::Mutability::MutMutable) {
//                    self.has_unsafe = true;
//                } else if self.cx.tcx.is_foreign_item(def_id) {
//                    self.has_unsafe = true;
//                }
//            }
//        };
//        self.super_place(place, context, location);
//    }
//}
//impl<'tcx, 'a> UnsafeBlocksVisitorData<'tcx, 'a> {
//
//    pub fn new(
//        cx: &'a LateContext<'a, 'tcx>,
//        mir: &'a Mir<'tcx>,
//        fn_def_id: hir::def_id::DefId,
//    ) -> Option<Self> {
//        match mir.source_scope_local_data {
//            ClearCrossCrate::Set(ref local_data) => Some(UnsafeBlocksVisitorData {
//                cx,
//                fn_node_id: cx.tcx.hir.def_index_to_node_id(fn_def_id.index),
//                mir,
//                param_env: cx.tcx.param_env(fn_def_id),
//                source_info: SourceInfo {
//                    span: mir.span,
//                    scope: OUTERMOST_SOURCE_SCOPE,
//                },
//                source_scope_local_data: local_data,
//                has_unsafe: false,
//            }),
//            ClearCrossCrate::Clear => {
//                info!("unsafety_violations: {:?} - remote, skipping", fn_def_id);
//                None
//            }
//        }
//    }
//
//    fn is_unsafety_user_provided(&self, source_info: SourceInfo) -> bool {
//        match self.source_scope_local_data[source_info.scope].safety {
//            Safety::Safe => {
//                //assert!(false, "loc = {:?}", source_info);
//                // it appears on alloc::alloc::box_free
//                false
//            },
//            Safety::BuiltinUnsafe => { false } // TODO check this
//            Safety::FnUnsafe => { true },
//            Safety::ExplicitUnsafe(node_id) => {
//                let node = self.cx.tcx.hir.get(node_id);
//                if let hir::Node::Block(block) = node {
//                    match block.rules {
//                        hir::BlockCheckMode::DefaultBlock => {
//                            false
//                        }
//                        hir::BlockCheckMode::UnsafeBlock(unsafe_source) |
//                        hir::BlockCheckMode::PushUnsafeBlock(unsafe_source) |
//                        hir::BlockCheckMode::PopUnsafeBlock(unsafe_source) => {
//                            match unsafe_source {
//                                hir::UnsafeSource::UserProvided => {
//                                    true
//                                }
//                                hir::UnsafeSource::CompilerGenerated => {
//                                    false
//                                }
//                            }
//                        }
//                    }
//                } else {
//                    assert!(false); false
//                }
//            },
//        }
//    }
//}

pub fn is_library_crate(crate_name: &String) -> bool {
    crate_name.as_str() == "alloc" ||
        crate_name.as_str() == "std" ||
        crate_name.as_str() == "core" ||
        crate_name.as_str() == "proc_macro" ||
        crate_name.as_str() == "clippy"
    //false
}