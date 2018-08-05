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
use rustc::hir::{FnDecl,Body,Crate,BodyId,Mod,Item,ItemId,Expr,Stmt,HirId};
use rustc::hir::intravisit;
use rustc::hir::intravisit::FnKind;

use rustc::hir::map::{Node,NodeItem,Map};

use syntax_pos::Span;

use syntax::ast;
use syntax::ast::NodeId;

struct FnInfo
{
    decl_id: NodeId,
    has_unsafe: bool,
    local_calls: Vec<HirId>,
}

impl FnInfo {
    fn new(hir_id: NodeId) -> Self {
        Self { decl_id:hir_id, has_unsafe: false, local_calls: Vec::new() }
    }
}

struct HiddenUnsafe {
    data: Vec<FnInfo>,
}

impl HiddenUnsafe {

    pub fn new() -> Self {
        Self{data: Vec::new()}
    }
}

declare_lint!(pub HIDDEN_UNSAFE, Allow, "Functions using hidden unsafe");

impl <'a, 'tcx>LintPass for HiddenUnsafe{
    fn get_lints(&self) -> LintArray {
        lint_array!(HIDDEN_UNSAFE)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for HiddenUnsafe {

    fn check_crate_post(&mut self, cx: &LateContext<'a, 'tcx>, _: &'tcx Crate) {
        for i in self.data.iter() {
            let fn_node = cx.tcx.hir.get(i.decl_id);
            match fn_node {
                ::hir::map::Node::NodeItem(item) => {
                    if let hir::ItemKind::Fn(ref fn_decl, ref fn_header, _, _) = item.node {
                        if let hir::Unsafety::Normal = fn_header.unsafety {
                            let loc = cx.tcx.sess.codemap().lookup_char_pos(item.span.lo());
                            let filename = &loc.file.name;
                            //TODO use formatter
                            println!("file: {:?} line {:?} | {:?} | {:?}", filename, loc.line,
                                     cx.tcx.node_path_str(item.id), i.has_unsafe);
                        }
                    }
                }
                ::hir::map::Node::NodeImplItem(impl_item) => {
                    if let hir::ImplItemKind::Method(ref method_sig,_) = impl_item.node {
                        if let hir::Unsafety::Normal = method_sig.header.unsafety {
                            //println!("{:?} {:?}", cx.tcx.node_path_str(impl_item.id), i.has_unsafe);
                            let loc = cx.tcx.sess.codemap().lookup_char_pos(impl_item.span.lo());
                            let filename = &loc.file.name;
                            //TODO use formatter
                            println!("file: {:?} line {:?} | {:?} | {:?}", filename, loc.line,
                                     cx.tcx.node_path_str(impl_item.id), i.has_unsafe);
                        }
                    }
                }
                _ => {println!("node NOT handled {:?}", fn_node);}
            }

        }
    }

    fn check_body(&mut self, cx: &LateContext<'a, 'tcx>, body: &'tcx hir::Body) {
        //need to find fn/method declaration of this body
        let owner_def_id = cx.tcx.hir.body_owner_def_id( body.id() );
        if let Some (owner_node_id) = cx.tcx.hir.as_local_node_id(owner_def_id) {
            let owner_node = cx.tcx.hir.get(owner_node_id);
            match owner_node {
                hir::map::Node::NodeItem(item) => {
                    match item.node {
                        hir::ItemKind::Fn(ref fn_decl, ref fn_header, _, _) => {
                            let mut fn_info = FnInfo::new(owner_node_id);
                            let mut visitor = UnsafeBlocks{ map: &cx.tcx.hir, has_unsafe: false};
                            hir::intravisit::walk_body(&mut visitor, body);
                            fn_info.has_unsafe = visitor.has_unsafe;
                            self.data.push(fn_info);
                        }
                        _ => { println!("Body owner node type NOT handled: {:?}", item); }
                    }
                }
                hir::map::Node::NodeImplItem(impl_item) => {
                    match impl_item.node {
                        ::hir::ImplItemKind::Method(..) => {
                            let mut fn_info = FnInfo::new(owner_node_id);
                            let mut visitor = UnsafeBlocks{ map: &cx.tcx.hir, has_unsafe: false};
                            hir::intravisit::walk_body(&mut visitor, body);
                            self.data.push(fn_info);
                        }
                        _ => {println!("Impl Item Kind NOT handled {:?}", impl_item.node);}
                    }
                }
                hir::map::Node::NodeExpr(ref expr) => {
                    if let hir::ExprKind::Closure(..) = expr.node {
                    } else {
                        println!("Body owner node NOT handled: {:?}", owner_node);
                    }
                }
                _ => {
                    println!("Body owner node NOT handled: {:?}", owner_node);
                }
            }
        }
    }

    fn check_expr(&mut self, cx: &LateContext, expr: &Expr) {
        match expr.node {
            ::hir::ExprKind::Call(ref fn_expr, ref _args) => {
                if let ::hir::ExprKind::Path(ref qpath) = fn_expr.node {

                }
            }
            ::hir::ExprKind::MethodCall(ref path_segment, _, ref arguments) => {
                //mark unsafe if call of a
            }
            _ => {}

        }
    }
}

struct UnsafeBlocks<'tcx> {
    map: &'tcx hir::map::Map<'tcx>,
    has_unsafe: bool,
}

impl<'a, 'tcx> hir::intravisit::Visitor<'tcx> for UnsafeBlocks<'tcx> {
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
        intravisit::NestedVisitorMap::All(self.map)
    }
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_late_lint_pass(box HiddenUnsafe::new() as LateLintPassObject);
}