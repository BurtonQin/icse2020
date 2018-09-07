use syntax::ast::NodeId;

use rustc::hir;
use rustc::lint::LateContext;
use rustc::hir::def_id::DefId;
use syntax_pos::Span;
use std::path::Path;
use std::fmt::Write;

pub fn get_node_name<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, node_id: NodeId) -> String {
    cx.tcx.node_path_str(node_id)
}

pub fn get_def_id_string <'a, 'tcx>(_cx: &LateContext<'a, 'tcx>, def_id: DefId) -> String {
    let mut res = String::new();
    // TODO might add details
    write!(res, "{:#?}", def_id);
    res
}

pub fn is_unsafe_fn<'a, 'tcx>(node_id: NodeId, cx: &LateContext<'a, 'tcx>) -> bool {
    let node = cx.tcx.hir.get(node_id);
    match node {
        hir::Node::Item(item) => {
            if let hir::ItemKind::Fn(ref _fn_decl, ref fn_header, _, _) = item.node {
                if let hir::Unsafety::Normal = fn_header.unsafety {
                    false
                } else {
                    true
                }
            } else {
                //println!("Body owner node type NOT handled: {:?}", item);
                false
            }
        }
        _ => { false }
    }
}

pub fn is_unsafe_method<'a, 'tcx>(node_id: NodeId, cx: &LateContext<'a, 'tcx>) -> bool {
    let node = cx.tcx.hir.get(node_id);
    match node {
        hir::Node::ImplItem(ref impl_item) => {
            if let ::hir::ImplItemKind::Method(ref method_sig, ..) = impl_item.node {
                if let hir::Unsafety::Normal = method_sig.header.unsafety {
                    false
                } else {
                    true
                }
            } else {
                //println!("Impl Item Kind NOT handled {:?}", impl_item.node);
                false
            }
        }
        _ => { false }
    }
}

pub fn is_fn_or_method<'a, 'tcx>(node_id: NodeId, cx: &LateContext<'a, 'tcx>) -> bool {
    let node = cx.tcx.hir.get(node_id);
    match node {
        hir::Node::Item(_item) => {true}
        hir::Node::ImplItem(ref _impl_item) => {true}
        hir::Node::Expr(ref _expr) => {false} //closure
        hir::Node::AnonConst(ref _anon_const) => {
            // nothing to do - this is not a stand alone function
            // any unsafe in this body will be processed by the enclosing function or method
            false
        }
        _ => {
            //println!("Body owner node NOT handled: {:?}", node);
            false
        }
    }
}

pub fn get_file_and_line<'a, 'tcx>( cx: &LateContext<'a, 'tcx>, span: Span ) -> String {
    let mut result = String::new();
    let loc = cx.tcx.sess.source_map().lookup_char_pos(span.lo());
    let filename = &loc.file.name;
    write!(result,
           "file: {:?} line {:?} | ",
            filename,
            loc.line
    );
    result
}

pub fn local_crate_name_and_version() -> (String, String) {
    let manifest_path = Path::new("./Cargo.toml");
    let features = cargo_metadata::CargoOpt::AllFeatures;
    let metadata =
        cargo_metadata::metadata_run(Some(manifest_path), false, Some(features)).unwrap();

    //println!("Crate {:?} Version {:?}", metadata.packages[0].name.clone(),metadata.packages[0].version.clone());

    (metadata.packages[0].name.clone(),metadata.packages[0].version.clone())
}

pub fn is_excluded_crate(crate_name: &String) -> bool {
    crate_name.as_str() == "alloc"
        || crate_name.as_str() == "std"
        || crate_name.as_str() == "core"
}


