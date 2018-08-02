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

use rustc::lint::{LateContext,LintPass,LintArray,LateLintPass};

use rustc::hir;
use rustc::hir::{FnDecl,Body};
use rustc::hir::intravisit;
use rustc::hir::intravisit::FnKind;

use rustc::hir::map::{Node,NodeItem};

use syntax_pos::Span;

use syntax::ast;
use syntax::ast::NodeId;

struct FnInfo<'a, 'tcx: 'a>
{
    cx: &'a LateContext<'a, 'tcx>,
    full_path_name: String,
    has_unsafe: bool,
    local_calls: Vec<String>,
}

impl<'a, 'tcx> hir::intravisit::Visitor<'tcx> for FnInfo<'a, 'tcx> {

    fn visit_block<'v>(&mut self, b: &'v hir::Block) {
        //TODO
    }

    fn nested_visit_map<'this>(&'this mut self) -> intravisit::NestedVisitorMap<'this, 'tcx> {
        intravisit::NestedVisitorMap::None
    }
}

struct HiddenUnsafe<'a, 'tcx: 'a>
{
    data: Vec<FnInfo<'a,'tcx>>,
}

declare_lint!(pub EXERNAL_CALLS, Allow, "Collect external function calls");

impl <'a, 'tcx>LintPass for HiddenUnsafe<'a, 'tcx>{
    fn get_lints(&self) -> LintArray {
        lint_array!(EXERNAL_CALLS)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for HiddenUnsafe<'a, 'tcx> {
    fn check_fn(
        &mut self,
        cx: &LateContext<'a, 'tcx>,
        kind: intravisit::FnKind<'tcx>,
        _decl: &'tcx hir::FnDecl,
        body: &'tcx hir::Body,
        span: Span,
        nodeid: ast::NodeId,
    ) {
        let is_impl = if let Some(NodeItem(item)) = cx.tcx.hir.find(cx.tcx.hir.get_parent_node(nodeid)) {
            //matches!(item.node, hir::ItemImpl(_, _, _, _, Some(_), _, _))
        } else {
            //false
        };
    }
}
