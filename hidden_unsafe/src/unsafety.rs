
use rustc::lint::LateContext;
use rustc::mir::{SourceInfo,Operand};
use rustc::hir;

use util::FnCallInfo;
use print::Print;

pub struct Source {
    pub loc: SourceInfo,
    pub kind: SourceKind,
}

pub enum SourceKind {
    UnsafeFnCall(FnCallInfo),
    DerefRawPointer(String), // TODO find a better solution
    Asm,
    MutableStatic (hir::def_id::DefId),
    //ForeignItem, //TODO check what is this
    BorrowPacked,
    AssignmentToNonCopyUnionField (hir::def_id::DefId),
    AccessToUnionField (hir::def_id::DefId),
    UseExternStatic (hir::def_id::DefId),
}


impl Source {
    pub fn new_unsafe_fn_call <'a, 'tcx> ( cx: &LateContext<'a, 'tcx>, func: &Operand<'tcx>,
                                        loc: SourceInfo  ) -> Option<Self> {
        if let Some(call_info) = ::util::find_callee(cx,func) {
            let kind = SourceKind::UnsafeFnCall(call_info);
            Some (Self { loc, kind })
        } else {None}
    }
}

impl Print for Source {
    fn print<'a, 'tcx>(&self, cx: &LateContext<'a, 'tcx>) -> () {
        match self.kind {
            SourceKind::UnsafeFnCall(ref callee_info) => {
                print!("UnsafeFnCall");
                callee_info.print(cx);
            },
            SourceKind::DerefRawPointer(ref ty) => {print!("DerefRawPointer | Type {:?}",ty);},
            SourceKind::Asm => {print!("Asm");},
            SourceKind::MutableStatic (ref def_id) => {print!("MutateStatic {:?}", def_id);},
            //SourceKind::ForeignItem => {print!("ForeignItem");},
            SourceKind::BorrowPacked => {print!("BorrowPacked");},
            SourceKind::AssignmentToNonCopyUnionField (ref adt_def) => {print!("AssignmentToNonCopyUnionField {:?}", adt_def);},
            SourceKind::AccessToUnionField (ref adt_def) => {print!("AccessToUnionField {:?}", adt_def);},
            SourceKind::UseExternStatic (ref adt_def) => {print!("UseExternStatic {:?}", adt_def);},
        }
        //TODO fix location printing
        print!(" | Loc: {:?}", self.loc);
    }
}