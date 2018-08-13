use rustc::hir;
use rustc::lint::LateContext;
use rustc::mir::visit::{PlaceContext, Visitor};
use rustc::mir::{
    AggregateKind, BasicBlock, Location, Mir, Place, Projection, ProjectionElem, Rvalue,
    SourceInfo, Statement, StatementKind, Static, Terminator, TerminatorKind,
    OUTERMOST_SOURCE_SCOPE,
};
use rustc::ty;
use rustc_mir;
use syntax::ast::NodeId;

use analysis::Analysis;
use fn_info::FnInfo;
use print::Print;
use unsafety::{Source, SourceKind};
use util;


//////////////////////////////////////////////////////////////////////
// Unsafe Functions Analysis
//////////////////////////////////////////////////////////////////////

// information about reasons for unsafety of functions declared unsafe
pub struct UnsafeFnUsafetyAnalysis {
    decl_id: NodeId,
    from_trait: bool,
    arguments: Vec<Argument>,
    sources: Vec<Source>,
}

pub struct Argument {
    ty_node_id: NodeId,
    kind: ArgumentKind,
}

pub enum ArgumentKind {
    RawPointer,
    UnsafeFunction,
}

impl UnsafeFnUsafetyAnalysis {
    fn new(decl_id: NodeId) -> Self {
        UnsafeFnUsafetyAnalysis {
            decl_id,
            from_trait: false,
            arguments: Vec::new(),
            sources: Vec::new(),
        }
    }

    fn process_fn_decl<'a, 'tcx>(&mut self, cx: &LateContext<'a, 'tcx>) -> () {
        self.from_trait = util::is_unsafe_method(self.decl_id,cx);
        if let Some(fn_decl) = cx.tcx.hir.fn_decl(self.decl_id) {
            for input in fn_decl.inputs {
                if let Some(reason) = UnsafeFnUsafetyAnalysis::process_type(&input) {
                    //TODO record some information about the argument
                    self.arguments.push(reason);
                }
            }
        } else {
            println!("Decl NOT found {:?}", self.decl_id);
        }
    }

    // returns true is a raw ptr is somewhere in the type
    fn process_type(ty: &hir::Ty) -> Option<Argument> {
        match ty.node {
            hir::TyKind::Slice(ref sty) | hir::TyKind::Array(ref sty, _) => {
                UnsafeFnUsafetyAnalysis::process_type(&sty)
            }

            hir::TyKind::Ptr(_) => Some(Argument {
                ty_node_id: ty.id,
                kind: ArgumentKind::RawPointer,
            }),

            hir::TyKind::Rptr(_, _) => None, //I think this is a Rust reference

            hir::TyKind::BareFn(ref bare_fn) => {
                if let hir::Unsafety::Unsafe = bare_fn.unsafety {
                    Some(Argument {
                        ty_node_id: ty.id,
                        kind: ArgumentKind::UnsafeFunction,
                    })
                } else {
                    UnsafeFnUsafetyAnalysis::process_ty_array(&bare_fn.decl.inputs)
                }
            }

            hir::TyKind::Tup(ref vty) => UnsafeFnUsafetyAnalysis::process_ty_array(&vty),

            hir::TyKind::Path(ref qpath) => match qpath {
                hir::QPath::Resolved(oty, _) => {
                    if let Some(sty) = oty {
                        UnsafeFnUsafetyAnalysis::process_type(sty)
                    } else {
                        None
                    }
                }
                hir::QPath::TypeRelative(pty, _) => UnsafeFnUsafetyAnalysis::process_type(pty),
            },

            hir::TyKind::TraitObject(ref poly_ref, _) => None, //TODO

            hir::TyKind::Never | hir::TyKind::Typeof(_) | hir::TyKind::Infer | hir::TyKind::Err => {
                None
            }
        }
    }

