use rustc::lint::LateContext;

use rustc::hir;
use rustc::ty;

use rustc::mir::visit::Visitor;
use rustc::mir::{BasicBlock,Terminator,Location,TerminatorKind,Operand};

use FnInfo;

pub struct SafeMethodsInUnsafeTraits <'a, 'tcx: 'a> {
    cx: &'a LateContext<'a, 'tcx>,
    fn_info: &'a mut FnInfo,
}

impl <'a, 'tcx> SafeMethodsInUnsafeTraits<'a, 'tcx> {
    pub fn new(cx: &'a LateContext<'a, 'tcx>,
               fn_info: &'a mut FnInfo) -> Self {
        SafeMethodsInUnsafeTraits { cx, fn_info}
    }
}

impl<'a,'tcx> Visitor<'tcx> for SafeMethodsInUnsafeTraits<'a,'tcx> {
    fn visit_terminator(&mut self, _block: BasicBlock,
                        terminator: &Terminator<'tcx>,
                        _location: Location) {
        if let TerminatorKind::Call{ref func, args: _, destination: _, cleanup: _} = terminator.kind {
            if let Operand::Constant(constant) = func {
                match constant.literal.ty.sty {
                    ty::TypeVariants::TyFnDef(callee_def_id,_) => {
                        let calee_sig = self.cx.tcx.fn_sig(callee_def_id);
                        if let hir::Unsafety::Normal = calee_sig.unsafety() {
                            // need to find the trait if it's a method impl
                            println!("Call {:?}", constant.literal.ty.sty);
                            println!("is_impl_trait_defn {:?}", ty::is_impl_trait_defn(self.cx.tcx, callee_def_id));
                        }

//                        if callee_def_id.is_local() {
//                            if let Some (callee_node_id) = self.cx.tcx.hir.as_local_node_id(callee_def_id) {
//                                println!("calee {:?}", self.cx.tcx.hir.get( self.cx.tcx.hir.def_index_to_node_id(callee_def_id.index) ));
//                            }
//                        }
                    }
                    _ => {}
                }

            }
        }
    }
}