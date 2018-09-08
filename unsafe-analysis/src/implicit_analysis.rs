use analysis::Analysis;
use fn_info::FnInfo;
use util;

use rustc::hir;
use rustc::hir::intravisit;
use rustc::lint::LateContext;
use rustc::mir::visit::Visitor;
use rustc::mir::{BasicBlock, Location, Operand, Terminator, TerminatorKind};
use rustc::ty::TyKind;

use results::implicit::UnsafeInBody;
use results::implicit::UnsafeTraitSafeMethodInBody;

///////////////////////////////// Blocks /////////////////////////////////////////////////
//pub fn save_implicit_analysis(analysis_results: Vec<(&FnInfo, UnsafeInBody)>) {
//    let cnv = util::local_crate_name_and_version();
//    let file = implicit::get_implicit_unsafe_file(cnv.0, cnv.1).open_file();
//    for (_, ub) in analysis_results.iter() {
//        let serialized = serde_json::to_string(ub).unwrap();
//        writeln!(file, "{}", serialized);
//    }
//}

impl Analysis for UnsafeInBody {
    fn is_set(&self) -> bool {
        self.has_unsafe
    }

    fn set(&mut self) {
        self.has_unsafe = true
    }

    fn run_analysis<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, fn_info: &'a FnInfo) -> Self {
        let tcx = &cx.tcx;
        let hir = &tcx.hir;
        let body_id = hir.body_owned_by(fn_info.decl_id());
        let body = hir.body(body_id);
        let mut visitor = UnsafeBlocksVisitorData {
            hir: &hir,
            has_unsafe: false,
        };
        hir::intravisit::walk_body(&mut visitor, body);
        let mut analysis = Self::new(tcx.node_path_str(fn_info.decl_id()));
        if visitor.has_unsafe {
            analysis.set();
        }
        analysis
    }
}

struct UnsafeBlocksVisitorData<'tcx> {
    hir: &'tcx hir::map::Map<'tcx>,
    has_unsafe: bool,
}

impl<'a, 'tcx> hir::intravisit::Visitor<'tcx> for UnsafeBlocksVisitorData<'tcx> {
    fn visit_block(&mut self, b: &'tcx hir::Block) {
        match b.rules {
            hir::BlockCheckMode::DefaultBlock => {}
            hir::BlockCheckMode::UnsafeBlock(_unsafe_source) => {
                self.has_unsafe = true;
            }
            hir::BlockCheckMode::PushUnsafeBlock(unsafe_source) => {
                error!("hir::BlockCheckMode::PushUnsafeBlock {:?}", unsafe_source);
            }
            hir::BlockCheckMode::PopUnsafeBlock(unsafe_source) => {
                error!("hir::BlockCheckMode::PopUnsafeBlock {:?}", unsafe_source);
            }
        }
        hir::intravisit::walk_block(self, b);
    }

    fn nested_visit_map<'this>(&'this mut self) -> intravisit::NestedVisitorMap<'this, 'tcx> {
        intravisit::NestedVisitorMap::All(self.hir)
    }
}

pub fn propagate_external<'a, 'tcx>(
    cx: &LateContext<'a, 'tcx>,
    graph: &mut Vec<(&FnInfo, UnsafeInBody)>,
    external_unsafety: &Vec<(hir::def_id::CrateNum, Vec<UnsafeInBody>)>,
) {
    let mut changes = true;
    let mut with_unsafe = Vec::new();

    // collect local functions that have been marked as havinb unsafe
    for (ref fn_info, ref mut t) in graph.iter_mut() {
        if t.has_unsafe {
            with_unsafe.push(fn_info.decl_id());
        }
    }

    // for each normal function
    for (ref fn_info, ref mut t) in graph.iter_mut() {
        // for each external call from the local function
        for (ext_crate_num, ext_call) in fn_info.external_calls() {
            if let Some((_, ub_vec)) = external_unsafety.iter().find(|&x| *ext_crate_num == x.0) {
                if let Some(ext_unsafety_in_body) = ub_vec.iter().find(|&x| x.fn_name == *ext_call)
                {
                    if ext_unsafety_in_body.has_unsafe {
                        t.set();
                        with_unsafe.push(fn_info.decl_id());
                    }
                } else {
                    let crate_name = cx.tcx.crate_name(*ext_crate_num).to_string();
                    if !util::is_excluded_crate(&crate_name) {
                        error!("Error external call NOT found {:?}", ext_call);
                    }
                }
            }
        }
    }

    while changes {
        changes = false;
        for &mut (ref fn_info, ref mut t) in graph.iter_mut() {
            if !t.is_set() {
                if (&fn_info.local_calls())
                    .into_iter()
                    //TODO continue here
                    .any(|call_id| with_unsafe.iter().any(|x| *x == *call_id))
                {
                    with_unsafe.push(fn_info.decl_id());
                    t.set();
                    changes = true;
                }
            }
        }
    }
}

////////////////////////////// Traits ///////////////////////////////////////////////////////

impl Analysis for UnsafeTraitSafeMethodInBody {
    fn is_set(&self) -> bool {
        self.has_unsafe
    }

    fn set(&mut self) {
        self.has_unsafe = true
    }

    fn run_analysis<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, fn_info: &'a FnInfo) -> Self {
        let tcx = cx.tcx;
        let owner_def_id = tcx.hir.local_def_id(fn_info.decl_id());
        let mut mir = tcx.optimized_mir(owner_def_id);
        let mut unsafe_trait_visitor = SafeMethodsInUnsafeTraits::new(cx);
        unsafe_trait_visitor.visit_mir(&mut mir);
        Self {
            fn_name: util::get_node_name(cx, fn_info.decl_id()),
            has_unsafe: unsafe_trait_visitor.has_unsafe,
        }
    }
}

struct SafeMethodsInUnsafeTraits<'a, 'tcx: 'a> {
    cx: &'a LateContext<'a, 'tcx>,
    has_unsafe: bool,
}

impl<'a, 'tcx> SafeMethodsInUnsafeTraits<'a, 'tcx> {
    fn new(cx: &'a LateContext<'a, 'tcx>) -> Self {
        SafeMethodsInUnsafeTraits {
            cx,
            has_unsafe: false,
        }
    }
}

impl<'a, 'tcx> Visitor<'tcx> for SafeMethodsInUnsafeTraits<'a, 'tcx> {
    fn visit_terminator(
        &mut self,
        _block: BasicBlock,
        terminator: &Terminator<'tcx>,
        _location: Location,
    ) {
        if let TerminatorKind::Call {
            ref func,
            args: _,
            destination: _,
            cleanup: _,
        } = terminator.kind
        {
            if let Operand::Constant(constant) = func {
                if let TyKind::FnDef(callee_def_id, _) = constant.literal.ty.sty {
                    let calee_sig = self.cx.tcx.fn_sig(callee_def_id);
                    if let hir::Unsafety::Normal = calee_sig.unsafety() {
                        // need to find the trait if it's a method impl
                        if callee_def_id.is_local() {
                            let callee_node_id =
                                self.cx.tcx.hir.def_index_to_node_id(callee_def_id.index);
                            match self.cx.tcx.hir.get(callee_node_id) {
                                hir::Node::TraitItem(ref _trait_item) => {
                                    let trait_node_id =
                                        self.cx.tcx.hir.get_parent_node(callee_node_id);
                                    if let hir::Node::Item(item) =
                                        self.cx.tcx.hir.get(trait_node_id)
                                    {
                                        if let hir::ItemKind::Trait(_, unsafety, ..) = item.node {
                                            if let hir::Unsafety::Unsafe = unsafety {
                                                self.has_unsafe = true;
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}
