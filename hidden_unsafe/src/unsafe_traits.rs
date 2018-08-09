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
                if let ty::TypeVariants::TyFnDef(callee_def_id,_) = constant.literal.ty.sty {
                    let calee_sig = self.cx.tcx.fn_sig(callee_def_id);
                    if let hir::Unsafety::Normal = calee_sig.unsafety() {
                        // need to find the trait if it's a method impl
                        if callee_def_id.is_local() {
                            let callee_node_id = self.cx.tcx.hir.def_index_to_node_id(callee_def_id.index);
                            match self.cx.tcx.hir.get(callee_node_id) {
                                hir::map::Node::NodeTraitItem(ref trait_item) => {
                                    let trait_node_id = self.cx.tcx.hir.get_parent_node(callee_node_id);
                                    if let hir::map::Node::NodeItem (item) = self.cx.tcx.hir.get(trait_node_id) {
                                        if let hir::ItemKind::Trait(_,unsafety,..) = item.node {
                                            if let hir::Unsafety::Unsafe = unsafety {
                                                self.fn_info.unsafe_trait_use = true;
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}