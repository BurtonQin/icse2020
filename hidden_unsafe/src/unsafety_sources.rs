use rustc_data_structures::indexed_vec::IndexVec;
use rustc::hir;
use rustc::lint::LateContext;
use rustc::mir::visit::{PlaceContext, Visitor};
use rustc::mir::{
    AggregateKind, BasicBlock, Location, Mir, Place, Projection, ProjectionElem, Rvalue,
    SourceInfo, Statement, StatementKind, Static, Terminator, TerminatorKind,
    OUTERMOST_SOURCE_SCOPE, SourceScope, SourceScopeLocalData, ClearCrossCrate,
    Safety
};
use rustc::ty;
use rustc_mir;
use syntax::ast::NodeId;

use analysis::Analysis;
use fn_info::FnInfo;
use results::unsafety_sources::{Source, SourceKind};
use util;
use std::fs::File;
use std::io::Write;
use results::functions::UnsafeFnUsafetySources;
use results::functions::Argument;
use results::functions::ArgumentKind;
use results::blocks::BlockUnsafetyAnalysisSources;


//////////////////////////////////////////////////////////////////////
// Unsafe Functions Analysis
//////////////////////////////////////////////////////////////////////


fn process_fn_decl<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, decl_id: NodeId) -> UnsafeFnUsafetySources {
    let from_trait = util::is_unsafe_method(decl_id,cx);
    let res =
        UnsafeFnUsafetySources::new( cx.tcx.node_path_str(decl_id), from_trait);
    if let Some(fn_decl) = cx.tcx.hir.fn_decl(decl_id) {
        for input in fn_decl.inputs {
            if let Some(reason) = UnsafeFnUsafetySources::process_type(&input) {
                //TODO record some information about the argument
                res.add_argument(reason);
            }
        }
    } else {
        println!("Decl NOT found {:?}", decl_id);
    }
    res
}

// returns true is a raw ptr is somewhere in the type
fn process_type(ty: &hir::Ty) -> Option<Argument> {
    match ty.node {
        hir::TyKind::Slice(ref sty) | hir::TyKind::Array(ref sty, _) => {
            UnsafeFnUsafetySources::process_type(&sty)
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
                UnsafeFnUsafetySources::process_ty_array(&bare_fn.decl.inputs)
            }
        }

        hir::TyKind::Tup(ref vty) => UnsafeFnUsafetySources::process_ty_array(&vty),

        hir::TyKind::Path(ref qpath) => match qpath {
            hir::QPath::Resolved(oty, _) => {
                if let Some(sty) = oty {
                    UnsafeFnUsafetySources::process_type(sty)
                } else {
                    None
                }
            }
            hir::QPath::TypeRelative(pty, _) => UnsafeFnUsafetySources::process_type(pty),
        },

        hir::TyKind::TraitObject(ref _poly_ref, _) => None, //TODO

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
            let arg_res = UnsafeFnUsafetySources::process_type(elt);
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

impl Analysis for UnsafeFnUsafetySources {
    fn is_set(&self) -> bool {
        false
    }

    fn set(&mut self) {}

    fn run_analysis<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, fn_info: &'a FnInfo) -> Self {
        let fn_def_id = tcx.hir.local_def_id(fn_info.decl_id());
        let res = process_fn_decl(cx,fn_info.decl_id());
        res.process_fn_decl(cx);
        {
            //needed for the borrow checker
            let mir = &mut tcx.optimized_mir(fn_def_id);
            if let Some(mut body_visitor) = UnsafetySourcesVisitor::new(
                cx, mir,&mut analysis, fn_def_id
            ) {
                body_visitor.visit_mir(mir);
            }
        }
        res
    }
}

