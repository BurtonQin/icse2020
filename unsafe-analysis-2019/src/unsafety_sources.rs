use core::ops::Bound;

use results::unsafety_sources::SourceKind;

use convert_abi;
use rustc::hir;
use rustc::hir::HirId;
use rustc::lint::LateContext;
use rustc::mir::{
    AggregateKind, BasicBlock, Body, ClearCrossCrate, Location, OUTERMOST_SOURCE_SCOPE, Place, PlaceBase,
    Projection, ProjectionElem, Rvalue, Safety, SourceInfo, SourceScope, SourceScopeLocalData,
    Statement, StatementKind, Static, StaticKind, Terminator, TerminatorKind,
};
use rustc::mir::visit::{MutatingUseContext, PlaceContext, Visitor};
use rustc::ty;
use rustc_data_structures::indexed_vec::IndexVec;
use rustc_mir;
use rustc_target;
use syntax::ast::NodeId;
use syntax::symbol::InternedString;

pub trait UnsafetySourceCollector {
    fn add_unsafety_source<'a, 'tcx>(
        &mut self,
        cx: &LateContext<'a, 'tcx>,
        kind: SourceKind,
        source_info: SourceInfo,
        node_id: NodeId,
        user_provided: bool,
    );
}

pub struct UnsafetySourcesVisitor<'a, 'tcx: 'a> {
    cx: &'a LateContext<'a, 'tcx>,
    fn_node_id: HirId,
    body: &'a Body<'tcx>,
    data: &'a mut UnsafetySourceCollector,
    param_env: ty::ParamEnv<'tcx>,
    source_info: SourceInfo,
    source_scope_local_data: &'a IndexVec<SourceScope, SourceScopeLocalData>,
}

impl<'a, 'tcx> UnsafetySourcesVisitor<'a, 'tcx> {
    pub fn new(
        cx: &'a LateContext<'a, 'tcx>,
        body: &'a Body<'tcx>,
        data: &'a mut UnsafetySourceCollector,
        fn_def_id: hir::def_id::DefId,
    ) -> Option<Self> {
        match body.source_scope_local_data {
            ClearCrossCrate::Set(ref local_data) => Some(UnsafetySourcesVisitor {
                cx,
                fn_node_id: cx.tcx.hir().def_index_to_hir_id(fn_def_id.index),
                body,
                data,
                param_env: cx.tcx.param_env(fn_def_id),
                source_info: SourceInfo {
                    span: body.span,
                    scope: OUTERMOST_SOURCE_SCOPE,
                },
                source_scope_local_data: local_data,
            }),
            ClearCrossCrate::Clear => {
                info!("unsafety_violations: {:?} - remote, skipping", fn_def_id);
                None
            }
        }
    }

    fn is_unsafety_user_provided(&self, source_info: SourceInfo) -> bool {
        match self.source_scope_local_data[source_info.scope].safety {
            Safety::Safe => {
                //assert!(false, "loc = {:?}", source_info);
                // it appears on alloc::alloc::box_free
                false
            },
            Safety::BuiltinUnsafe => { false } // TODO check this
            Safety::FnUnsafe => { true },
            Safety::ExplicitUnsafe(node_id) => {
                let node = self.cx.tcx.hir().get(node_id);
                if let hir::Node::Block(block) = node {
                    match block.rules {
                        hir::BlockCheckMode::DefaultBlock => {
                            false
                        }
                        hir::BlockCheckMode::UnsafeBlock(unsafe_source) |
                        hir::BlockCheckMode::PushUnsafeBlock(unsafe_source) |
                        hir::BlockCheckMode::PopUnsafeBlock(unsafe_source) => {
                            match unsafe_source {
                                hir::UnsafeSource::UserProvided => {
                                    true
                                }
                                hir::UnsafeSource::CompilerGenerated => {
                                    false
                                }
                            }
                        }
                    }
                } else {
                    assert!(false); false
                }
            },
        }
    }

