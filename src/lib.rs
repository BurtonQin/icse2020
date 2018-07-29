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
use rustc::lint::LintContext;
use rustc::lint::LintArray;
use rustc::hir::def_id::LOCAL_CRATE;
use rustc::hir::def::Def;
use rustc::hir;

struct ExternalCalls;
//{
    //crate_name: String;
    //external_calls: HashMap<,>;
//}

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
                        if let Def::Fn(def_id) = def {
                            Some(def_id)
                        } else {  // `Def::Local` if it was a closure, for which we
                            println!("Not Def::Fn {:?}", def);
                            None  // do not currently support must-use linting
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
                        if did.krate == LOCAL_CRATE {
                            println!("Local crate name {:?}",
                                     cx.tcx.crate_name(did.krate));
                        } else {
                            println!("External crate name {:?}",
                                     cx.tcx.crate_name(did.krate));
                        }
                
                    None => {
                        // TODO fix this
                        let mut msg = format!("def id not found {:?}", expr.node);
                        let mut err = cx.struct_span_lint(EXERNAL_CALLS, expr.span, &msg);
                        err.emit();
                    }
                }
            }
            _ => {}
        }
            
    }
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_late_lint_pass(box ExternalCalls as LateLintPassObject);
}
