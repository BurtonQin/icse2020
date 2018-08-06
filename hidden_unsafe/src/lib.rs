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
use rustc::hir::print;

use rustc::hir::map::{Node,NodeItem,Map};

use syntax_pos::Span;

use syntax::ast;
use syntax::ast::NodeId;

struct FnInfo
{
    decl_id: NodeId,
    has_unsafe: bool,
    local_calls: Vec<NodeId>,
    external_calls: Vec<(hir::def_id::CrateNum,String)>,
}

impl FnInfo {
    fn new(hir_id: NodeId) -> Self {
        Self {
            decl_id:hir_id,
            has_unsafe: false,
            local_calls: Vec::new(),
            external_calls: Vec::new()
        }
    }

    fn print_fn_info<'a,'tcx>(&self, cx: &LateContext<'a, 'tcx>,
                              span:syntax_pos::Span, item_id: NodeId) {
        let loc = cx.tcx.sess.codemap().lookup_char_pos(span.lo());
        let filename = &loc.file.name;
        //TODO use formatter
        println!("===================================================");
        println!("file: {:?} line {:?} | {:?} | {:?}",
                 filename, loc.line,
                 cx.tcx.node_path_str(item_id), self.has_unsafe);
        println!("Local calls:");
        self.print_local_calls(cx);
        self.print_external_calls(cx);
    }