    fn get_unsafety_node_id(&self, source_info: SourceInfo) -> Option<HirId> {
        match self.source_scope_local_data[source_info.scope].safety {
            Safety::Safe => {
                //assert!(false);
                None
            },
            Safety::BuiltinUnsafe => { Some(self.fn_node_id) }
            Safety::FnUnsafe => { Some(self.fn_node_id) },
            Safety::ExplicitUnsafe(hir_id) => { Some(hir_id) }
        }
    }

    fn add_unsafety_source(
        &mut self,
        kind: SourceKind,
        source_info: SourceInfo,
    ) {
        let user_provided = self.is_unsafety_user_provided(source_info);
        if let Some (unsafety_node_id) = self.get_unsafety_node_id(source_info) {
            self.data.add_unsafety_source(
                self.cx,
                kind,
                source_info,
                self.cx.tcx.hir().hir_to_node_id(unsafety_node_id),
                user_provided
            );
        }
    }

    fn check_mut_borrowing_layout_constrained_field(
        &mut self,
        mut place: &Place<'tcx>,
        is_mut_use: bool,
    ) {
        while let &Place::Projection(box Projection {
            ref base, ref elem
        }) = place {
            match *elem {
                ProjectionElem::Field(..) => {
                    let ty = base.ty(&self.body.local_decls, self.cx.tcx).ty;
                    match ty.sty {
                        ty::Adt(def, _) => match self.cx.tcx.layout_scalar_valid_range(def.did) {
                            (Bound::Unbounded, Bound::Unbounded) => {}
                            _ => {
                                let source_info = self.source_info;
                                self.add_unsafety_source(
                                    SourceKind::ConstantFn,
                                    source_info,
                                );
                            }
                        },
                        _ => {}
                    }
                }
                _ => {}
            }
            place = base;
        }
    }
}


