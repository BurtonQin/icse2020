use rustc::hir;
use rustc::lint::LateContext;
use rustc::mir::visit::{PlaceContext, Visitor};
use rustc::mir::{
    AggregateKind, BasicBlock, ClearCrossCrate, Location, Mir, Place, Projection,
    ProjectionElem, Rvalue, Safety, SourceInfo, SourceScope, SourceScopeLocalData, Statement,
    StatementKind, Static, Terminator, TerminatorKind, OUTERMOST_SOURCE_SCOPE,
};
use rustc::ty;
use rustc_data_structures::indexed_vec::IndexVec;
use rustc_mir;
use rustc_target;
use syntax::ast::NodeId;

use results::unsafety_sources::SourceKind;
use convert_abi;

pub trait UnsafetySourceCollector {
    fn add_unsafety_source<'a, 'tcx>(
        &mut self,
        cx: &LateContext<'a, 'tcx>,
        kind: SourceKind,
        source_info: SourceInfo,
        node_id: NodeId,
    );
}

pub struct UnsafetySourcesVisitor<'a, 'tcx: 'a> {
    cx: &'a LateContext<'a, 'tcx>,
    fn_node_id: NodeId,
    mir: &'a Mir<'tcx>,
    data: &'a mut UnsafetySourceCollector,
    param_env: ty::ParamEnv<'tcx>,
    source_info: SourceInfo,
    source_scope_local_data: &'a IndexVec<SourceScope, SourceScopeLocalData>,
}

impl<'a, 'tcx> UnsafetySourcesVisitor<'a, 'tcx> {
    pub fn new(
        cx: &'a LateContext<'a, 'tcx>,
        mir: &'a Mir<'tcx>,
        data: &'a mut UnsafetySourceCollector,
        fn_def_id: hir::def_id::DefId,
    ) -> Option<Self> {
        match mir.source_scope_local_data {
            ClearCrossCrate::Set(ref local_data) => Some(UnsafetySourcesVisitor {
                cx,
                fn_node_id: cx.tcx.hir.def_index_to_node_id(fn_def_id.index),
                mir,
                data,
                param_env: cx.tcx.param_env(fn_def_id),
                source_info: SourceInfo {
                    span: mir.span,
                    scope: OUTERMOST_SOURCE_SCOPE,
                },
                source_scope_local_data: local_data,
            }),
            ClearCrossCrate::Clear => {
                error!("unsafety_violations: {:?} - remote, skipping", fn_def_id);
                None
            }
        }
    }

    fn get_unsafety_node_id(&self) -> NodeId {
        match self.source_scope_local_data[self.source_info.scope].safety {
            Safety::Safe => self.fn_node_id,
            Safety::BuiltinUnsafe | Safety::FnUnsafe => self.fn_node_id,
            Safety::ExplicitUnsafe(node_id) => node_id,
        }
    }
}


