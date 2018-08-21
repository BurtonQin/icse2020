use analysis::Analysis;
use fn_info::FnInfo;
use print::Print;
use rustc::hir;
use rustc::lint::LateContext;
use rustc::mir::visit::Visitor;
use rustc::mir::{BasicBlock, Location, Operand, Terminator, TerminatorKind};
use rustc::ty::TypeVariants;
use std::fs::File;
use std::io::Write;

pub struct UnsafeTraitSafeMethod {
    has_unsafe: bool,
}

impl Print for UnsafeTraitSafeMethod {
    fn print<'a, 'tcx>(&self, _cx: &LateContext<'a, 'tcx>, file: &mut File) -> () {
        write!(file, "{:?}", self.has_unsafe);
    }
}

impl UnsafeTraitSafeMethod {
    fn new() -> Self {
        UnsafeTraitSafeMethod { has_unsafe: false }
    }
}

impl Analysis for UnsafeTraitSafeMethod {
    fn is_set(&self) -> bool {
        self.has_unsafe
    }

    fn set(&mut self) {
        self.has_unsafe = true
    }

    fn run_analysis<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, fn_info: &'a FnInfo) -> Self {
        let tcx = cx.tcx;
        let owner_def_id = tcx.hir.local_def_id(fn_info.decl_id());
        let mut mir = tcx.optimized_mir(owner_def_id);
        let mut unsafe_trait_visitor = SafeMethodsInUnsafeTraits::new(cx);
        unsafe_trait_visitor.visit_mir(&mut mir);
        let mut analysis: Self = UnsafeTraitSafeMethod::new();
        if unsafe_trait_visitor.has_unsafe {
            analysis.set();
        }
        analysis
    }
}

pub struct SafeMethodsInUnsafeTraits<'a, 'tcx: 'a> {
    cx: &'a LateContext<'a, 'tcx>,
    has_unsafe: bool,
}

impl<'a, 'tcx> SafeMethodsInUnsafeTraits<'a, 'tcx> {
    pub fn new(cx: &'a LateContext<'a, 'tcx>) -> Self {
        SafeMethodsInUnsafeTraits {
            cx,
            has_unsafe: false,
        }
    }
}

impl<'a, 'tcx> Visitor<'tcx> for SafeMethodsInUnsafeTraits<'a, 'tcx> {
    fn visit_terminator(
        &mut self,
        _block: BasicBlock,
        terminator: &Terminator<'tcx>,
        _location: Location,
    ) {
        if let TerminatorKind::Call {
            ref func,
            args: _,
            destination: _,
            cleanup: _,
        } = terminator.kind
        {
            if let Operand::Constant(constant) = func {
                if let TypeVariants::TyFnDef(callee_def_id, _) = constant.literal.ty.sty {
                    let calee_sig = self.cx.tcx.fn_sig(callee_def_id);
                    if let hir::Unsafety::Normal = calee_sig.unsafety() {
                        // need to find the trait if it's a method impl
                        if callee_def_id.is_local() {
                            let callee_node_id =
                                self.cx.tcx.hir.def_index_to_node_id(callee_def_id.index);
                            match self.cx.tcx.hir.get(callee_node_id) {
                                hir::map::Node::NodeTraitItem(ref _trait_item) => {
                                    let trait_node_id =
                                        self.cx.tcx.hir.get_parent_node(callee_node_id);
                                    if let hir::map::Node::NodeItem(item) =
                                        self.cx.tcx.hir.get(trait_node_id)
                                    {
                                        if let hir::ItemKind::Trait(_, unsafety, ..) = item.node {
                                            if let hir::Unsafety::Unsafe = unsafety {
                                                self.has_unsafe = true;
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
