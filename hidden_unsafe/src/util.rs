use syntax::ast::NodeId;

use rustc::lint::LateContext;
use rustc::mir::Operand;
use rustc::hir;
use rustc::ty::TypeVariants;
use rustc_target::spec::abi::Abi;

use print::Print;

pub enum FnCallInfo {
    Local(NodeId,Abi),
    External(hir::def_id::CrateNum, String,Abi),
}

impl Print for FnCallInfo {
    fn print<'a, 'tcx>(&self, cx: &LateContext<'a, 'tcx>) -> () {
        match self {
            FnCallInfo::Local(node_id,abi) => {
                print!("{:?} |abi: {:?}",  cx.tcx.node_path_str(*node_id), abi);
            }
            FnCallInfo::External(krate, path_str, abi) => {
                print!("Crate: {:?} | Calee: {:?} | abi: {:?}",
                       cx.tcx.crate_name(*krate), path_str, abi
                );
            }
        }

    }
}

pub fn find_callee<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, func: &Operand<'tcx>) -> Option<FnCallInfo> {
    if let Operand::Constant(constant) = func {
        if let TypeVariants::TyFnDef(callee_def_id, _) = constant.literal.ty.sty {
            let abi = cx.tcx.fn_sig(callee_def_id).abi();
            if callee_def_id.is_local() {
                if let Some(callee_node_id) = cx.tcx.hir.as_local_node_id(callee_def_id) {
                    Some(FnCallInfo::Local(callee_node_id,abi))
                } else{
                    println!("local node id NOT found {:?}", callee_def_id);
                    None
                }
            } else {
                let mut output = std::format
                !("{}", constant.literal.ty.sty);
                Some(FnCallInfo::External(callee_def_id.krate, output,abi))
            }
        } else {
            println!("TypeVariants NOT handled {:?}", constant.literal.ty.sty);
            None
        }
    } else {
        println!("Operand Type NOT handled {:?}", func);
        None
    }
}