impl<'a, 'tcx> Visitor<'tcx> for UnsafetySourcesVisitor<'a, 'tcx> {
    fn visit_terminator(
        &mut self,
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
                let func_ty = func.ty(self.body, self.cx.tcx);
                let sig = func_ty.fn_sig(self.cx.tcx);
                if let hir::Unsafety::Unsafe = sig.unsafety() {
                    let loc = terminator.source_info;
                    let kind = SourceKind::UnsafeFnCall(convert_abi(sig.abi()));
                    self.add_unsafety_source(kind, loc);
                }
            }
        }
        self.super_terminator(terminator, location);
    }

    fn visit_statement(
        &mut self,
        statement: &Statement<'tcx>,
        location: Location,
    ) {
        self.source_info = statement.source_info;
        match statement.kind {
            StatementKind::InlineAsm { .. } => {
                self.add_unsafety_source(
                        SourceKind::Asm,
                        statement.source_info,
                    );
            }
            _ => {
                // safe (at least as emitted during MIR construction)
            }
        }
        self.super_statement(statement, location);
    }

    fn visit_rvalue(&mut self, rvalue: &Rvalue<'tcx>, location: Location) {
        if let &Rvalue::Aggregate(box ref aggregate, _) = rvalue {
            match aggregate {
                &AggregateKind::Array(..) | &AggregateKind::Tuple | &AggregateKind::Adt(..) => {}
                &AggregateKind::Closure(def_id, _) | &AggregateKind::Generator(def_id, _, _) => {
                    // TODO add tests for this
                    //TODO check why Rust unsafe analysis is on mir_built
                    let body = &mut self.cx.tcx.optimized_mir(def_id);
                    let mut body_visitor = UnsafetySourcesVisitor {
                        cx: self.cx,
                        fn_node_id: self.fn_node_id,
                        body,
                        data: self.data,
                        param_env: self.cx.tcx.param_env(def_id),
                        source_info: self.source_info,
                        source_scope_local_data: self.source_scope_local_data,
                    };
                    body_visitor.visit_body(body);
                }
            }
        }
        self.super_rvalue(rvalue, location);
    }

    fn visit_place(
        &mut self,
        place: &Place<'tcx>,
        context: PlaceContext,
        location: Location,
    ) {

        place.iterate(|place_base, place_projections| {
            match place_base {
                PlaceBase::Local(..) => {
                    // Locals are safe.
                }
                PlaceBase::Static(box Static { kind: StaticKind::Promoted(_), .. }) => {
                    //bug!("unsafety checking should happen before promotion")
                }
                PlaceBase::Static(box Static { kind: StaticKind::Static(def_id), .. }) => {
                    if self.cx.tcx.is_mutable_static(*def_id) {
                        self.add_unsafety_source(
                            SourceKind::Static,
                            self.source_info,
                        );
                    } else if self.cx.tcx.is_foreign_item(*def_id) {
                        let source_info = self.source_info;
                        self.add_unsafety_source(
                            SourceKind::ExternStatic,
                            self.source_info,
                        );
                    }
                }
            }

            for proj in place_projections {
                if context.is_borrow() {
                    if rustc_mir::util::is_disaligned(self.cx.tcx, self.body, self.param_env, place) {
                        let source_info = self.source_info;
                        self.add_unsafety_source(
                            SourceKind::BorrowPacked,
                            source_info,
                        );
                    }
                }
                let is_borrow_of_interior_mut = context.is_borrow() && !proj.base
                    .ty(self.body, self.cx.tcx)
                    .ty
                    .is_freeze(self.cx.tcx, self.param_env, self.source_info.span);
                // prevent
                // * `&mut x.field`
                // * `x.field = y;`
                // * `&x.field` if `field`'s type has interior mutability
                // because either of these would allow modifying the layout constrained field and
                // insert values that violate the layout constraints.
                if context.is_mutating_use() || is_borrow_of_interior_mut {
                    self.check_mut_borrowing_layout_constrained_field(
                        place, context.is_mutating_use(),
                    );
                }
                let old_source_info = self.source_info;
                if let Place::Base(PlaceBase::Local(local)) = proj.base {
                    if self.body.local_decls[local].internal {
                        // Internal locals are used in the `move_val_init` desugaring.
                        // We want to check unsafety against the source info of the
                        // desugaring, rather than the source info of the RHS.
                        self.source_info = self.body.local_decls[local].source_info;
                    }
                }
                let base_ty = proj.base.ty(self.body, self.cx.tcx).ty;
                match base_ty.sty {
                    ty::RawPtr(..) => {
                        self.add_unsafety_source(
                            SourceKind::DerefRawPointer,
                            self.source_info,
                        );
                    }
                    ty::Adt(adt, _) => {
                        if adt.is_union() {
                            if context == PlaceContext::MutatingUse(MutatingUseContext::Store) ||
                                context == PlaceContext::MutatingUse(MutatingUseContext::Drop) ||
                                context == PlaceContext::MutatingUse(
                                    MutatingUseContext::AsmOutput
                                )
                            {
                                let elem_ty = match proj.elem {
                                    ProjectionElem::Field(_, ty) => ty,
                                    _ => span_bug!(
                                        self.source_info.span,
                                        "non-field projection {:?} from union?",
                                        place)
                                };
                                if !elem_ty.is_copy_modulo_regions(
                                    self.cx.tcx,
                                    self.param_env,
                                    self.source_info.span,
                                ) {
                                    self.add_unsafety_source(
                                        SourceKind::AssignmentToNonCopyUnionField,
                                        self.source_info,
                                    );
                                } else {
                                    // write to non-move union, safe
                                }
                            } else {
                                self.add_unsafety_source(
                                    SourceKind::AccessToUnionField,
                                    self.source_info,
                                );
                            }
                        }
                    }
                    _ => {}
                }
                self.source_info = old_source_info;
            }
        });
        self.super_place(place, context, location);
    }
}

