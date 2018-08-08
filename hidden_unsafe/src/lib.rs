#![crate_name = "hidden_unsafe"]
#![crate_type = "dylib"]
#![feature(plugin_registrar)]
#![feature(rustc_private)]
#![feature(box_syntax)]
#![feature(macro_at_most_once_rep)]
#![feature(macro_vis_matcher)]
#![feature(extern_prelude)]
#![feature(use_extern_macros)]

#[macro_use]
extern crate rustc;
extern crate rustc_plugin;
extern crate syntax_pos;
extern crate syntax;

use rustc_plugin::Registry;

use rustc::lint::{LateContext,LintPass,LintArray,LateLintPass, LateLintPassObject};

use rustc::hir;
use rustc::hir::Crate;
use rustc::hir::intravisit;

use rustc::mir::visit::Visitor;

use syntax::ast::NodeId;

mod calls;
mod unsafe_traits;

pub struct FnInfo
{
    decl_id: NodeId,
    has_unsafe: bool,
    local_calls: Vec<NodeId>,
    external_calls: Vec<(hir::def_id::CrateNum,String)>,
    unsafe_trait_use: bool,
}

impl FnInfo {
    fn new(hir_id: NodeId) -> Self {
        Self {
            decl_id:hir_id,
            has_unsafe: false,
            local_calls: Vec::new(),
            external_calls: Vec::new(),
            unsafe_trait_use: false
        }
    }

    fn print_fn_info<'a,'tcx>(&self, cx: &LateContext<'a, 'tcx>,
                              span:syntax_pos::Span, item_id: NodeId) {
        let loc = cx.tcx.sess.codemap().lookup_char_pos(span.lo());
        let filename = &loc.file.name;
        //TODO use formatter
        println!("===================================================");
        println!("file: {:?} line {:?} | {:?} | {:?} | {:?}",
                    filename, loc.line,
                    cx.tcx.node_path_str(item_id), self.has_unsafe,
                    item_id
                );
        println!("Local calls:");
        self.print_local_calls(cx);
        self.print_external_calls(cx);
    }

    fn print_local_calls<'a,'tcx>(&self, cx: &LateContext<'a, 'tcx>) {
        for node_id in self.local_calls.iter() {
            println!("{:?} | {:?} ", cx.tcx.node_path_str(*node_id), node_id);
        }
    }

    fn print_external_calls<'a,'tcx>(&self, cx: &LateContext<'a, 'tcx>) {
        let mut external_crates = Vec::new();
        self.external_calls.iter().for_each(
            |elt|
                if !external_crates.iter().any(
                    |crate_num| *crate_num == elt.0) {
                    external_crates.push(elt.0)
                }
        );

        external_crates.iter().for_each(
            |krate| {
                println!("External crate name {:?}",
                         cx.tcx.crate_name(*krate));
                self.external_calls.iter().
                    filter(|elt| elt.0 == *krate).
                    for_each (|elt| {
                        println!("{:?}", elt.1);
                    }) ;
            });
    }

    fn push_external_call(&mut self, krate: hir::def_id::CrateNum, func:String) -> () {
        let found = self.external_calls.iter().any(
            |elt| elt.1 == func && elt.0 == krate
        );
        if !found {
            self.external_calls.push((krate,func));
        }
    }

    fn push_local_call(&mut self, node_id: NodeId) -> () {
        let found = self.local_calls.iter().any(
            |elt| *elt == node_id
        );
        if !found {
            self.local_calls.push(node_id);
        }
    }

}


struct HiddenUnsafe {
     data: Vec<FnInfo>,
}

impl HiddenUnsafe {

    pub fn new() -> Self {
        Self{data: Vec::new()}
    }

    pub fn push_fn_info<'a,'tcx>(&mut self, cx: &'a LateContext<'a, 'tcx>,
                                 node_id: NodeId, body: &'tcx hir::Body) {
        let mut fn_info = FnInfo::new(node_id);
        let mut visitor = UnsafeBlocks{ map: &cx.tcx.hir, has_unsafe: false};
        hir::intravisit::walk_body(&mut visitor, body);
        fn_info.has_unsafe = visitor.has_unsafe;
        self.data.push(fn_info);
    }

    pub fn print_results<'a,'tcx>(&mut self, cx: &LateContext<'a, 'tcx>) {
        for fn_info in self.data.iter() {
            let fn_node = cx.tcx.hir.get(fn_info.decl_id);
            match fn_node {
                ::hir::map::Node::NodeItem(item) => {
                    if let hir::ItemKind::Fn(ref _fn_decl, ref fn_header, _, _) = item.node {
                        if let hir::Unsafety::Normal = fn_header.unsafety {
                            fn_info.print_fn_info(cx, item.span, item.id);
                        }
                    }
                }
                ::hir::map::Node::NodeImplItem(impl_item) => {
                    if let hir::ImplItemKind::Method(ref method_sig,_) = impl_item.node {
                        if let hir::Unsafety::Normal = method_sig.header.unsafety {
                            fn_info.print_fn_info(cx, impl_item.span, impl_item.id);
                        }
                    }
                }
                _ => {println!("node NOT handled {:?}", fn_node);}
            }

        }
    }

