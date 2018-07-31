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
use rustc::hir::{Expr, Expr_, Crate, Ty};
use rustc::hir::def::Def;
use rustc::hir::def_id::{LOCAL_CRATE, CrateNum, DefId};
use rustc::hir::map::definitions::DefPath;

use rustc::lint::{LateLintPass, LateLintPassObject, LateContext};
use rustc::lint::{LintPass, LintArray};

use syntax::ptr::P;


struct ExternalCalls
{
    external_crates: Vec<CrateNum>,
    external_calls: Vec<(DefPath,Option<String>)>,
}

impl ExternalCalls {
    
    fn add_crate(&mut self, krate: CrateNum) -> () {
        let found = self.external_crates.iter().any(|elt| *elt == krate);
        if !found {
            self.external_crates.push(krate);
        }
    }

    fn add_def_id(&mut self, def_path: DefPath, ty: Option<String>) -> () {
        let found = self.external_calls.iter().any(|elt| elt.0 == def_path && elt.1 == ty);
        if !found {
            self.external_calls.push((def_path,ty));
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
        let _t = cx.tables.expr_ty(&expr);
        let maybe_def_id : Option<(DefPath,Option<String>)> = match expr.node {
            Expr_::ExprCall(ref callee, _) => {
                match callee.node {
                    Expr_::ExprPath(ref qpath) => {
                        let def = cx.tables.qpath_def(qpath, callee.hir_id);                        
                        match def {
                            Def::Fn(def_id) => { Some ((cx.tcx.def_path(def_id),None)) }
                            Def::Method(def_id) => { Some ((cx.tcx.def_path(def_id),None)) }
                            Def::VariantCtor(..) => { None }
                            _ => {
                                println!("Not Def::Fn {:?}", def);
                                None 
                            }
                        }
                    },                    
                    _ => {
                        println!("Not ExprPath {:?}", callee.node);
                        None
                    }
                }
            },
            Expr_::ExprMethodCall(ref ps,_,ref args) => {
                let def = cx.tables.type_dependent_defs().get(expr.hir_id).cloned();
                let arg_expr = &args[0];
                match def {
                    Some(hir::def::Def::Method(def_id)) => {
                        match arg_expr.node {
                            Expr_::ExprPath(ref qpath) => {
                                let def = cx.tables.qpath_def(qpath, arg_expr.hir_id);
                                println!("Expr: {:?}",  expr);
                                println!("Ps: {:?}",  ps);
                                println!("Args: {:?}",  args);
                                println!("Arg qpath {:?}", qpath);
                                println!("DefI: {:?}",  def);
                                println!("Local root id {:?}", cx.tables.local_id_root);
                                match cx.tables.local_id_root {
                                    Some (did) => {println!("Local root id def path {:?}", cx.tcx.def_path(did));}
                                    None => {println!("Local root None");}
                                }
                                match def {
                                    hir::def::Def::Local(node_id) => {
                                        //Some ((cx.tcx.def_path(def_id), Some(def.kind_name().to_string())))
                                        Some ((cx.tcx.def_path(def_id), Some(cx.tcx.node_path_str(node_id)))) }
                                    _ => {println!("arg_expr not not Local {:?}", def.kind_name().to_string()); None} 
                                }
                            }
                            _ => {println!("arg_expr not ExprPath {:?}", arg_expr); None}                              
                        }
                    }
                    _ =>  {
                        println!("Not in cx.tables.type_dependent_defs().get(expr.hir_id).cloned();");
                        None
                    }            
                }
            },
            _ => None
        };

        match expr.node {
            Expr_::ExprCall(..) |
            Expr_::ExprMethodCall(..) => {
                match maybe_def_id {  
                    Some ((path,ty)) => {
                        if path.krate != LOCAL_CRATE {                     
                            self.add_crate(path.krate);
                            self.add_def_id(path,ty);
                        }
                    }                
                    None => {
                        // println!("DefId Not Found: {:?}",  maybe_def_id);
                    }
                }
            }
            _ => {}
        } 
    }

    fn check_crate_post(&mut self, cx: &LateContext<'a, 'tcx>, _: &'tcx Crate) {
        self.external_crates.iter().for_each(
            |krate| {
                println!("===================================================");
                println!("External crate name {:?}",
                         cx.tcx.crate_name(*krate));
                self.external_calls.iter().
                    filter(|ref elt| elt.0.krate == *krate).
                    for_each (|ref elt| {
                        // TODO print full path
                        match elt.1 {
                            None => { println!("{:?}", elt.0); }
                            Some (ref arg_expr) => { println!("{:?} {:?}", elt.0, arg_expr); }
                                                              //cx.tables.expr_ty(&arg_expr)); }
                        }                        
                        //println!("{:?}", cx.tcx.def_symbol_name(*def_id));
                    }) ;
            });
    }

}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_late_lint_pass(box ExternalCalls{ external_calls: Vec::new(), external_crates: Vec::new() } as LateLintPassObject);
}
