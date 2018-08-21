use fn_info::FnInfo;
use rustc::hir;
use rustc::lint::LateContext;
use rustc::mir::visit::Visitor;
use rustc::mir::{BasicBlock, Location, Operand, Terminator, TerminatorKind};
use rustc::ty::TypeVariants;

pub fn build_call_graph<'a, 'tcx>(data: &mut Vec<FnInfo>, cx: &LateContext<'a, 'tcx>) {
    let tcx = &cx.tcx;
    let hir = &tcx.hir;
    for mut fn_info in data.iter_mut() {
        let body_owner_kind = hir.body_owner_kind(fn_info.decl_id());
        if let hir::BodyOwnerKind::Fn = body_owner_kind {
            let owner_def_id = hir.local_def_id(fn_info.decl_id());
            let mut mir = &tcx.optimized_mir(owner_def_id);
            {
                let mut calls_visitor = CallsVisitor::new(cx, &mut fn_info);
                calls_visitor.visit_mir(mir);
            }
        }
    }
}

struct CallsVisitor<'a, 'tcx: 'a> {
    cx: &'a LateContext<'a, 'tcx>,
    fn_info: &'a mut FnInfo,
}

impl<'a, 'tcx> CallsVisitor<'a, 'tcx> {
    fn new(cx: &'a LateContext<'a, 'tcx>, fn_info: &'a mut FnInfo) -> Self {
        CallsVisitor { cx, fn_info }
    }
}

impl<'a, 'tcx> Visitor<'tcx> for CallsVisitor<'a, 'tcx> {
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
                    if callee_def_id.is_local() {
                        if let Some(callee_node_id) =
                            self.cx.tcx.hir.as_local_node_id(callee_def_id)
                        {
                            self.fn_info.push_local_call(callee_node_id);
                        }
                    } else {
//                        let mut output = std::format!("{}", constant.literal.ty.sty);
//                        self.fn_info.push_external_call(callee_def_id.krate, output);
                        self.fn_info.push_external_call( self.cx, callee_def_id);
                    }
                } else {
                    println!("TypeVariants NOT handled {:?}", constant.literal.ty.sty);
                }
            } else {
                println!("calls.rs::Operand Type NOT handled {:?}", func);
            }
        }
    }
}
