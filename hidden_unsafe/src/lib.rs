#![crate_name = "hidden_unsafe"]
#![crate_type = "dylib"]
#![feature(plugin_registrar)]
#![feature(rustc_private)]
#![feature(box_syntax)]
#![feature(macro_at_most_once_rep)]
#![feature(macro_vis_matcher)]

#[macro_use]
extern crate rustc;
extern crate rustc_plugin;
extern crate syntax_pos;
extern crate syntax;

use rustc_plugin::Registry;

use rustc::lint::{LateContext,LintPass,LintArray,LateLintPass, LateLintPassObject};

use rustc::hir;
use rustc::hir::{FnDecl,Body,Crate,BodyId,Mod,Item,ItemId,Expr,Stmt};
use rustc::hir::intravisit;
use rustc::hir::intravisit::FnKind;

use rustc::hir::map::{Node,NodeItem,Map};

use syntax_pos::Span;

use syntax::ast;
use syntax::ast::NodeId;

struct FnInfo<'a, 'tcx: 'a>
{
    decl: &'tcx FnDecl,
    has_unsafe: bool,
    local_calls: Vec<String>,
    map: &'a hir::map::Map<'tcx>,
}

impl <'a, 'tcx: 'a> FnInfo<'a, 'tcx> {
    fn new(decl: &'tcx FnDecl, m: &'a hir::map::Map<'tcx>) -> Self {
        Self{ decl: decl, has_unsafe: false, local_calls: Vec::new(), map: m }
    }
}

impl<'a, 'tcx> hir::intravisit::Visitor<'tcx> for FnInfo<'a, 'tcx> {

    fn visit_expr(&mut self, ex: &'tcx Expr) {
        match ex.node {
            ::hir::ExprKind::MethodCall(ref segment, _span , ref arguments) => {
                println!("MethodCall expr {:?}", ex);
                println!("MethodCall segment {:?}", segment);
                println!("MethodCall arguments {:?}", arguments);
                // check if it's unsafe impl of unsafe trait
                // Want a DefKey

                let obj_ex = &arguments[0];

                println!("Obj def path {:?}", obj_ex.node);
                match obj_ex.node {
                    ::hir::ExprKind::Path(ref qpath) => {
                        match qpath {
                            ::hir::QPath::Resolved(_, path) => {
                                print!("path def {:?}", path.def);
                                match path.def {
                                    hir::def::Def::Local(node_id) => {
                                        println!("get {:?}", self.map.get(node_id));
                                        match (self.map.get(node_id)) {
                                            hir::map::Node::NodeBinding(pat) => {
                                                println!("pat  kind {:?}", pat.node);
                                            }
                                            _ => {}
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }

//                match  {
//                    Some (def) => {println!("Def {:?}",def)}
//                    None => {println!("None")}
//                }
            }
            _ => {}
        }
        hir::intravisit::walk_expr(self, ex)
    }

    fn visit_block(&mut self, b: &'tcx hir::Block) {
        match b.rules {
            hir::BlockCheckMode::DefaultBlock => {}
            hir::BlockCheckMode::UnsafeBlock(unsafe_source) => {
                self.has_unsafe = true;
            }
            hir::BlockCheckMode::PushUnsafeBlock(unsafe_source) => {
                println!("hir::BlockCheckMode::PushUnsafeBlock {:?}", unsafe_source);
            }
            hir::BlockCheckMode::PopUnsafeBlock(unsafe_source) => {
                println!("hir::BlockCheckMode::PopUnsafeBlock {:?}", unsafe_source);
            }
        }
        hir::intravisit::walk_block(self, b)
    }

    fn nested_visit_map<'this>(&'this mut self) -> intravisit::NestedVisitorMap<'this, 'tcx> {
        //intravisit::NestedVisitorMap::All(self.map) //TODO maybe All or None (?)
        intravisit::NestedVisitorMap::None
    }
}

struct CrateInfo <'a, 'tcx: 'a> {
    data: Vec<FnInfo<'a, 'tcx>>,
    ffi: Vec<rustc::hir::ForeignItem>, //TODO understand when FFI items are visited (intravisit.rs)
    map: &'a hir::map::Map<'tcx>,
}

impl <'a, 'tcx: 'a> CrateInfo<'a, 'tcx> {
    fn new(map: &'a hir::map::Map<'tcx>) -> Self {
        Self{ data: Vec::new(), map: map, ffi: Vec::new() }
    }
}

impl<'a, 'tcx> hir::intravisit::Visitor<'tcx> for CrateInfo<'a, 'tcx> {

    fn visit_fn( &mut self, kind: hir::intravisit::FnKind<'tcx>,
                 fn_decl: &'tcx hir::FnDecl, body_id: hir::BodyId,
                 _: syntax_pos::Span, node_id: syntax::ast::NodeId) {
        println!("visit_fn {:?}", fn_decl);
        match kind {
            hir::intravisit::FnKind::ItemFn(name, _generics, fn_header, _visibility, _attributes) => {
                match fn_header.unsafety {
                    hir::Unsafety::Unsafe => {
                        // Nothing to do for now
                        // Later detect reasons for unsafety in the body
                    }
                    hir::Unsafety::Normal => {
                        let mut visitor = FnInfo::new(fn_decl, self.map);
                        hir::intravisit::walk_body(&mut visitor, self.map.body(body_id));
                        if visitor.has_unsafe {
                            self.data.push(visitor);
                        }
                    }
                }
            }
            hir::intravisit::FnKind::Method(_ident, sig, _visibility, _attributes) => {
                //TODO
            },
            hir::intravisit::FnKind::Closure(_attributes) => {}
        }
    }

    //TODO overwrite methods for other items to do nothing

    fn nested_visit_map<'this>(&'this mut self) -> intravisit::NestedVisitorMap<'this, 'tcx> {
        intravisit::NestedVisitorMap::All(self.map) // TODO maybe None?
    }
}

struct HiddenUnsafe {}

impl HiddenUnsafe {
    pub fn new() -> Self {
        Self{}
    }
}

declare_lint!(pub HIDDEN_UNSAFE, Allow, "Functions using hidden unsafe");

impl <'a, 'tcx>LintPass for HiddenUnsafe{
    fn get_lints(&self) -> LintArray {
        lint_array!(HIDDEN_UNSAFE)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for HiddenUnsafe {

    fn check_crate(&mut self, cx: &LateContext<'a, 'tcx>, krate: &'tcx Crate) {
        let mut visitor = CrateInfo::new(&cx.tcx.hir);
        hir::intravisit::walk_crate(&mut visitor, krate);
    }

    fn check_crate_post(&mut self, _: &LateContext<'a, 'tcx>, _: &'tcx Crate) {
        //TODO print results
    }

}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_late_lint_pass(box HiddenUnsafe::new() as LateLintPassObject);
}