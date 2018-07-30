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

use rustc::lint::{LateLintPass, LateLintPassObject, LateContext};
use rustc::lint::{LintPass, LintArray};

use syntax::ptr::P;


struct ExternalCalls
{
    external_crates: Vec<CrateNum>,
    external_calls: Vec<DefPath>,
}

impl ExternalCalls {
    
    fn add_crate(&mut self, krate: CrateNum) -> () {
        let found = self.external_crates.iter().any(|elt| *elt == krate);
        if !found {
            self.external_crates.push(krate);
        }
    }

    fn add_def_id(&mut self, def_path: DefPath) -> () {
        let found = self.external_calls.iter().any(|elt| *elt == def_id);
        if !found {
            self.external_calls.push(def_id);
        }
    }

    fn mk_ty_string(&self, ty:&P<Ty>) -> String {
        let mut res = String::new();
        match ty.node {
            hir::Ty_::TySlice(ref ty) => {
                res.push_str("[");
                res.push_str(&self.mk_ty_string(&ty));
                res.push_str("]");
            }
            hir::Ty_::TyPtr(ref mt) => {
                res.push_str("*");
                match mt.mutbl {
                    hir::MutMutable => res.push_str("mut"),
                    hir::MutImmutable => res.push_str("const"),
                }
                res.push_str(&self.mk_ty_string(&mt.ty));
            }
            hir::Ty_::TyRptr(ref lifetime, ref mt) => {
                res.push_str("&");
                res.push_str("TODO");
                //self.print_opt_lifetime(lifetime);
                //self.print_mt(mt)?;
            }
            hir::Ty_::TyNever => {
                res.push_str("!");
            },
            hir::Ty_::TyTup(ref elts) => {
                res.push_str("TODO");
                // self.popen()?;
                // self.commasep(Inconsistent, &elts[..], |s, ty| s.print_type(&ty))?;
                // if elts.len() == 1 {
                //     self.s.word(",")?;
                // }
                // self.pclose()?;
            }
            hir::Ty_::TyBareFn(ref f) => {
                res.push_str("TODO");
                // self.print_ty_fn(f.abi, f.unsafety, &f.decl, None, &f.generic_params,
                //                  &f.arg_names[..])?;
            }
            hir::Ty_::TyPath(ref qpath) => {
                res.push_str(&self.mk_qpath_string(qpath));
            }
            hir::Ty_::TyTraitObject(ref bounds, ref lifetime) => {
                res.push_str("TraitObject - TODO");
                // let mut first = true;
                // for bound in bounds {
                //     if first {
                //         first = false;
                //     } else {
                //         self.nbsp()?;
                //         self.word_space("+")?;
                //     }
                //     self.print_poly_trait_ref(bound)?;
                // }
                // if !lifetime.is_elided() {
                //     self.nbsp()?;
                //     self.word_space("+")?;
                //     self.print_lifetime(lifetime)?;
                // }
            }
            hir::Ty_::TyArray(ref ty, ref length) => {
                res.push_str("[");
                res.push_str(&self.mk_ty_string(&ty));
                //res.push_str("; ");
                //self.print_anon_const(length)?;
                res.push_str("]");
            }
            hir::Ty_::TyTypeof(ref e) => {
                res.push_str("typeof(");
                //self.print_anon_const(e)?;
                res.push_str(")");
            }
            hir::Ty_::TyInfer => {
                res.push_str("_");
            }
            hir::Ty_::TyErr => {
                res.push_str("?");
            }
            hir::Ty_::TyImplTraitExistential(_, _) => {
                res.push_str("TODO");
            }
        }
        res
    }

    fn mk_path_segment_string(&self, segment: &hir::PathSegment) -> String {
        (String::from_utf8(segment.name.as_str().as_bytes().to_vec()).unwrap())
    }

    fn mk_path_string(&self, path: &hir::Path) -> String {
        let mut res = String::new();
        for (i, segment) in path.segments.iter().enumerate() {
            if i > 0 {
                res.push_str("::");
            }
            // if segment.ident.name != keywords::CrateRoot.name() &&
            //    segment.ident.name != keywords::DollarCrate.name() {
            res.push_str(&self.mk_path_segment_string(segment));
            // segment.with_generic_args(|generic_args| {
            //     self.print_generic_args(generic_args, segment.infer_types,
            //                             colons_before_params)
            // })?;
            // }
        }
        res
    }

