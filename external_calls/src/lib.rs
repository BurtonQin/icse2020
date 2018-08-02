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

use rustc::hir;
use rustc::hir::{Expr, Expr_, Crate, Ty, print};
use rustc::hir::def::Def;
use rustc::hir::def_id::{LOCAL_CRATE, CrateNum, DefId};
use rustc::hir::map::definitions::DefPath;

use rustc::ty::TypeAndMut;

use rustc::lint::{LateLintPass, LateLintPassObject, LateContext};
use rustc::lint::{LintPass, LintArray};

use syntax::ptr::P;
use std::fmt;

struct ExternalCalls
{
    external_crates: Vec<CrateNum>,
    external_calls: Vec<(CrateNum,String)>,
}

impl ExternalCalls {
    
    fn add_crate(&mut self, krate: CrateNum) -> () {
        let found = self.external_crates.iter().any(|elt| *elt == krate);
        if !found {
            self.external_crates.push(krate);
        }
    }

    fn add_call(&mut self, krate: CrateNum, func:String) -> () {
        self.add_crate(krate);
        let found = self.external_calls.iter().any(|elt| elt.1 == func);
        if !found {
            self.external_calls.push((krate,func));
        }
    }

}
declare_lint!(pub EXERNAL_CALLS, Allow, "Collect external function calls");

impl LintPass for ExternalCalls{
    fn get_lints(&self) -> LintArray {
        lint_array!(EXERNAL_CALLS)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for ExternalCalls {
    
    fn check_expr(&mut self, cx: &LateContext, expr: &Expr) {
        match expr.node {
            Expr_::ExprCall(ref callee, _) => {
                match callee.node {
                    Expr_::ExprPath(ref qpath) => {
                        let ref calee_def = cx.tables.qpath_def(qpath, callee.hir_id);
                        match calee_def {
                            Def::Fn(def_id) |
                            Def::Method(def_id) => {
                                if !def_id.is_local() {                                    
                                    match qpath {
                                        hir::QPath::Resolved(oty,call_path) => {
                                            match oty {
                                                None => {
                                                    self.add_call(def_id.krate,
                                                                  call_path.to_string());
                                                }
                                                Some (ty) => {
                                                    println!("Expr {:?}", expr);
                                                    //TODO
                                                }
                                            }
                                        }
                                        hir::QPath::TypeRelative(ref pty,path) => {
                                            let tys = print::to_string(
                                                print::NO_ANN,
                                                |s| s.print_type(pty));
                                            let mut res = String::new();
                                            res.push_str(&tys);
                                            res.push_str("::");
                                            res.push_str(&path.name.to_string());
                                            self.add_call(def_id.krate, res);
                                        }
                                    }
                                } 
                            }
                            Def::VariantCtor(..) => { }
                            _ => {
                                println!("Not Def::Fn in {:?}", expr); 
                            }
                        }
                    },              
                    _ => {
                        println!("Not ExprPath {:?}", expr);
                    }
                }
            }
            Expr_::ExprMethodCall(ref ps,_,ref args) => {
                let local_table = cx.tables.type_dependent_defs();
                let def = local_table.get(expr.hir_id);
                
                let arg_expr = &args[0];
                match def {
                    Some(hir::def::Def::Method(def_id)) => {
                        if !def_id.is_local() {
                            let mut call = String::new();
                            let ty = cx.tables.expr_ty_adjusted(&arg_expr);
                            match ty.sty {
                                rustc::ty::TypeVariants::TyRef(_,ref ty,_) => {
                                    call.push_str(&ty.to_string());
                                }
                                _ => {
                                    call.push_str(&ty.to_string());
                                }
                            }
                            call.push_str("::");
                            call.push_str(&ps.name.to_string());
                            self.add_call(def_id.krate, call);
                        }
                    }
                    _ => {println!("Def not hir::def::Def::Method {:?}", arg_expr);}
                }
            }
            _ =>  {}
        }
    }

    fn check_crate_post(&mut self, cx: &LateContext<'a, 'tcx>, _: &'tcx Crate) {
        self.external_crates.iter().for_each(
            |krate| {
                println!("===================================================");
                println!("External crate name {:?}",
                         cx.tcx.crate_name(*krate));
                self.external_calls.iter().
                    filter(|elt| elt.0 == *krate).
                    for_each (|elt| {
                        println!("{:?}", elt.1);
                    }) ;
            });
    }

}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_late_lint_pass(box ExternalCalls{ external_calls: Vec::new(),
                                                   external_crates: Vec::new()
    } as LateLintPassObject);
}
