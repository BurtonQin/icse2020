use rustc::hir;
use rustc::lint::LateContext;
use syntax::ast::NodeId;
use util;

#[derive(Debug)]
pub struct FnInfo {
    decl_id: NodeId,
    local_calls: Vec<NodeId>,
    external_calls: Vec<(hir::def_id::CrateNum, String, Safety)>,
}

#[derive(Debug)]
pub enum Safety {
    Unsafe,
    Normal,
}

impl FnInfo {
    pub fn decl_id(&self) -> NodeId {
        self.decl_id
    }

    pub fn local_calls(&self) -> &Vec<NodeId> {
        &self.local_calls
    }

    pub fn external_calls(&self) -> &Vec<(hir::def_id::CrateNum, String, Safety)> {
        &self.external_calls
    }

    pub fn new(hir_id: NodeId) -> Self {
        Self {
            decl_id: hir_id,
            local_calls: Vec::new(),
            external_calls: Vec::new(),
        }
    }

    pub fn push_external_call<'a, 'tcx>(
        &mut self,
        cx: &LateContext<'a, 'tcx>,
        def_id: hir::def_id::DefId,
        safety: Safety
    ) -> () {

        let krate = def_id.krate;

        let mut crate_name: String = cx.tcx.crate_name(krate).to_string();
        crate_name.push_str("::");

        let func = cx
            .tcx
            .item_path_str(def_id)
            .to_string()
            .replace(crate_name.as_str(), "");
        let found = self
            .external_calls
            .iter()
            .any(|elt| elt.1 == func && elt.0 == krate);

        if !found {
            self.external_calls.push((krate, func,safety));
        }
    }

    pub fn push_local_call(&mut self, node_id: NodeId) -> () {
        let found = self.local_calls.iter().any(|elt| *elt == node_id);
        if !found {
            self.local_calls.push(node_id);
        }
    }

    pub fn build_long_fn_info<'a, 'tcx>(
        &self,
        cx: &LateContext<'a, 'tcx>,
    ) -> results::functions::LongFnInfo {
        let name = cx.tcx.node_path_str(self.decl_id);
        let node_id = self.decl_id.to_string();
        let span = cx.tcx.hir.span(self.decl_id);
        let location = util::get_file_and_line(cx, span);

        let mut local_calls = Vec::new();
        for node_id in self.local_calls.iter() {
            local_calls.push((cx.tcx.node_path_str(*node_id), node_id.to_string()));
        }

        let mut external_calls = Vec::new();
        let mut external_crates = Vec::new();
        self.external_calls.iter().for_each(|elt| {
            if !external_crates.iter().any(|crate_num| *crate_num == elt.0) {
                external_crates.push(elt.0)
            }
        });
        external_crates.iter().for_each(|krate| {
            let crate_name = cx.tcx.crate_name(*krate);
            let mut crate_calls = Vec::new();
            self.external_calls
                .iter()
                .filter(|elt| elt.0 == *krate)
                .for_each(|elt| {
                    crate_calls.push(elt.1.clone());
                });
            external_calls.push((crate_name.to_string(), crate_calls));
        });
        results::functions::LongFnInfo::new(name, node_id, location, local_calls, external_calls)
    }

    pub fn build_short_fn_info<'a, 'tcx>(
        &self,
        cx: &LateContext<'a, 'tcx>,
    ) -> results::functions::ShortFnInfo {
        let name = cx.tcx.node_path_str(self.decl_id);
        let node_id = self.decl_id.to_string();
        let span = cx.tcx.hir.span(self.decl_id);
        let location = util::get_file_and_line(cx, span);
        results::functions::ShortFnInfo::new(name, node_id, location)
    }
}
