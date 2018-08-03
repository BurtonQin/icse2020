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
use rustc::hir::{FnDecl,Body,Crate,BodyId};
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
    fn new(decl: &'tcx FnDecl, map: &'a hir::map::Map<'tcx>) -> Self {
        Self{ decl: decl, has_unsafe: false, local_calls: Vec::new(), map: map }
    }
}

impl<'a, 'tcx> hir::intravisit::Visitor<'tcx> for FnInfo<'a, 'tcx> {

    fn visit_block<'v>(&mut self, b: &'v hir::Block) {
        //TODO
    }

    fn nested_visit_map<'this>(&'this mut self) -> intravisit::NestedVisitorMap<'this, 'tcx> {
        intravisit::NestedVisitorMap::OnlyBodies(self.map) //TODO maybe All
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
        match kind {
            hir::intravisit::FnKind::ItemFn(name, _, unsafety, _, _, _, _) => {
                match unsafety {
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
            hir::intravisit::FnKind::Method(_, sig, _, _) => {
                //TODO
            },
            hir::intravisit::FnKind::Closure(_) => {}
        }
    }


    fn nested_visit_map<'this>(&'this mut self) -> intravisit::NestedVisitorMap<'this, 'tcx> {
        intravisit::NestedVisitorMap::OnlyBodies(self.map) // TODO maybe None?
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