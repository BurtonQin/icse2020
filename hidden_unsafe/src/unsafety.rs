use rustc::hir;
use rustc::lint::LateContext;
use rustc::mir::{Operand, SourceInfo};

use print::Print;
use util::FnCallInfo;
use util;
use std::fs::File;
use std::io::Write;

pub struct Source {
    pub loc: SourceInfo,
    pub kind: SourceKind,
}

pub enum SourceKind {
    UnsafeFnCall(FnCallInfo),
    DerefRawPointer(String), // TODO find a better solution
    Asm,
    MutableStatic(hir::def_id::DefId),
    //ForeignItem, //TODO check what is this
    BorrowPacked,
    AssignmentToNonCopyUnionField(hir::def_id::DefId),
    AccessToUnionField(hir::def_id::DefId),
    UseExternStatic(hir::def_id::DefId),
}

impl Source {
    pub fn new_unsafe_fn_call<'a, 'tcx>(
        cx: &LateContext<'a, 'tcx>,
        func: &Operand<'tcx>,
        loc: SourceInfo,
    ) -> Option<Self> {
        if let Some(call_info) = ::util::find_callee(cx, func) {
            let kind = SourceKind::UnsafeFnCall(call_info);
            Some(Self { loc, kind })
        } else {
            None
        }
    }
}

impl Print for Source {
    fn print<'a, 'tcx>(&self, cx: &LateContext<'a, 'tcx>, file: &mut File) -> () {
        match self.kind {
            SourceKind::UnsafeFnCall(ref callee_info) => {
                write!(file, "UnsafeFnCall |  ");
                callee_info.print(cx, file);
            }
            SourceKind::DerefRawPointer(ref ty) => {
                write!(file, "DerefRawPointer | Type {:?}", ty);
            }
            SourceKind::Asm => {
                write!(file, "Asm");
            }
            SourceKind::MutableStatic(ref def_id) => {
                write!(file, "MutateStatic {:?}", def_id);
            }
            //SourceKind::ForeignItem => {print!("ForeignItem");},
            SourceKind::BorrowPacked => {
                write!(file, "BorrowPacked");
            }
            SourceKind::AssignmentToNonCopyUnionField(ref adt_def) => {
                write!(file, "AssignmentToNonCopyUnionField {:?}", adt_def);
            }
            SourceKind::AccessToUnionField(ref adt_def) => {
                print!("AccessToUnionField {:?}", adt_def);
            }
            SourceKind::UseExternStatic(ref adt_def) => {
                write!(file, "UseExternStatic {:?}", adt_def);
            }
        }
        write!(file, " | ");
        util::print_file_and_line(cx, self.loc.span, file);
    }
}
