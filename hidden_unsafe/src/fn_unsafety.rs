
use rustc::hir;
use syntax::ast::NodeId;
use rustc::lint::LateContext;
use rustc::mir::{BasicBlock, Location, Operand, Terminator, TerminatorKind, Mir};
use rustc::mir::visit::Visitor;
use rustc::ty::TypeVariants;

use unsafety::{Source,SourceKind};
use fn_info::FnInfo;
use analysis::Analysis;
use print::Print;

// information about reasons for unsafety of functions declared unsafe
pub struct UnsafeFnUsafetyAnalysis {
    decl_id: NodeId,
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

impl UnsafeFnUsafetyAnalysis{

    fn new(decl_id: NodeId) -> Self {
        UnsafeFnUsafetyAnalysis {
            decl_id,
            arguments: Vec::new(),
            sources: Vec::new(),
        }
    }

    fn process_fn_decl<'a, 'tcx>(&mut self, cx: &LateContext<'a, 'tcx>) -> () {
        if let Some (fn_decl) = cx.tcx.hir.fn_decl(self.decl_id) {
            for input in fn_decl.inputs {
                if let Some (reason) = UnsafeFnUsafetyAnalysis::process_type(&input) {
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
            hir::TyKind::Slice(ref sty) |
            hir::TyKind::Array(ref sty, _) => { UnsafeFnUsafetyAnalysis::process_type(&sty) }

            hir::TyKind::Ptr(_) => {
                Some ( Argument {
                    ty_node_id: ty.id,
                     kind: ArgumentKind::RawPointer
                })
            }

            hir::TyKind::Rptr(_, _ ) => {None} //I think this is a Rust reference

            hir::TyKind::BareFn(ref bare_fn) => {
                if let hir::Unsafety::Unsafe = bare_fn.unsafety {
                    Some ( Argument {
                        ty_node_id: ty.id,
                        kind: ArgumentKind::UnsafeFunction
                    })
                } else {
                    UnsafeFnUsafetyAnalysis::process_ty_array(&bare_fn.decl.inputs)
                }
            }

            hir::TyKind::Tup(ref vty) => {
                UnsafeFnUsafetyAnalysis::process_ty_array(&vty)
            }

            hir::TyKind::Path(ref qpath) => {
                match qpath {
                    hir::QPath::Resolved( oty, _ ) => {
                        if let Some (sty) = oty {
                            UnsafeFnUsafetyAnalysis::process_type(sty)
                        } else {
                            None
                        }
                    }
                    hir::QPath::TypeRelative (pty,_) => {
                        UnsafeFnUsafetyAnalysis::process_type(pty)
                    }
                }
            }

            hir::TyKind::TraitObject(ref poly_ref,_) => {None} //TODO

            hir::TyKind::Never |
            hir::TyKind::Typeof(_) |
            hir::TyKind::Infer |
            hir::TyKind::Err => {None}
        }
    }

    fn process_ty_array( array: &hir::HirVec<hir::Ty> ) -> Option<Argument> {
        let mut iter = array.iter();
        let mut done = false;
        let mut res = None;
        while (!done ) {
            if let Some (elt) = iter.next() {
                let arg_res = UnsafeFnUsafetyAnalysis::process_type(elt);
                if let Some (_) = arg_res {
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



impl Analysis for UnsafeFnUsafetyAnalysis {

    fn is_set(&self) -> bool { false }

    fn set(&mut self) { }

    fn run_analysis<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, fn_info: &'a FnInfo)
                              -> Self {
        let tcx = cx.tcx;
        let mut analysis: Self = Self::new(fn_info.decl_id());
        let fn_def_id = tcx.hir.local_def_id(fn_info.decl_id());
        analysis.process_fn_decl(cx);
        { //needed for the borrow checker
            let mir = &mut tcx.mir_validated(fn_def_id).borrow();
            let mut body_visitor = FnVisitor { cx, mir, data: &mut analysis };
            body_visitor.visit_mir(mir);
        }
        analysis
    }
}

struct FnVisitor<'a, 'tcx: 'a> {
    cx: &'a LateContext<'a, 'tcx>,
    mir: &'a Mir<'tcx>,
    data: &'a mut UnsafeFnUsafetyAnalysis,
}

impl <'a, 'tcx> Visitor<'tcx> for FnVisitor<'a, 'tcx> {

    fn visit_terminator(&mut self,
                        block: BasicBlock,
                        terminator: &Terminator<'tcx>,
                        location: Location)
    {
//        self.source_info = terminator.source_info;
        match terminator.kind {
            TerminatorKind::Goto { .. } |
            TerminatorKind::SwitchInt { .. } |
            TerminatorKind::Drop { .. } |
            TerminatorKind::Yield { .. } |
            TerminatorKind::Assert { .. } |
            TerminatorKind::DropAndReplace { .. } |
            TerminatorKind::GeneratorDrop |
            TerminatorKind::Resume |
            TerminatorKind::Abort |
            TerminatorKind::Return |
            TerminatorKind::Unreachable |
            TerminatorKind::FalseEdges { .. } |
            TerminatorKind::FalseUnwind { .. } => {
                // safe (at least as emitted during MIR construction)
            }

            TerminatorKind::Call { ref func, .. } => {
                let func_ty = func.ty(self.mir, self.cx.tcx);
                let sig = func_ty.fn_sig(self.cx.tcx);
                if let hir::Unsafety::Unsafe = sig.unsafety() {
                    let loc = terminator.source_info;
                    if let Some (unsafe_fn_call) = Source::new_unsafe_fn_call(self.cx,func,loc) {
                        self.data.sources.push(unsafe_fn_call);
                    }
                }
            }
        }
        self.super_terminator(block, terminator, location);
    }

}

impl Print for UnsafeFnUsafetyAnalysis {

    fn print<'a,'tcx>(&self, cx: &LateContext<'a, 'tcx>) -> () {
        println!("\nUnsafety in arguments: ");
        for arg in &self.arguments {
            arg.print(cx);
        }
        println!("Unsafety in body: ");
        for source in &self.sources {
            source.print(cx);
        }
    }

}

impl Print for ArgumentKind {
    fn print<'a,'tcx>(&self, _cx: &LateContext<'a, 'tcx>) -> () {
        match self {
            RawPointer => { print!("RawPointer"); }
            UnsafeFunction => { print!("UnsafeFunction"); }
        }
    }
}

impl Print for Argument {
    fn print<'a,'tcx>(&self, cx: &LateContext<'a, 'tcx>) -> () {
        print!("Kind ");
        self.kind.print(cx);
        println!( " Type {:?}", cx.tcx.node_path_str(self.ty_node_id) );
    }
}