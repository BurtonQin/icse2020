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
use std::collections::HashMap;
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
    fn check_expr(&mut self, _: &LateContext, expr: &Expr) {
//    fn check_stmt(&mut self, _cx: &EarlyContext, stm: &Stmt) {
//        if let StmtKind::Expr(ref expr) = stm.node {
            match expr.node {
                Expr_::ExprCall(ref fname,_) => {
                    match fname.node {
                        Expr_::ExprPath(ref path) => {
                            print_path(path);
                            println!();
                            println!("Call {:?}", fname);
                        }                        
                        _ => {
                            println!("Call {:?}", fname);
                        }
                    }
                }
                Expr_::ExprMethodCall(ref ps,_,_) => {
                    println!("MethodCall {:?}", ps);
                }               
                _ => {}
            }
                
//        }
    }
}

fn print_path(path: &QPath) {
     match path {
         QPath::Resolved(_, path1) => {
             //print!("{:?}", path1);
             print!("{:?}", print::to_string(print::NO_ANN, |s| s.print_path(path1, false)))
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
