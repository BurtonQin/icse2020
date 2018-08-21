use print::Print;
use rustc::hir;
use rustc::lint::LateContext;
use syntax::ast::NodeId;
use util;
use std::fs::File;
use std::io::Write;

pub struct FnInfo {
    decl_id: NodeId,
    local_calls: Vec<NodeId>,
    external_calls: Vec<(hir::def_id::CrateNum, String)>,
}

impl FnInfo {
    pub fn decl_id(&self) -> NodeId {
        self.decl_id
    }

    pub fn local_calls(&self) -> &Vec<NodeId> {
        &self.local_calls
    }

    pub fn external_calls(&self) -> &Vec<(hir::def_id::CrateNum, String)> {
        &self.external_calls
    }

    pub fn new(hir_id: NodeId) -> Self {
        Self {
            decl_id: hir_id,
            local_calls: Vec::new(),
            external_calls: Vec::new(),
        }
    }

    pub fn push_external_call<'a, 'tcx>(&mut self, cx: &LateContext<'a, 'tcx>,
                                        def_id: hir::def_id::DefId) -> () {
        let krate = def_id.krate;

        let mut crate_name: String = cx.tcx.crate_name(krate).to_string();
        crate_name.push_str("::");

        let func = cx.tcx.item_path_str(def_id).to_string().replace(crate_name.as_str(),"");

        let found = self
            .external_calls
            .iter()
            .any(|elt| elt.1 == func && elt.0 == krate);
        if !found {
            self.external_calls.push((krate, func.to_string()));
        }
    }

    pub fn push_local_call(&mut self, node_id: NodeId) -> () {
        let found = self.local_calls.iter().any(|elt| *elt == node_id);
        if !found {
            self.local_calls.push(node_id);
        }
    }

    pub fn print<'a, 'tcx>(&self, cx: &LateContext<'a, 'tcx>, printer: &Print, file: &mut File) {
        let tcx = cx.tcx;
        let span = tcx.hir.span(self.decl_id);
        file.write_fmt( format_args!(
               "{:?} | ",
            tcx.node_path_str(self.decl_id))
        ).unwrap();
        util::print_file_and_line(cx,span,file);
        printer.print(cx, file);
        writeln!(file, "");
    }

    pub fn print_local_calls<'a, 'tcx>(&self, cx: &LateContext<'a, 'tcx>, file: &mut File) {
        if !self.local_calls.is_empty() {
            writeln!(file, "Local calls:");
            for node_id in self.local_calls.iter() {
                writeln!(file, "{:?} | {:?} ", cx.tcx.node_path_str(*node_id), node_id);
            }
        }
    }

    pub fn print_external_calls<'a, 'tcx>(&self, cx: &LateContext<'a, 'tcx>, file: &mut File) {
        let tcx = cx.tcx;
        let mut external_crates = Vec::new();
        self.external_calls.iter().for_each(|elt| {
            if !external_crates.iter().any(|crate_num| *crate_num == elt.0) {
                external_crates.push(elt.0)
            }
        });

        external_crates.iter().for_each(|krate| {
            writeln!(file, "External crate {:?}", tcx.crate_name(*krate));
            self.external_calls
                .iter()
                .filter(|elt| elt.0 == *krate)
                .for_each(|elt| {
                    writeln!(file, "{:?}", elt.1);
                });
        });
    }
}