    fn mk_qpath_string(&self, qpath: &hir::QPath) -> String {        
        match *qpath {
            hir::QPath::Resolved(None, ref path) => {
                let mut res = String::new();
                res.push_str("QPath::Resolved no self:");
                res.push_str(&self.mk_path_string(path));
                res
            }
            hir::QPath::Resolved(Some(ref qself), ref path) => {
                let mut res = String::new();
                res.push_str("QPath::Resolved with self:");
                res.push_str("<");
                res.push_str(&self.mk_ty_string(qself));
                res.push_str(" as ");
                
                for (i, segment) in path.segments[..path.segments.len() - 1].iter().enumerate() {
                    if i > 0 {
                        res.push_str("::");
                    }
                    // if segment.ident.name != keywords::CrateRoot.name() &&
                    //     segment.ident.name != keywords::DollarCrate.name() {
                    res.push_str(&self.mk_path_segment_string(segment));
                    // segment.with_generic_args(|generic_args| {
                    //     self.print_generic_args(generic_args,
                    //                             segment.infer_types,
                    //                             colons_before_params)
                    // })?;
                // }
                }

                res.push_str(">");
                res.push_str("::");
                let item_segment = path.segments.last().unwrap();
                //self.print_ident(item_segment.ident)?;
                res.push_str(&self.mk_path_segment_string(item_segment));
                // item_segment.with_generic_args(|generic_args| {
                //     self.print_generic_args(generic_args,
                //                             item_segment.infer_types,
                //                             colons_before_params)
                //  })
                res
            }
            hir::QPath::TypeRelative(ref qself, ref item_segment) => {
                let mut res = String::new();
                res.push_str("QPath::TypeRelative no self:");
                res.push_str("<");
                res.push_str(&self.mk_ty_string(qself));
                res.push_str(">");
                res.push_str("::");
                //self.print_ident(item_segment.ident)?;
                res.push_str(&self.mk_path_segment_string(item_segment));
                // item_segment.with_generic_args(|generic_args| {
                //     self.print_generic_args(generic_args,
                //                             item_segment.infer_types,
                //                             colons_before_params)
                // })
                res
            }
        }
    }

    fn mk_method_call_string(&self, ps:&hir::PathSegment, args: &hir::HirVec<Expr>, cx: &LateContext) -> String {
        let res = String::new();
        let obj_expr = &args[0]; // the object the method is called on
        //need a def id as a param.        
        match obj_expr.node {
            Expr_::ExprPath(ref qpath) => {
                let def = cx.tables.qpath_def(qpath, obj_expr.hir_id);
                match def {
                    hir::def::Def::Mod(did) | hir::def::Def::Struct(did) |
                    hir::def::Def::Union(did) | hir::def::Def::Enum(did) |
                    hir::def::Def::Variant(did) | hir::def::Def::Trait(did) |
                    hir::def::Def::TyAlias(did) | hir::def::Def::TyForeign(did) |
                    hir::def::Def::AssociatedTy(did) | hir::def::Def::TyParam(did) |
                    hir::def::Def::Fn(did) | hir::def::Def::Const(did) |
                    hir::def::Def::Static(did,_) | hir::def::Def::StructCtor(did, _) |
                    hir::def::Def::VariantCtor(did, _) | hir::def::Def::Method(did) |
                    hir::def::Def::AssociatedConst(did) |
                    hir::def::Def::Macro(did, _) | hir::def::Def::GlobalAsm(did) =>
                    { print!("def of qpath {:?}", cx.tcx.type_of(did)); }
                    hir::def::Def::SelfTy(..) => {  println!("mk_method_call_string is SelfTy"); }
                    hir::def::Def::Local(node_id) => {  println!("Local"); }
                    hir::def::Def::Upvar(..) => {  println!("mk_method_call_string is Upvar"); }
                    hir::def::Def::Label(..) => {  println!("mk_method_call_string is Local"); }
                    _ => {  println!("mk_method_call_string Def not implemented"); }
                }
            }
            _ => {  println!("mk_method_call_string Expr_ not implemented"); }
        }
        res
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
                        println!("QPath {:?}", self.mk_qpath_string(qpath));
                        let def = cx.tables.qpath_def(qpath, callee.hir_id);                        
                        match def {
                            Def::Fn(def_id) => { println!("ExprCall callee {:?}", cx.tcx.def_path(def_id)); Some (def_id) }
                            Def::Method(def_id) => { println!("ExprCall callee {:?}", cx.tcx.def_path(def_id)); Some (def_id) }
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
                match def {
                    Some(hir::def::Def::Method(def_id)) => {
                        println!("MethodCall def_id {:?}", cx.tcx.def_path(def_id));
                        // let obj_expr = &args[0];
                        // //need a def id as a param.
                        // let ty = cx.tcx.type_of(def_id);
                        // match obj_expr.node {
                        //     Expr_::ExprPath(ref qpath) => {
                        //         match qpath {
                        //             QPathResolved(_,path) =>
                        //         print!("{:?}.", self.mk_qpath_string(qpath));
                        //     }
                        //     _ =>{
                        //         println!("First not a match");
                        //     }
                        // }
                        println!("ExprMethodCall {:?}", self.mk_method_call_string(ps,args,cx));
                        Some(def_id)
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
                        // TODO print full path
                        println!("FnSig {:?}", cx.tcx.fn_sig(*def_id));
                        //println!("{:?}", cx.tcx.def_symbol_name(*def_id));
                    }) ;
            });
    }

}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_late_lint_pass(box ExternalCalls{ external_calls: Vec::new(), external_crates: Vec::new() } as LateLintPassObject);
}