//impl Print for UnsafeFnUsafetyAnalysis {
//
//    fn print<'a, 'tcx>(&self, cx: &LateContext<'a, 'tcx>, file: &mut File) -> () {
//        if self.from_trait {
//            write!(file, "\nUnsafe from signature in trait");
//        }
//        if !self.arguments.is_empty() {
//            for arg in &self.arguments {
//                writeln!(file,"");
//                arg.print(cx, file);
//            }
//        }
//        if !self.sources.is_empty() {
//            //writeln!(file, "\nUnsafety in body: ");
//            for source in &self.sources {
//                writeln!(file,"");
//                source.print(cx, file);
//            }
//        }
//        writeln!(file, "");
//    }
//}
//
//impl Print for ArgumentKind {
//    fn print<'a, 'tcx>(&self, _cx: &LateContext<'a, 'tcx>, file: &mut File) -> () {
//        match self {
//            ArgumentKind::RawPointer => {
//                write!(file, "RawPointer");
//            }
//            ArgumentKind::UnsafeFunction => {
//                write!(file, "UnsafeFunction");
//            }
//        }
//    }
//}
//
//impl Print for Argument {
//    fn print<'a, 'tcx>(&self, cx: &LateContext<'a, 'tcx>, file: &mut File) -> () {
//        write!(file,"Unsafety in arguments kind: ");
//        self.kind.print(cx, file);
//        write!(file, " Type: {:#?}", cx.tcx.hir.get(self.ty_node_id));
//    }
//}

//////////////////////////////////////////////////////////////////////
// Unsafe Blocks Analysis
//////////////////////////////////////////////////////////////////////

//impl Print for UnsafeBlockUnsafetyAnalysis {
//
//    fn empty(&self) -> bool {
//        self.sources.is_empty()
//    }
//
//    fn print<'a, 'tcx>(&self, cx: &LateContext<'a, 'tcx>, file: &mut File) -> () {
//        if !self.sources.is_empty() {
//            //writeln!(file, "\nUnsafety in unsafe blocks: ");
//            for (node_id, block_sources) in self.sources.iter() {
//                // todo print span with \n as new line
//                let item = cx.tcx.hir.get(*node_id);
//                if let hir::Node::Block(ref block) = item {
//                    let span = block.span;
//                    write!(file, "\nBlock node_id: {}, Block: {}",
//                             node_id, cx.tcx.sess.source_map().span_to_snippet(span).unwrap());
//                }
//                for source in block_sources {
//                    writeln!(file,"");
//                    source.print(cx, file);
//                }
//                writeln!(file,"");
//            }
//        }
//    }
//
//}

//impl Analysis for BlockUnsafetyAnalysisSources {
//    fn is_set(&self) -> bool {
//        false
//    }
//
//    fn set(&mut self) {}
//
//    fn run_analysis<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, fn_info: &'a FnInfo) -> Self {
//        let tcx = cx.tcx;
//        let mut analysis: Self = Self::new();
//        let fn_def_id = tcx.hir.local_def_id(fn_info.decl_id());
//        // closures are handled by their parent fn.
//        if !cx.tcx.is_closure(fn_def_id) {
//            let mir = &mut tcx.optimized_mir(fn_def_id);
//            for (bb,bbd) in mir.basic_blocks() {
//                // is the bb marked unsafe
//                if let Some(mut body_visitor) = UnsafetySourcesVisitor::new(
//                    cx, mir, &mut analysis, fn_def_id) {
//                    body_visitor.visit_mir(mir);
//                }
//            }
//        }
//        analysis
//    }
//}

//////////////////////////////////////////////////////////////////////
// Common Parts
//////////////////////////////////////////////////////////////////////

