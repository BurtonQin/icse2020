use rustc_target::spec::abi::Abi;
use rustc::lint::LateContext;
use syntax::ast::NodeId;
use rustc::mir::{SourceInfo,Operand};

use util::{FnCallInfo,find_callee};
use print::Print;

pub struct Source {
    loc: SourceInfo,
    kind: SourceKind,
}

pub enum SourceKind {
    UnsafeFnCall(FnCallInfo),
    DerefRawPointer,
    Asm,
    MutateStatic,
    ForeignItem, //TODO check what is this
    BorrowPacked,
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
    fn print<'a, 'tcx>(&self, cx: &LateContext<'a, 'tcx>) -> () {}
}