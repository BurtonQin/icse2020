use rustc::hir;
use rustc::lint::LateContext;
use rustc::mir::visit::{PlaceContext, Visitor};
use rustc::mir::{
    AggregateKind, BasicBlock, ClearCrossCrate, Location, Mir, Operand, Place, Projection,
    ProjectionElem, Rvalue, Safety, SourceInfo, SourceScope, SourceScopeLocalData, Statement,
    StatementKind, Static, Terminator, TerminatorKind, OUTERMOST_SOURCE_SCOPE,
};
use rustc::ty;
use rustc_data_structures::indexed_vec::IndexVec;
use rustc_mir;
use rustc_target;
use syntax::ast::NodeId;

use analysis::Analysis;
use fn_info::FnInfo;
use results::blocks::BlockUnsafetyAnalysisSources;
use results::functions::Argument;
use results::functions::ArgumentKind;
use results::functions::UnsafeFnUsafetySources;
use results::unsafety_sources::FnCallInfo;
use results::unsafety_sources::{Source, SourceKind};
use util;

//////////////////////////////////////////////////////////////////////
// Unsafe Functions Analysis
//////////////////////////////////////////////////////////////////////
pub fn collect_no_reason<'a, 'tcx, 'b, 'c>(
    cx: &LateContext<'a, 'tcx>,
    data: &'c Vec<(&'b FnInfo, UnsafeFnUsafetySources)>,
) -> Vec<(&'b FnInfo, results::functions::ShortFnInfo)> {
    let mut res = Vec::new();
    for (fn_info, sources) in data.iter() {
        if sources.arguments().is_empty() && sources.sources().is_empty() && !sources.from_trait() {
            res.push((*fn_info, fn_info.build_short_fn_info(cx)));
        }
    }
    res
}

fn process_fn_decl<'a, 'tcx>(
    cx: &LateContext<'a, 'tcx>,
    decl_id: NodeId,
) -> UnsafeFnUsafetySources {
    let from_trait = util::is_unsafe_method(decl_id, cx);
    let mut res = UnsafeFnUsafetySources::new(cx.tcx.node_path_str(decl_id), from_trait);
    if let Some(fn_decl) = cx.tcx.hir.fn_decl(decl_id) {
        for input in fn_decl.inputs {
            if let Some(reason) = process_type(cx, &input) {
                //TODO record some information about the argument
                res.add_argument(reason);
            }
        }
    } else {
        error!("Decl NOT found {:?}", decl_id);
    }
    res
}

// returns true is a raw ptr is somewhere in the type
fn process_type<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, ty: &hir::Ty) -> Option<Argument> {
    match ty.node {
        hir::TyKind::Slice(ref sty) | hir::TyKind::Array(ref sty, _) => process_type(cx, &sty),

        hir::TyKind::Ptr(_) => Some(Argument::new(
            util::get_node_name(cx, ty.id),
            ArgumentKind::RawPointer,
        )),

        hir::TyKind::Rptr(_, _) => None, //I think this is a Rust reference

        hir::TyKind::BareFn(ref bare_fn) => {
            if let hir::Unsafety::Unsafe = bare_fn.unsafety {
                Some(Argument::new(
                    util::get_node_name(cx, ty.id),
                    ArgumentKind::UnsafeFunction,
                ))
            } else {
                process_ty_array(cx, &bare_fn.decl.inputs)
            }
        }

        hir::TyKind::Tup(ref vty) => process_ty_array(cx, &vty),

        hir::TyKind::Path(ref qpath) => match qpath {
            hir::QPath::Resolved(oty, _) => {
                if let Some(sty) = oty {
                    process_type(cx, sty)
                } else {
                    None
                }
            }
            hir::QPath::TypeRelative(pty, _) => process_type(cx, pty),
        },

        hir::TyKind::TraitObject(ref _poly_ref, _) => None, //TODO

        hir::TyKind::Never | hir::TyKind::Typeof(_) | hir::TyKind::Infer | hir::TyKind::Err => None,
    }
}

fn process_ty_array<'a, 'tcx>(
    cx: &LateContext<'a, 'tcx>,
    array: &hir::HirVec<hir::Ty>,
) -> Option<Argument> {
    let mut iter = array.iter();
    let mut done = false;
    let mut res = None;
    while !done {
        if let Some(elt) = iter.next() {
            let arg_res = process_type(cx, elt);
            if let Some(_) = arg_res {
                res = arg_res;
                done = true;
            }
        } else {
            done = true;
        }
    }
    res
}