trait UnsafetySourceCollector {
    fn add_source( &mut self, Source, NodeId);
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

impl <'a, 'tcx> UnsafetySourcesVisitor<'a, 'tcx> {
    fn new(cx: &'a LateContext<'a, 'tcx>,
           mir: &'a Mir<'tcx>, data: &'a mut UnsafetySourceCollector,
           fn_def_id: hir::def_id::DefId ) -> Option<Self> {
        match mir.source_scope_local_data {
            ClearCrossCrate::Set(ref local_data) => Some (
                UnsafetySourcesVisitor {
                    cx,
                    fn_node_id : cx.tcx.hir.def_index_to_node_id( fn_def_id.index ),
                    mir, data,
                    param_env: cx.tcx.param_env(fn_def_id),
                    source_info: SourceInfo {
                        span: mir.span,
                        scope: OUTERMOST_SOURCE_SCOPE,
                    },
                    source_scope_local_data: local_data,
                }
            ),
            ClearCrossCrate::Clear => {
                println!("unsafety_violations: {:?} - remote, skipping", fn_def_id);
                None
            }
        }
    }

    fn get_unsafety_node_id(&self) -> NodeId{
        match self.source_scope_local_data[self.source_info.scope].safety {
            Safety::Safe => {
                self.fn_node_id
            }
            Safety::BuiltinUnsafe | Safety::FnUnsafe => self.fn_node_id,
            Safety::ExplicitUnsafe(node_id) => {
                node_id
            }
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
                    if let Some(unsafe_fn_call) = Source::new_unsafe_fn_call(self.cx, func, loc) {
//                        println!("Unsafe function call!! {:?} {:?}", func,
//                                 util::get_file_and_line(self.cx,loc.span));
                        let unsafety_node_id = self.get_unsafety_node_id();
                        self.data.add_source(unsafe_fn_call, unsafety_node_id);
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
//                println!("Asm");
                let unsafety_node_id = self.get_unsafety_node_id();
                self.data.add_source(Source {
                    kind: SourceKind::Asm,
                    loc: statement.source_info,
                }, unsafety_node_id);
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
                        source_scope_local_data: self.source_scope_local_data
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
//                println!("Unalligned Borrow");
                let unsafety_node_id = self.get_unsafety_node_id();
                self.data.add_source(Source {
                    kind: SourceKind::BorrowPacked,
                    loc: self.source_info,
                }, unsafety_node_id);
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
//                        println!("DerefRawPointer");
                        let mut output = std::format!("{}", base_ty.sty);
                        let unsafety_node_id = self.get_unsafety_node_id();
                        self.data.add_source(Source {
                            kind: SourceKind::DerefRawPointer(output),
                            loc: self.source_info,
                        }, unsafety_node_id);
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
//                                    println!("AssignmentToNonCopyUnionField");
                                    let unsafety_node_id = self.get_unsafety_node_id();
                                    self.data.add_source(Source {
                                        kind: SourceKind::AssignmentToNonCopyUnionField(adt.did),
                                        loc: self.source_info,
                                    }, unsafety_node_id);
                                } else {
                                    // write to non-move union, safe
                                }
                            } else {
//                                println!("AccessToUnionField");
                                let unsafety_node_id = self.get_unsafety_node_id();
                                self.data.add_source(Source {
                                    kind: SourceKind::AccessToUnionField(adt.did),
                                    loc: self.source_info,
                                }, unsafety_node_id);
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
                //println!("Place::Promoted {:?}", p);
            }
            &Place::Static(box Static { def_id, ty: _ }) => {
                if self.cx.tcx.is_static(def_id) == Some(hir::Mutability::MutMutable) {
                    println!("Static");
                    let unsafety_node_id = self.get_unsafety_node_id();
                    self.data.add_source(Source {
                        kind: SourceKind::Static(def_id),
                        loc: self.source_info,
                    }, unsafety_node_id);
                } else if self.cx.tcx.is_foreign_item(def_id) {
                    println!("ExternStatic");
                    let unsafety_node_id = self.get_unsafety_node_id();
                    self.data.add_source(Source {
                        kind: SourceKind::ExternStatic(def_id),
                        loc: self.source_info,
                    }, unsafety_node_id);
                }
            }
        };
        self.super_place(place, context, location);
    }
}

