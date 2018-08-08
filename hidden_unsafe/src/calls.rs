extern crate serialize;
extern crate core;

use rustc::lint::LateContext;

use rustc::hir;
use rustc::ty;

use rustc::mir::visit::Visitor;
use rustc::mir::{BasicBlock,Terminator,Location,TerminatorKind,Operand};

use FnInfo;


pub struct Calls<'a, 'tcx: 'a> {
    cx: &'a LateContext<'a, 'tcx>,
    fn_info: &'a mut FnInfo,
}


impl<'a, 'tcx> Calls<'a, 'tcx> {

    pub fn new(cx: &'a LateContext<'a, 'tcx>,
               fn_info: &'a mut FnInfo) -> Self {
        Calls { cx, fn_info}
    }

    pub fn run(&mut self) {
        let body_owner_kind = self.cx.tcx.hir.body_owner_kind(self.fn_info.decl_id);
        if let hir::BodyOwnerKind::Fn = body_owner_kind {
            let owner_def_id = self.cx.tcx.hir.local_def_id( self.fn_info.decl_id );
            let mut mir = self.cx.tcx.mir_validated(owner_def_id);
            //self.visit_mir(&mut mir.steal());
            self.visit_mir(&mut mir.borrow());
        }
    }
}

impl<'a,'tcx> Visitor<'tcx> for Calls<'a,'tcx> {
    fn visit_terminator(&mut self, _block: BasicBlock,
                        terminator: &Terminator<'tcx>,
                        _location: Location) {
        if let TerminatorKind::Call{ref func, args: _, destination: _, cleanup: _} = terminator.kind {
            if let Operand::Constant(constant) = func {
                match constant.literal.ty.sty {
                    ty::TypeVariants::TyFnDef(callee_def_id,_) => {
                        if callee_def_id.is_local() {
                            if let Some (callee_node_id) = self.cx.tcx.hir.as_local_node_id(callee_def_id) {
                                self.fn_info.push_local_call(callee_node_id);
                            }
                        } else {
                            let mut output = std::format!("{}", constant.literal.ty.sty);
                            self.fn_info.push_external_call(callee_def_id.krate, output);
                        }
                    }
                    _ => {}
                }

            }
        }
    }
}


