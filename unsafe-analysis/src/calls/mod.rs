use rustc::hir;
use rustc::lint::LateContext;
use rustc::mir::{BasicBlock, Location, Mir, Terminator, TerminatorKind};
use rustc::mir::visit::Visitor;
use rustc::ty;
use rustc::ty::TyKind;
use rustc_target;
use results::unsafety_sources::Abi;

pub fn run_analysis<'a, 'tcx>(cx: &'a LateContext<'a, 'tcx>) -> Vec<(Abi, String)> {
    let mut data = Vec::new();
    for &def_id in cx.tcx.mir_keys(hir::def_id::LOCAL_CRATE).iter() {
        let mir = &cx.tcx.optimized_mir(def_id);
        let mut visitor = UnsafeCallsVisitor::new(cx, mir, &mut data);
        visitor.visit_mir( mir );
    }
    data
}

struct UnsafeCallsVisitor<'a, 'tcx: 'a> {
    cx: &'a LateContext<'a, 'tcx>,
    mir: &'tcx Mir<'tcx>,
    data: &'a mut Vec<(Abi, String)>,
}

impl<'a, 'tcx> UnsafeCallsVisitor<'a, 'tcx> {
    fn new(cx: &'a LateContext<'a, 'tcx>, mir: &'tcx Mir, data: &'a mut Vec<(Abi,String)>) -> Self {
        UnsafeCallsVisitor { cx, mir, data }
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
                            rustc_target::spec::abi::Abi::Cdecl => {
                                self.data.push((Abi::Cdecl, self.cx.tcx.item_name(callee_def_id).to_string()));
                            }
                            rustc_target::spec::abi::Abi::Stdcall => {
                                self.data.push((Abi::Stdcall, self.cx.tcx.item_name(callee_def_id).to_string()));
                            }
                            rustc_target::spec::abi::Abi::Fastcall => {
                                self.data.push((Abi::Fastcall, self.cx.tcx.item_name(callee_def_id).to_string()));
                            }
                            rustc_target::spec::abi::Abi::Vectorcall => {
                                self.data.push((Abi::Vectorcall, self.cx.tcx.item_name(callee_def_id).to_string()));
                            }
                            rustc_target::spec::abi::Abi::Thiscall => {
                                self.data.push((Abi::Thiscall, self.cx.tcx.item_name(callee_def_id).to_string()));
                            }
                            rustc_target::spec::abi::Abi::Aapcs => {
                                self.data.push((Abi::Aapcs, self.cx.tcx.item_name(callee_def_id).to_string()));
                            }
                            rustc_target::spec::abi::Abi::Win64 => {
                                self.data.push((Abi::Win64, self.cx.tcx.item_name(callee_def_id).to_string()));
                            }
                            rustc_target::spec::abi::Abi::SysV64 => {
                                self.data.push((Abi::SysV64, self.cx.tcx.item_name(callee_def_id).to_string()));
                            }
                            rustc_target::spec::abi::Abi::PtxKernel => {
                                self.data.push((Abi::PtxKernel, self.cx.tcx.item_name(callee_def_id).to_string()));
                            }
                            rustc_target::spec::abi::Abi::Msp430Interrupt => {
                                self.data.push((Abi::Msp430Interrupt, self.cx.tcx.item_name(callee_def_id).to_string()));
                            }
                            rustc_target::spec::abi::Abi::X86Interrupt => {
                                self.data.push((Abi::X86Interrupt, self.cx.tcx.item_name(callee_def_id).to_string()));
                            }
                            rustc_target::spec::abi::Abi::AmdGpuKernel => {
                                self.data.push((Abi::AmdGpuKernel, self.cx.tcx.item_name(callee_def_id).to_string()));
                            }
                            rustc_target::spec::abi::Abi::Rust => {

                                let call_instance = match std::env::var("DO_NOT_USE_INSTANCE") {
                                    Err(_) => {"".to_string()}
                                    Ok(val) => {val}
                                };

                                if call_instance == "" {
                                    if let hir::Unsafety::Unsafe = self.cx.tcx.fn_sig(callee_def_id).unsafety() {
                                        let param_env = self.cx.tcx.param_env(callee_def_id);
                                        if let Some(instance) = ty::Instance::resolve(self.cx.tcx, param_env, callee_def_id, substs) {
                                            match instance.def {
                                                ty::InstanceDef::Item(def_id)
                                                | ty::InstanceDef::Intrinsic(def_id)
                                                | ty::InstanceDef::Virtual(def_id, _)
                                                | ty::InstanceDef::DropGlue(def_id, _) => {
                                                    self.data.push((Abi::Rust, self.cx.tcx.absolute_item_path_str(def_id).to_string()));
                                                }
                                                _ => error!("ty::InstanceDef:: NOT handled {:?}", instance.def),
                                            }
                                        } else {
                                            // Generics
                                            self.data.push((Abi::Rust, self.cx.tcx.absolute_item_path_str(callee_def_id)));
                                        }
                                    }
                                } else {
                                    self.data.push((Abi::Rust, self.cx.tcx.absolute_item_path_str(callee_def_id)));
                                }
                            }
                            rustc_target::spec::abi::Abi::C => {
                                self.data.push((Abi::C, self.cx.tcx.item_name(callee_def_id).to_string()));
                            }
                            rustc_target::spec::abi::Abi::System => {
                                self.data.push((Abi::System, self.cx.tcx.item_name(callee_def_id).to_string()));
                            }
                            rustc_target::spec::abi::Abi::RustIntrinsic => {
                                self.data.push((Abi::RustIntrinsic, self.cx.tcx.absolute_item_path_str(callee_def_id).to_string()));
                            }
                            rustc_target::spec::abi::Abi::RustCall => {
                                self.data.push((Abi::RustCall, self.cx.tcx.absolute_item_path_str(callee_def_id).to_string()));
                            }
                            rustc_target::spec::abi::Abi::PlatformIntrinsic => {
                                self.data.push((Abi::PlatformIntrinsic, self.cx.tcx.item_name(callee_def_id).to_string()));
                            }
                            rustc_target::spec::abi::Abi::Unadjusted => {
                                self.data.push((Abi::Unadjusted, self.cx.tcx.item_name(callee_def_id).to_string()));
                            }
                        }
                    }

                }
                TyKind::FnPtr(ref poly_sig) => {
                    if let hir::Unsafety::Unsafe = poly_sig.unsafety() {
                        self.data.push((Abi::Rust,"Unsafe_Call_Fn_Ptr".to_string()) );
                    }
                }
                _ => {
                    error!("TypeVariants NOT handled {:?}", func.ty(&self.mir.local_decls, self.cx.tcx).sty);
                }
            }
        }
    }
}
