use rustc::hir;
use rustc::lint::LateContext;
use rustc::mir;
use rustc::mir::{BasicBlock, Location, Mir, Terminator, TerminatorKind};
use rustc::mir::visit::Visitor;
use rustc::ty;
use rustc::ty::TyKind;
use rustc_target;
use results::Abi;
use rustc::hir::def_id::DefId;
use convert_abi;
use get_fn_path;

pub fn run_analysis<'a, 'tcx>(cx: &'a LateContext<'a, 'tcx>) -> Vec<results::calls::ExternalCall> {
    let mut data = Vec::new();
    for &def_id in cx.tcx.mir_keys(hir::def_id::LOCAL_CRATE).iter() {
        let mir = &cx.tcx.optimized_mir(def_id);
        let mut visitor = UnsafeCallsVisitor::new(cx, mir, def_id, &mut data);
        visitor.visit_mir( mir );
    }
    data
}

struct UnsafeCallsVisitor<'a, 'tcx: 'a> {
    cx: &'a LateContext<'a, 'tcx>,
    mir: &'tcx Mir<'tcx>,
    fn_def_id: DefId,
    data: &'a mut Vec<results::calls::ExternalCall>,
}

impl<'a, 'tcx> UnsafeCallsVisitor<'a, 'tcx> {
    fn new(cx: &'a LateContext<'a, 'tcx>, mir: &'tcx Mir, fn_def_id: DefId, data: &'a mut Vec<results::calls::ExternalCall>,) -> Self {
        UnsafeCallsVisitor { cx, mir, fn_def_id, data }
    }
}

impl<'a, 'tcx> Visitor<'tcx> for UnsafeCallsVisitor<'a, 'tcx> {
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
        } = terminator.kind {
            match func.ty(&self.mir.local_decls,self.cx.tcx).sty {
                TyKind::FnDef(callee_def_id, substs) => {

                    if let hir::Unsafety::Unsafe = self.cx.tcx.fn_sig(callee_def_id).unsafety() {

                        match self.cx.tcx.fn_sig(callee_def_id).abi() {
                            rustc_target::spec::abi::Abi::Cdecl |
                            rustc_target::spec::abi::Abi::Stdcall |
                            rustc_target::spec::abi::Abi::Fastcall |
                            rustc_target::spec::abi::Abi::Vectorcall |
                            rustc_target::spec::abi::Abi::Thiscall |
                            rustc_target::spec::abi::Abi::Aapcs |
                            rustc_target::spec::abi::Abi::Win64 |
                            rustc_target::spec::abi::Abi::SysV64 |
                            rustc_target::spec::abi::Abi::PtxKernel |
                            rustc_target::spec::abi::Abi::Msp430Interrupt |
                            rustc_target::spec::abi::Abi::X86Interrupt |
                            rustc_target::spec::abi::Abi::AmdGpuKernel |
                            rustc_target::spec::abi::Abi::C |
                            rustc_target::spec::abi::Abi::System |
                            rustc_target::spec::abi::Abi::RustIntrinsic |
                            rustc_target::spec::abi::Abi::RustCall |
                            rustc_target::spec::abi::Abi::PlatformIntrinsic |
                            rustc_target::spec::abi::Abi::Unadjusted => {
                                self.data.push( get_external_call(self.cx, self.cx.tcx.fn_sig(callee_def_id).abi(), callee_def_id) );
                            }
                            rustc_target::spec::abi::Abi::Rust => {
                                if let hir::Unsafety::Unsafe = self.cx.tcx.fn_sig(callee_def_id).unsafety() {
                                    let param_env = self.cx.tcx.param_env(self.fn_def_id);
                                    if let Some(instance) = ty::Instance::resolve(self.cx.tcx, param_env, callee_def_id, substs) {
                                        match instance.def {
                                            ty::InstanceDef::Item(def_id)
                                            | ty::InstanceDef::Intrinsic(def_id)
                                            | ty::InstanceDef::Virtual(def_id, _)
                                            | ty::InstanceDef::DropGlue(def_id, _) => {
                                                self.data.push(
                                                    get_external_call(
                                                        self.cx,
                                                    self.cx.tcx.fn_sig(def_id).abi(),def_id) );
                                            }
                                            _ => error!("ty::InstanceDef:: NOT handled {:?}", instance.def),
                                        }
                                    } else {
                                        // Generics
                                        self.data.push( get_external_call(
                                            self.cx,
                                            self.cx.tcx.fn_sig(callee_def_id).abi(), callee_def_id) );
                                    }
                                }
                            }

                        }
                    }
                }
                TyKind::FnPtr(ref poly_sig) => {

                    match func {
                        mir::Operand::Move(arg)
                        | mir::Operand::Copy(arg) => {
                            info!("func {:?} is fn ptr", arg.ty(&self.mir.local_decls,self.cx.tcx));
                            if let hir::Unsafety::Unsafe = poly_sig.unsafety() {
                                let elt = results::calls::ExternalCall {
                                    abi: convert_abi(poly_sig.abi()),
                                    def_path: "Unsafe_Call_Fn_Ptr".to_string(),
                                    name: arg.ty(&self.mir.local_decls,self.cx.tcx).to_ty(self.cx.tcx).to_string(),
                                    crate_name: "Unsafe_Call_Fn_Ptr".to_string(),
                                };
                                self.data.push(elt);
                            }
                        }
                        _ => {
                        }
                    }
                }
                _ => {
                    error!("TypeVariants NOT handled {:?}", func.ty(&self.mir.local_decls, self.cx.tcx).sty);
                }
            }
        }
    }
}

fn get_external_call<'a, 'tcx>(cx: &'a LateContext<'a, 'tcx>, abi: rustc_target::spec::abi::Abi, def_id: DefId) -> results::calls::ExternalCall {

    let crate_name =
        if def_id.is_local() {
            ::local_crate_name()
        } else {
            cx.tcx.crate_name(def_id.krate).to_string()
        };

    results::calls::ExternalCall {
        abi: convert_abi(abi),
        def_path: get_fn_path( cx, def_id),
        name: cx.tcx.item_name(def_id).to_string(),
        crate_name
    }
}