    // TODO change this to an efficient algorithm
    pub fn propagate_unsafe(&mut self) {
        let mut changes= true;
        let mut with_unsafe = Vec::new();
        { // to pass borrow checker
            for fn_info in &self.data {
                if fn_info.has_unsafe {
                    with_unsafe.push(fn_info.decl_id);
                }
            }
        }
        while changes {
            changes = false;
            for fn_info in &mut self.data {
                if !fn_info.has_unsafe {
                    if (&fn_info.local_calls).into_iter().any(
                        |call_id|
                            with_unsafe.iter().any(
                                |x| *x == *call_id
                            )
                        ) {
                        with_unsafe.push(fn_info.decl_id);
                        fn_info.has_unsafe = true;
                        changes = true;
                    }
                }
            }
        }
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
        for fn_info in &mut self.data {
            let body_owner_kind = cx.tcx.hir.body_owner_kind(fn_info.decl_id);
            if let hir::BodyOwnerKind::Fn = body_owner_kind {
                let owner_def_id = cx.tcx.hir.local_def_id(fn_info.decl_id);
                let mut mir = cx.tcx.mir_validated(owner_def_id);
                {
                    let mut calls_visitor = calls::Calls::new(cx, fn_info);
                    calls_visitor.visit_mir(&mut mir.borrow());
                }
                {
                    let mut unsafe_trait_visitor =
                        unsafe_traits::SafeMethodsInUnsafeTraits::new( cx, fn_info);
                    unsafe_trait_visitor.visit_mir(&mut mir.borrow());
                }
            }
        }
        self.propagate_unsafe();
        self.print_results(cx);
    }

    fn check_body(&mut self, cx: &LateContext<'a, 'tcx>, body: &'tcx hir::Body) {
        //need to find fn/method declaration of this body
        let owner_def_id = cx.tcx.hir.body_owner_def_id( body.id() );
        if let Some (owner_node_id) = cx.tcx.hir.as_local_node_id(owner_def_id) {
            let owner_node = cx.tcx.hir.get(owner_node_id);
            match owner_node {
                hir::map::Node::NodeItem(item) => {
                    if let hir::ItemKind::Fn(ref _fn_decl, ref fn_header, _, _) = item.node {
                        if let hir::Unsafety::Normal = fn_header.unsafety {
                            self.push_fn_info(cx, owner_node_id, body);
                        }
                    } else {
                        println!("Body owner node type NOT handled: {:?}", item);
                    }
                }
                hir::map::Node::NodeImplItem(ref impl_item) => {
                    if let ::hir::ImplItemKind::Method(ref method_sig,..) = impl_item.node {
                        if let hir::Unsafety::Normal = method_sig.header.unsafety {
                            self.push_fn_info(cx, owner_node_id, body);
                        }
                    } else {
                        println!("Impl Item Kind NOT handled {:?}", impl_item.node);
                    }
                }
                hir::map::Node::NodeExpr(ref expr) => {
                    if let hir::ExprKind::Closure(..) = expr.node {
                        // nothing to do - this is not a stand alone function
                        // any unsafe in this body will be processed by the enclosing function or method
                    } else {
                        println!("Body owner node NOT handled: {:?}", owner_node);
                    }
                }
                hir::map::Node::NodeAnonConst(ref _anon_const) => {
                    // nothing to do - this is not a stand alone function
                    // any unsafe in this body will be processed by the enclosing function or method
                }
                _ => {
                    println!("Body owner node NOT handled: {:?}", owner_node);
                }
            }
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
            hir::BlockCheckMode::UnsafeBlock(_unsafe_source) => {
                self.has_unsafe = true;
            }
            hir::BlockCheckMode::PushUnsafeBlock(unsafe_source) => {
                println!("hir::BlockCheckMode::PushUnsafeBlock {:?}", unsafe_source);
            }
            hir::BlockCheckMode::PopUnsafeBlock(unsafe_source) => {
                println!("hir::BlockCheckMode::PopUnsafeBlock {:?}", unsafe_source);
            }
        }
        hir::intravisit::walk_block(self, b);
    }

    fn nested_visit_map<'this>(&'this mut self) -> intravisit::NestedVisitorMap<'this, 'tcx> {
        intravisit::NestedVisitorMap::All(self.map)
    }
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_late_lint_pass(box HiddenUnsafe::new() as LateLintPassObject);
}