    fn print_local_calls<'a,'tcx>(&self, cx: &LateContext<'a, 'tcx>) {
        for node_id in self.local_calls.iter() {
            println!("{:?}", cx.tcx.node_path_str(*node_id));
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
        //self.add_crate(krate);
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

    pub fn push_local_call<'a,'tcx>(&mut self, cx: &'a LateContext<'a, 'tcx>,
                                    node_id: NodeId, body: &'tcx hir::Body) {
        let mut fn_info = FnInfo::new(node_id);
        let mut visitor = UnsafeBlocks{ map: &cx.tcx.hir, has_unsafe: false};
        hir::intravisit::walk_body(&mut visitor, body);
        fn_info.has_unsafe = visitor.has_unsafe;
        {
            let mut fn_visitor = Calls { cx, fn_info: &mut fn_info };
            hir::intravisit::walk_body(&mut fn_visitor, body);
        }
        self.data.push(fn_info);
    }

    pub fn print_results<'a,'tcx>(&mut self, cx: &LateContext<'a, 'tcx>) {
        for fn_info in self.data.iter() {
            let fn_node = cx.tcx.hir.get(fn_info.decl_id);
            match fn_node {
                ::hir::map::Node::NodeItem(item) => {
                    if let hir::ItemKind::Fn(ref fn_decl, ref fn_header, _, _) = item.node {
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
                    let mut local_change = false; // to pass borrow checker
                    for call_id in (&fn_info.local_calls).into_iter().filter(
                        |call_id|
                            with_unsafe.iter().any(
                                |x| x == *call_id
                            )
                    ) {
                        fn_info.has_unsafe = true;
                        local_change = true;
                        changes = true;
                    }
                    if local_change {
                        with_unsafe.push(fn_info.decl_id);
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
                    if let hir::ItemKind::Fn(ref fn_decl, ref fn_header, _, _) = item.node {
                        if let hir::Unsafety::Normal = fn_header.unsafety {
                            self.push_local_call(cx, owner_node_id, body);
                        }
                    } else {
                        println!("Body owner node type NOT handled: {:?}", item);
                    }
                }
                hir::map::Node::NodeImplItem(ref impl_item) => {
                    if let ::hir::ImplItemKind::Method(ref method_sig,..) = impl_item.node {
                        if let hir::Unsafety::Normal = method_sig.header.unsafety {
                            self.push_local_call(cx, owner_node_id, body);
                        }
                    } else {
                        println!("Impl Item Kind NOT handled {:?}", impl_item.node);
                    }
                }
                hir::map::Node::NodeExpr(ref expr) => {
                    if let hir::ExprKind::Closure(..) = expr.node {
                    } else {
                        println!("Body owner node NOT handled: {:?}", owner_node);
                    }
                }
                hir::map::Node::NodeAnonConst(ref anon_const) => {
                    // TODO
                    println!("AnonConst {:?} in body {:?}", anon_const, body);
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
        hir::intravisit::walk_block(self, b);
    }

    fn nested_visit_map<'this>(&'this mut self) -> intravisit::NestedVisitorMap<'this, 'tcx> {
        intravisit::NestedVisitorMap::All(self.map)
    }
}


struct Calls<'a, 'tcx: 'a> {
    cx: &'a LateContext<'a,'tcx>,
    fn_info: &'a mut FnInfo,
}


impl <'a, 'tcx> Calls<'a, 'tcx>{
    fn process_local_call(&mut self, def_id: &hir::def_id::DefId) -> () {
        let node_id = self.cx.tcx.hir.def_index_to_node_id(def_id.index);
        self.fn_info.push_local_call(node_id);
    }
}

impl<'a, 'tcx> hir::intravisit::Visitor<'tcx> for Calls<'a, 'tcx> {

    fn visit_expr(&mut self, expr: &'tcx Expr) {
        //println!("expr {:?}", expr, );
        match expr.node {
            ::hir::ExprKind::Call(ref fn_expr, ref _args) => {
                if let ::hir::ExprKind::Path(ref qpath) = fn_expr.node {
                    let fn_def = self.cx.tables.qpath_def(qpath, fn_expr.hir_id);
                    match fn_def {
                        hir::def::Def::Fn(def_id) |
                        hir::def::Def::Method(def_id) => {
                            if def_id.is_local() {
                                self.process_local_call(&def_id);
                            } else {
                                match qpath {
                                    hir::QPath::Resolved(oty,call_path) => {
                                        match oty {
                                            None => {
                                                self.fn_info.push_external_call(def_id.krate,
                                                                                call_path.to_string());
                                            }
                                            Some (ty) => {
                                                println!("Expr {:?}", expr);
                                                //TODO
                                            }
                                        }
                                    }
                                    hir::QPath::TypeRelative(ref pty,path) => {
                                        let tys = print::to_string(
                                            print::NO_ANN,
                                            |s| s.print_type(pty));
                                        let mut res = String::new();
                                        res.push_str(&tys);
                                        res.push_str("::");
                                        res.push_str(&path.ident.to_string());
                                        self.fn_info.push_external_call(def_id.krate, res);
                                    }
                                }
                            }
                        }
                        hir::def::Def::VariantCtor(..) => {}
                        hir::def::Def::Local(..) => {} // closure
                        _ => {
                            println!("Def NOT handled {:?} in expr {:?}", fn_def, expr);
                        }
                    }
                } else {
                    println!("ExprKind NOT handled {:?} in expr {:?}", fn_expr.node, expr);
                }
            }
            ::hir::ExprKind::MethodCall(ref path_segment, _, ref arguments) => {
                let local_table = self.cx.tables.type_dependent_defs();
                if let Some (def) = local_table.get(expr.hir_id) {
                    if let hir::def::Def::Method(def_id) = def {
                        if def_id.is_local() {
                            self.process_local_call(&def_id);
                        } else {
                            let local_table = self.cx.tables.type_dependent_defs();
                            let def = local_table.get(expr.hir_id);

                            let arg_expr = &arguments[0];
                            match def {
                                Some(hir::def::Def::Method(def_id)) => {
                                    if !def_id.is_local() {
                                        let mut call = String::new();
                                        let ty = self.cx.tables.expr_ty_adjusted(&arg_expr);
                                        match ty.sty {
                                            rustc::ty::TypeVariants::TyRef(_,ref ty,_) => {
                                                call.push_str(&ty.to_string());
                                            }
                                            _ => {
                                                call.push_str(&ty.to_string());
                                            }
                                        }
                                        call.push_str("::");
                                        call.push_str(&path_segment.ident.to_string());
                                        self.fn_info.push_external_call(def_id.krate, call);
                                    }
                                }
                                _ => {println!("Def not hir::def::Def::Method {:?}", arg_expr);}
                            }
                        }
                    } else {
                        println!("Def NOT handled {:?} in expr {:?}", def, expr);
                    }
                } else {
                    //TODO  output.write(&zero_buf[::std::ops::RangeTo{end: amount_to_write,}]) in elf2tbf
                    println!("Def NOT found for {:?}", expr);
                }
            }
            _ => {}
        }
        hir::intravisit::walk_expr(self, expr);
    }

    fn nested_visit_map<'this>(&'this mut self) -> intravisit::NestedVisitorMap<'this, 'tcx> {
        intravisit::NestedVisitorMap::All(&self.cx.tcx.hir)
    }
}


#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_late_lint_pass(box HiddenUnsafe::new() as LateLintPassObject);
}