impl UnsafetySourceCollector for UnsafeFnUsafetySources {
    fn add_unsafety_source<'a, 'tcx>(
        &mut self,
        cx: &LateContext<'a, 'tcx>,
        kind: SourceKind,
        source_info: SourceInfo,
        _node_id: NodeId,
    ) {
        let source = Source {
            kind,
            loc: util::get_file_and_line(cx, source_info.span),
        };
        self.add_source(source);
    }
}

impl Analysis for UnsafeFnUsafetySources {
    fn is_set(&self) -> bool {
        false
    }

    fn set(&mut self) {}

    fn run_analysis<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, fn_info: &'a FnInfo) -> Self {
        let fn_def_id = cx.tcx.hir.local_def_id(fn_info.decl_id());
        let mut res = process_fn_decl(cx, fn_info.decl_id());
        {
            //needed for the borrow checker
            let mir = &mut cx.tcx.optimized_mir(fn_def_id);
            if let Some(mut body_visitor) =
                UnsafetySourcesVisitor::new(cx, mir, &mut res, fn_def_id)
            {
                body_visitor.visit_mir(mir);
            }
        }
        res
    }
}

//////////////////////////////////////////////////////////////////////
// Unsafe Blocks Analysis
//////////////////////////////////////////////////////////////////////

impl UnsafetySourceCollector for BlockUnsafetyAnalysisSources {
    fn add_unsafety_source<'a, 'tcx>(
        &mut self,
        cx: &LateContext<'a, 'tcx>,
        kind: SourceKind,
        source_info: SourceInfo,
        block_id: NodeId,
    ) {
        let source = Source {
            kind,
            loc: util::get_file_and_line(cx, source_info.span),
        };
        self.add_source(block_id.to_string(), source)
    }
}

impl Analysis for BlockUnsafetyAnalysisSources {
    fn is_set(&self) -> bool {
        false
    }

    fn set(&mut self) {}

    fn run_analysis<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, fn_info: &'a FnInfo) -> Self {
        let mut analysis: Self = Self::new();
        let fn_def_id = cx.tcx.hir.local_def_id(fn_info.decl_id());
        // closures are handled by their parent fn.
        if !cx.tcx.is_closure(fn_def_id) {
            let mir = &mut cx.tcx.optimized_mir(fn_def_id);
            if let Some(mut body_visitor) =
                UnsafetySourcesVisitor::new(cx, mir, &mut analysis, fn_def_id)
            {
                body_visitor.visit_mir(mir);
            }
        }
        analysis
    }
}

//////////////////////////////////////////////////////////////////////
// Common Parts
//////////////////////////////////////////////////////////////////////

trait UnsafetySourceCollector {
    fn add_unsafety_source<'a, 'tcx>(
        &mut self,
        cx: &LateContext<'a, 'tcx>,
        kind: SourceKind,
        source_info: SourceInfo,
        node_id: NodeId,
    );
}

struct UnsafetySourcesVisitor<'a, 'tcx: 'a> {
    cx: &'a LateContext<'a, 'tcx>,
    fn_node_id: NodeId,
    mir: &'a Mir<'tcx>,
    data: &'a mut UnsafetySourceCollector,
    param_env: ty::ParamEnv<'tcx>,
    source_info: SourceInfo,
    source_scope_local_data: &'a IndexVec<SourceScope, SourceScopeLocalData>,
}

