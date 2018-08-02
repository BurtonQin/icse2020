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

use syntax_pos::Span;

use syntax::ast;
use syntax::ast::NodeId;

struct FnInfo
{
    full_path_name: String,
    has_unsafe: bool,
    local_calls: Vec<String>,
}

struct HiddenUnsafe
{
    data: Vec<FnInfo>,
}

declare_lint!(pub EXERNAL_CALLS, Allow, "Collect external function calls");

impl LintPass for HiddenUnsafe{
    fn get_lints(&self) -> LintArray {
        lint_array!(EXERNAL_CALLS)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for HiddenUnsafe {
    fn check_fn(
        &mut self,
        cx: &LateContext<'a, 'tcx>,
        kind: intravisit::FnKind<'tcx>,
        _decl: &'tcx hir::FnDecl,
        body: &'tcx hir::Body,
        span: Span,
        nodeid: ast::NodeId,
    ) {
    }
}
