#![crate_name = "external_calls"]
#![crate_type = "dylib"]
#![feature(plugin_registrar)]
#![feature(rustc_private)]
#![feature(box_syntax)]
#![feature(macro_vis_matcher)]

#[macro_use]
extern crate rustc;

extern crate rustc_plugin;
extern crate syntax;

use rustc_plugin::Registry;
use rustc::hir::Expr;
use rustc::hir::Expr_;
use rustc::hir::Ty_;
use rustc::lint::LateLintPass;
use rustc::lint::LateLintPassObject;
use rustc::lint::LateContext;
use rustc::lint::LintPass;
use rustc::lint::LintArray;
use rustc::hir::QPath;
use rustc::hir::print;
use rustc::hir::def_id::LOCAL_CRATE;
use rustc::hir::def_id::DefId;
use rustc::hir::def::Def;
use rustc::hir;
use rustc::ty::TyCtxt;

struct ExternalCalls;
//{
    //crate_name: String;
    //external_calls: HashMap<,>;
//}

declare_lint!(NONE, Allow, "Collect external function calls");

impl LintPass for ExternalCalls{
    fn get_lints(&self) -> LintArray {
        lint_array!(NONE)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for ExternalCalls {
    fn check_expr(&mut self, cx: &LateContext, expr: &Expr) {
        
        let t = cx.tables.expr_ty(&expr);
        let mut ty_warned = false;
        let mut fn_warned = false;
        let mut op_warned = false;
        let maybe_def_id = match expr.node {
            Expr_::ExprCall(ref callee, _) => {
                match callee.node {
                    Expr_::ExprPath(ref qpath) => {
                        let def = cx.tables.qpath_def(qpath, callee.hir_id);
                        if let Def::Fn(def_id) = def {
                            Some(def_id)
                        } else {  // `Def::Local` if it was a closure, for which we
                            None  // do not currently support must-use linting
                        }
                    },
                    _ => None
                }
            },
            Expr_::ExprMethodCall(..) => {
                let def = cx.tables.type_dependent_defs().get(expr.hir_id).cloned();
                match def {
                    Some(hir::def::Def::Method(def_id)) => { Some(def_id) }
                    _ =>  { None }
                }
            },
            _ => None
        };

        match expr.node {
            Expr_::ExprCall(..) |
            Expr_::ExprMethodCall(..) => {
                println!("===================================================");
                println!("Expr {:?}", expr);
                println!("Type {:?}", expr);
        
                match maybe_def_id {
                    Some (did) =>
                        if (did.krate == LOCAL_CRATE) {
                            println!("Local crate name {:?}",
                                     cx.tcx.crate_name(did.krate));
                        } else {
                            println!("External crate name {:?}",
                                     cx.tcx.crate_name(did.krate));
                        }
                
                    None => { println!("def id not found") }
                }
            }
            _ => {}
        }
            
        
        // match expr.node {
        //     Expr_::ExprCall(ref fname,_) => {
        //         match fname.node {
        //             Expr_::ExprPath(ref path) => {
        //                 print_path(path);
        //                 println!();
        //                 println!("Call {:?}", fname);
        //                 let def_id = get_def_id(path);
        //                 match def_id {
        //                     Some (did) => {
        //                         println!("Crate id {:?}",
        //                                  ltcx.tcx.crate_name(did.krate));
        //                     }
        //                     None => {
        //                         println!("Crate is not found");
        //                     }
        //                 }
        //             }                        
        //             _ => {
        //                 println!("Call {:?}", fname);
        //             }
        //         }
        //     }
        //     Expr_::ExprMethodCall(ref ps,_,_) => {
        //         println!("MethodCall Type {:?}", ltcx.tables.node_id_to_type(expr.hir_id));
        //     }               
        //     _ => {}
        // }
    }
}

fn get_def_id(p: &QPath) -> Option<DefId> {
     match p {
         QPath::Resolved(_, path) => {
             match (*path).def {
                 rustc::hir::def::Def::Fn(defId) => { return Some(defId) }
                 _ => {}
             }
             // let v = path.into_vec();
             // for p1 in &v {
             //     match p1.def {
             //         rustc::hir::def::Def::Fn(defId) => { return Some(defId) }
             //         _ => {}
             //     }
             // }
             return None
         }
         QPath::TypeRelative(ty, ps) => {
             return None
         }
     }
}

fn print_path(p: &QPath) {
     match p {
         QPath::Resolved(_, path) => {
             //print!("{:?}", path1);
             print!("{:?}", print::to_string(print::NO_ANN, |s| s.print_path(path, false)))
         }
         QPath::TypeRelative(ty, ps) => {
             match ty.node {
                 Ty_::TyPath(ref p) => {
                     print_path(p);
                     print!("::{:?}", ps.name);
                 }
                 _ =>  {
                     println!("Call3 {:?}::{:?}", ty, ps);
                 }
             }
         }
     }
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_late_lint_pass(box ExternalCalls as LateLintPassObject);
}