impl<'a, 'tcx> Visitor<'tcx> for UnsafetySourcesVisitor<'a, 'tcx> {
    fn visit_terminator(
        &mut self,
        block: BasicBlock,
        terminator: &Terminator<'tcx>,
        location: Location,
    ) {
        self.source_info = terminator.source_info;
        match terminator.kind {
            TerminatorKind::Goto { .. }
            | TerminatorKind::SwitchInt { .. }
            | TerminatorKind::Drop { .. }
            | TerminatorKind::Yield { .. }
            | TerminatorKind::Assert { .. }
            | TerminatorKind::DropAndReplace { .. }
            | TerminatorKind::GeneratorDrop
            | TerminatorKind::Resume
            | TerminatorKind::Abort
            | TerminatorKind::Return
            | TerminatorKind::Unreachable
            | TerminatorKind::FalseEdges { .. }
            | TerminatorKind::FalseUnwind { .. } => {
                // safe (at least as emitted during MIR construction)
            }

            TerminatorKind::Call { ref func, .. } => {
                let func_ty = func.ty(self.mir, self.cx.tcx);
                let sig = func_ty.fn_sig(self.cx.tcx);
                if let hir::Unsafety::Unsafe = sig.unsafety() {
                    let loc = terminator.source_info;
                    let kind = SourceKind::UnsafeFnCall(convert_abi(sig.abi()));
                    let unsafety_node_id = self.get_unsafety_node_id();
                    self.data.add_unsafety_source(self.cx, kind, loc, unsafety_node_id);
                }
            }
        }
        self.super_terminator(block, terminator, location);
    }

    fn visit_statement(
        &mut self,
        block: BasicBlock,
        statement: &Statement<'tcx>,
        location: Location,
    ) {
        self.source_info = statement.source_info;
        match statement.kind {
            StatementKind::InlineAsm { .. } => {
                let unsafety_node_id = self.get_unsafety_node_id();
                self.data.add_unsafety_source(
                    self.cx,
                    SourceKind::Asm,
                    statement.source_info,
                    unsafety_node_id,
                );
            }
            _ => {
                // safe (at least as emitted during MIR construction)
            }
        }
        self.super_statement(block, statement, location);
    }

    fn visit_rvalue(&mut self, rvalue: &Rvalue<'tcx>, location: Location) {
        if let &Rvalue::Aggregate(box ref aggregate, _) = rvalue {
            match aggregate {
                &AggregateKind::Array(..) | &AggregateKind::Tuple | &AggregateKind::Adt(..) => {}
                &AggregateKind::Closure(def_id, _) | &AggregateKind::Generator(def_id, _, _) => {
                    // TODO add tests for this
                    //TODO check why Rust unsafe analysis is on mir_built
                    let mir = &mut self.cx.tcx.optimized_mir(def_id);
                    let mut body_visitor = UnsafetySourcesVisitor {
                        cx: self.cx,
                        fn_node_id: self.fn_node_id,
                        mir,
                        data: self.data,
                        param_env: self.cx.tcx.param_env(def_id),
                        source_info: self.source_info,
                        source_scope_local_data: self.source_scope_local_data,
                    };
                    body_visitor.visit_mir(mir);
                }
            }
        }
        self.super_rvalue(rvalue, location);
    }

    fn visit_place(
        &mut self,
        place: &Place<'tcx>,
        context: PlaceContext<'tcx>,
        location: Location,
    ) {
        if let PlaceContext::Borrow { .. } = context {
            if rustc_mir::util::is_disaligned(self.cx.tcx, self.mir, self.param_env, place) {
                let unsafety_node_id = self.get_unsafety_node_id();
                self.data.add_unsafety_source(
                    self.cx,
                    SourceKind::BorrowPacked,
                    self.source_info,
                    unsafety_node_id,
                );
            }
        }

        match place {
            &Place::Projection(box Projection { ref base, ref elem }) => {
                let old_source_info = self.source_info;
                if let &Place::Local(local) = base {
                    if self.mir.local_decls[local].internal {
                        // Internal locals are used in the `move_val_init` desugaring.
                        // We want to check unsafety against the source info of the
                        // desugaring, rather than the source info of the RHS.
                        self.source_info = self.mir.local_decls[local].source_info;
                    }
                }
                let base_ty = base.ty(self.mir, self.cx.tcx).to_ty(self.cx.tcx);
                match base_ty.sty {
                    ty::TyKind::RawPtr(..) => {
                        let unsafety_node_id = self.get_unsafety_node_id();
                        self.data.add_unsafety_source(
                            self.cx,
                            SourceKind::DerefRawPointer,
                            self.source_info,
                            unsafety_node_id,
                        );
                    }
                    ty::TyKind::Adt(adt, _) => {
                        if adt.is_union() {
                            if context == PlaceContext::Store
                                || context == PlaceContext::AsmOutput
                                || context == PlaceContext::Drop
                                {
                                    let elem_ty = match elem {
                                        &ProjectionElem::Field(_, ty) => ty,
                                        _ => span_bug!(
                                        self.source_info.span,
                                        "non-field projection {:?} from union?",
                                        place
                                    ),
                                    };
                                    if elem_ty.moves_by_default(
                                        self.cx.tcx,
                                        self.param_env,
                                        self.source_info.span,
                                    ) {
                                        let unsafety_node_id = self.get_unsafety_node_id();
                                        let kind = SourceKind::AssignmentToNonCopyUnionField;
                                        self.data.add_unsafety_source(
                                            self.cx,
                                            kind,
                                            self.source_info,
                                            unsafety_node_id,
                                        );
                                    } else {
                                        // write to non-move union, safe
                                    }
                                } else {
                                let unsafety_node_id = self.get_unsafety_node_id();
                                self.data.add_unsafety_source(
                                    self.cx,
                                    SourceKind::AccessToUnionField,
                                    self.source_info,
                                    unsafety_node_id,
                                );
                            }
                        }
                    }
                    _ => {}
                }
                self.source_info = old_source_info;
            }
            &Place::Local(..) => {
                // locals are safe
            }
            &Place::Promoted(ref _p) => {
                //TODO find out what this is
            }
            &Place::Static(box Static { def_id, ty: _ }) => {
                if self.cx.tcx.is_static(def_id) == Some(hir::Mutability::MutMutable) {
                    let unsafety_node_id = self.get_unsafety_node_id();
                    self.data.add_unsafety_source(
                        self.cx,
                        SourceKind::Static,
                        self.source_info,
                        unsafety_node_id,
                    );
                } else if self.cx.tcx.is_foreign_item(def_id) {
                    let unsafety_node_id = self.get_unsafety_node_id();
                    self.data.add_unsafety_source(
                        self.cx,
                        SourceKind::ExternStatic,
                        self.source_info,
                        unsafety_node_id,
                    );
                }
            }
        };
        self.super_place(place, context, location);
    }
}