    fn process_ty_array(array: &hir::HirVec<hir::Ty>) -> Option<Argument> {
        let mut iter = array.iter();
        let mut done = false;
        let mut res = None;
        while !done {
            if let Some(elt) = iter.next() {
                let arg_res = UnsafeFnUsafetyAnalysis::process_type(elt);
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
}

impl UnsafetySourceCollector for UnsafeFnUsafetyAnalysis {
    fn add_source(&mut self, source: Source) {
        self.sources.push(source);
    }
}

impl Analysis for UnsafeFnUsafetyAnalysis {
    fn is_set(&self) -> bool {
        false
    }

    fn set(&mut self) {}

    fn run_analysis<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, fn_info: &'a FnInfo) -> Self {
        let tcx = cx.tcx;
        let mut analysis: Self = Self::new(fn_info.decl_id());
        let fn_def_id = tcx.hir.local_def_id(fn_info.decl_id());
        analysis.process_fn_decl(cx);
        {
            //needed for the borrow checker
            let mir = &mut tcx.mir_validated(fn_def_id).borrow();
            let mut body_visitor = UnsafetySourcesVisitor {
                cx,
                mir,
                data: &mut analysis,
                param_env: tcx.param_env(fn_def_id),
                source_info: SourceInfo {
                    span: mir.span,
                    scope: OUTERMOST_SOURCE_SCOPE,
                },
            };
            body_visitor.visit_mir(mir);
        }
        analysis
    }
}

impl Print for UnsafeFnUsafetyAnalysis {
    fn print<'a, 'tcx>(&self, cx: &LateContext<'a, 'tcx>) -> () {
        if self.from_trait {
            println!("\nUnsafe from signature in trait");
        }
        if !self.arguments.is_empty() {
            println!("Unsafety in arguments: ");
            for arg in &self.arguments {
                arg.print(cx);
            }
        }
        if !self.sources.is_empty() {
            println!("Unsafety in body: ");
            for source in &self.sources {
                source.print(cx);
            }
        }
    }
}

impl Print for ArgumentKind {
    fn print<'a, 'tcx>(&self, _cx: &LateContext<'a, 'tcx>) -> () {
        match self {
            ArgumentKind::RawPointer => {
                print!("RawPointer");
            }
            ArgumentKind::UnsafeFunction => {
                print!("UnsafeFunction");
            }
        }
    }
}

impl Print for Argument {
    fn print<'a, 'tcx>(&self, cx: &LateContext<'a, 'tcx>) -> () {
        print!("Kind ");
        self.kind.print(cx);
        println!(" Type {:?}", cx.tcx.node_path_str(self.ty_node_id));
    }
}

//////////////////////////////////////////////////////////////////////
// Unsafe Blocks Analysis
//////////////////////////////////////////////////////////////////////

pub struct UnsafeBlockUnsafetyAnalysis {
    enclosing_fn_node_id: NodeId,
    sources: Vec<Source>,
}


impl UnsafeBlockUnsafetyAnalysis {
    fn new(decl_id: NodeId) -> Self {
        UnsafeBlockUnsafetyAnalysis {
            enclosing_fn_node_id: decl_id,
            sources: Vec::new(),
        }
    }

}

impl Print for UnsafeBlockUnsafetyAnalysis {

    fn print<'a, 'tcx>(&self, cx: &LateContext<'a, 'tcx>) -> () {
        if !self.sources.is_empty() {
            println!("\nUnsafety in unsafe blocks: ");
            for source in &self.sources {
                source.print(cx);
            }
        }
    }

}

impl UnsafetySourceCollector for UnsafeBlockUnsafetyAnalysis {
    fn add_source(&mut self, source: Source) {
        self.sources.push(source);
    }
}

impl Analysis for UnsafeBlockUnsafetyAnalysis {
    fn is_set(&self) -> bool {
        false
    }

    fn set(&mut self) {}

    fn run_analysis<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, fn_info: &'a FnInfo) -> Self {
        let tcx = cx.tcx;
        let mut analysis: Self = Self::new(fn_info.decl_id());
        let fn_def_id = tcx.hir.local_def_id(fn_info.decl_id());
        // closures are handled by their parent fn.
        if !cx.tcx.is_closure(fn_def_id) {
            let mir = &mut tcx.mir_validated(fn_def_id).borrow();
            let mut body_visitor = UnsafetySourcesVisitor {
                cx,
                mir,
                data: &mut analysis,
                param_env: tcx.param_env(fn_def_id),
                source_info: SourceInfo {
                    span: mir.span,
                    scope: OUTERMOST_SOURCE_SCOPE,
                },
            };
            body_visitor.visit_mir(mir);
        }
        analysis
    }
}

//////////////////////////////////////////////////////////////////////
// Common Parts
//////////////////////////////////////////////////////////////////////

trait UnsafetySourceCollector {
    fn add_source( &mut self, Source );
}

struct UnsafetySourcesVisitor<'a, 'tcx: 'a> {
    cx: &'a LateContext<'a, 'tcx>,
    mir: &'a Mir<'tcx>,
    data: &'a mut UnsafetySourceCollector,
    param_env: ty::ParamEnv<'tcx>,
    source_info: SourceInfo,
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
                    if let Some(unsafe_fn_call) = Source::new_unsafe_fn_call(self.cx, func, loc) {
                        self.data.add_source(unsafe_fn_call);
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
                self.data.add_source(Source {
                    kind: SourceKind::Asm,
                    loc: statement.source_info,
                });
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
                    let mir = &mut self.cx.tcx.mir_built(def_id).borrow();
                    let mut body_visitor = UnsafetySourcesVisitor {
                        cx: self.cx,
                        mir,
                        data: self.data,
                        param_env: self.cx.tcx.param_env(def_id),
                        source_info: self.source_info,
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
                self.data.add_source(Source {
                    kind: SourceKind::BorrowPacked,
                    loc: self.source_info,
                });
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
                    ty::TyRawPtr(..) => {
                        let mut output = std::format!("{}", base_ty.sty);
                        self.data.add_source(Source {
                            kind: SourceKind::DerefRawPointer(output),
                            loc: self.source_info,
                        });
                    }
                    ty::TyAdt(adt, _) => {
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
                                    self.data.add_source(Source {
                                        kind: SourceKind::AssignmentToNonCopyUnionField(adt.did),
                                        loc: self.source_info,
                                    });
                                } else {
                                    // write to non-move union, safe
                                }
                            } else {
                                self.data.add_source(Source {
                                    kind: SourceKind::AccessToUnionField(adt.did),
                                    loc: self.source_info,
                                });
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
            &Place::Promoted(ref p) => {
                //TODO find out what this is
                //println!("Place::Promoted {:?}", p);
            }
            &Place::Static(box Static { def_id, ty: _ }) => {
                if self.cx.tcx.is_static(def_id) == Some(hir::Mutability::MutMutable) {
                    self.data.add_source(Source {
                        kind: SourceKind::MutableStatic(def_id),
                        loc: self.source_info,
                    });
                } else if self.cx.tcx.is_foreign_item(def_id) {
                    self.data.add_source(Source {
                        kind: SourceKind::UseExternStatic(def_id),
                        loc: self.source_info,
                    });
                }
            }
        };
        self.super_place(place, context, location);
    }
}