impl<'a, 'tcx> UnsafetySourcesVisitor<'a, 'tcx> {
    fn new(
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

pub fn find_callee<'a, 'tcx>(
    cx: &LateContext<'a, 'tcx>,
    func: &Operand<'tcx>,
) -> Option<FnCallInfo> {
    if let Operand::Constant(constant) = func {
        if let ty::TyKind::FnDef(callee_def_id, _) = constant.literal.ty.sty {
            let abi = cx.tcx.fn_sig(callee_def_id).abi();
            if callee_def_id.is_local() {
                if let Some(callee_node_id) = cx.tcx.hir.as_local_node_id(callee_def_id) {
                    Some(FnCallInfo::Local(
                        callee_node_id.to_string(),
                        convert_abi(abi),
                    ))
                } else {
                    error!("local node id NOT found {:?}", callee_def_id);
                    None
                }
            } else {
                let mut output = std::format!("{}", constant.literal.ty.sty);
                Some(FnCallInfo::External(
                    cx.tcx.crate_name(callee_def_id.krate).to_string(),
                    output,
                    convert_abi(abi),
                ))
            }
        } else {
            error!("TypeVariants NOT handled {:?}", constant.literal.ty.sty);
            None
        }
    } else {
        error!("find_callee::Operand Type NOT handled {:?}", func);
        None
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
                    if let Some(call_info) = find_callee(self.cx, func) {
                        let kind = SourceKind::UnsafeFnCall(call_info);
                        let unsafety_node_id = self.get_unsafety_node_id();
                        self.data
                            .add_unsafety_source(self.cx, kind, loc, unsafety_node_id);
                    } else {
                        error!("find_callee NOT found {:?}", func);
                    }
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
            StatementKind::Assign(..)
            | StatementKind::ReadForMatch(..)
            | StatementKind::SetDiscriminant { .. }
            | StatementKind::StorageLive(..)
            | StatementKind::StorageDead(..)
            | StatementKind::EndRegion(..)
            | StatementKind::Validate(..)
            | StatementKind::UserAssertTy(..)
            | StatementKind::Nop => {
                // safe (at least as emitted during MIR construction)
            }

            StatementKind::InlineAsm { .. } => {
                let unsafety_node_id = self.get_unsafety_node_id();
                self.data.add_unsafety_source(
                    self.cx,
                    SourceKind::Asm,
                    statement.source_info,
                    unsafety_node_id,
                );
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
                        let mut output = std::format!("{}", base_ty.sty);
                        let unsafety_node_id = self.get_unsafety_node_id();
                        self.data.add_unsafety_source(
                            self.cx,
                            SourceKind::DerefRawPointer(output),
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
                                    let kind = SourceKind::AssignmentToNonCopyUnionField(
                                        util::get_def_id_string(self.cx, adt.did),
                                    );
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
                                    SourceKind::AccessToUnionField(util::get_def_id_string(
                                        self.cx, adt.did,
                                    )),
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
                        SourceKind::Static(util::get_def_id_string(self.cx, def_id)),
                        self.source_info,
                        unsafety_node_id,
                    );
                } else if self.cx.tcx.is_foreign_item(def_id) {
                    let unsafety_node_id = self.get_unsafety_node_id();
                    self.data.add_unsafety_source(
                        self.cx,
                        SourceKind::ExternStatic(util::get_def_id_string(self.cx, def_id)),
                        self.source_info,
                        unsafety_node_id,
                    );
                }
            }
        };
        self.super_place(place, context, location);
    }
}

fn convert_abi(abi: rustc_target::spec::abi::Abi) -> results::unsafety_sources::Abi {
    match abi {
        rustc_target::spec::abi::Abi::Cdecl => results::unsafety_sources::Abi::Cdecl,
        rustc_target::spec::abi::Abi::Stdcall => results::unsafety_sources::Abi::Stdcall,
        rustc_target::spec::abi::Abi::Fastcall => results::unsafety_sources::Abi::Fastcall,
        rustc_target::spec::abi::Abi::Vectorcall => results::unsafety_sources::Abi::Vectorcall,
        rustc_target::spec::abi::Abi::Thiscall => results::unsafety_sources::Abi::Thiscall,
        rustc_target::spec::abi::Abi::SysV64 => results::unsafety_sources::Abi::SysV64,
        rustc_target::spec::abi::Abi::PtxKernel => results::unsafety_sources::Abi::PtxKernel,
        rustc_target::spec::abi::Abi::Msp430Interrupt => {
            results::unsafety_sources::Abi::Msp430Interrupt
        }
        rustc_target::spec::abi::Abi::X86Interrupt => results::unsafety_sources::Abi::X86Interrupt,
        rustc_target::spec::abi::Abi::AmdGpuKernel => results::unsafety_sources::Abi::AmdGpuKernel,
        rustc_target::spec::abi::Abi::Rust => results::unsafety_sources::Abi::Rust,
        rustc_target::spec::abi::Abi::C => results::unsafety_sources::Abi::C,
        rustc_target::spec::abi::Abi::System => results::unsafety_sources::Abi::System,
        rustc_target::spec::abi::Abi::RustIntrinsic => {
            results::unsafety_sources::Abi::RustIntrinsic
        }
        rustc_target::spec::abi::Abi::RustCall => results::unsafety_sources::Abi::RustCall,
        rustc_target::spec::abi::Abi::PlatformIntrinsic => {
            results::unsafety_sources::Abi::PlatformIntrinsic
        }
        rustc_target::spec::abi::Abi::Unadjusted => results::unsafety_sources::Abi::Unadjusted,
        rustc_target::spec::abi::Abi::Aapcs => results::unsafety_sources::Abi::Aapcs,
        rustc_target::spec::abi::Abi::Win64 => results::unsafety_sources::Abi::Win64,
    }
}
