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
use rustc::lint::LateLintPass;
use rustc::lint::LateLintPassObject;
use rustc::lint::LateContext;
use rustc::lint::LintPass;
use rustc::lint::LintArray;
use rustc::hir::def_id::LOCAL_CRATE;
use rustc::hir::def::Def;
use rustc::hir;
use rustc::hir::def_id::CrateNum;
use rustc::hir::def_id::DefId;
use rustc::hir::Crate;

struct ExternalCalls
{
    external_crates: Vec<CrateNum>,
    external_calls: Vec<DefId>,
}

impl ExternalCalls {
    
    fn add_crate(&mut self, krate: CrateNum) -> () {
        let found = self.external_crates.iter().any(|elt| *elt == krate);
        if !found {
            self.external_crates.push(krate);
        }
    }

    fn add_def_id(&mut self, def_id: DefId) -> () {
        let found = self.external_calls.iter().any(|elt| *elt == def_id);
        if !found {
            self.external_calls.push(def_id);
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
        let maybe_def_id = match expr.node {
            Expr_::ExprCall(ref callee, _) => {
                match callee.node {
                    Expr_::ExprPath(ref qpath) => {
                        let def = cx.tables.qpath_def(qpath, callee.hir_id);
                        match def {
                            Def::Fn(def_id) => { Some (def_id) }
                            Def::Method(def_id) => { Some (def_id) }
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
            Expr_::ExprMethodCall(..) => {
                let def = cx.tables.type_dependent_defs().get(expr.hir_id).cloned();
                match def {
                    Some(hir::def::Def::Method(def_id)) => { Some(def_id) }
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
                    Some (did) => {
                        if did.krate != LOCAL_CRATE {                     
                            self.add_crate(did.krate);
                            self.add_def_id(did);
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
        println!("Local crate name {:?}", cx.tcx.crate_name(LOCAL_CRATE));
        self.external_crates.iter().for_each(
            |krate| {
                println!("===================================================");
                println!("External crate name {:?}",
                         cx.tcx.crate_name(*krate));
                self.external_calls.iter().
                    filter(|def_id| def_id.krate == *krate).
                    for_each (|def_id| {                    
                                  println!("FnSig {:?}", cx.tcx.fn_sig(*def_id));
                    }) ;
            });
    }
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_late_lint_pass(box ExternalCalls{ external_calls: Vec::new(), external_crates: Vec::new() } as LateLintPassObject);
}
