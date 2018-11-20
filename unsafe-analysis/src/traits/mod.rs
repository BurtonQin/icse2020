use rustc;
use rustc::lint::LateContext;

use results::traits::UnsafeTrait;
use get_node_name;

pub struct TraitsAnalysis {
    pub unsafe_traits_impls: Vec<UnsafeTrait>,
    pub unsafe_traits: Vec<UnsafeTrait>,
}

pub fn run_analysis<'a, 'tcx>(cx: &'a LateContext<'a, 'tcx>) -> TraitsAnalysis {
    let mut visitor = TraitVisitor::new(cx);
    rustc::hir::intravisit::walk_crate(&mut visitor, cx.tcx.hir.krate());
    TraitsAnalysis{
        unsafe_traits_impls: visitor.unsafe_traits_impls,
        unsafe_traits: visitor.unsafe_traits
    }
}

struct TraitVisitor<'a, 'tcx: 'a> {
    unsafe_traits_impls: Vec<UnsafeTrait>,
    unsafe_traits: Vec<UnsafeTrait>,
    cx: &'a LateContext<'a, 'tcx>,
}

impl<'a, 'tcx> TraitVisitor<'a, 'tcx> {
    pub fn new(cx: &'a LateContext<'a, 'tcx>) -> Self {
        TraitVisitor {
            unsafe_traits_impls: Vec::new(),
            unsafe_traits: Vec::new(),
            cx,
        }
    }
}

impl<'a, 'tcx> rustc::hir::intravisit::Visitor<'tcx> for TraitVisitor<'a, 'tcx> {
    fn visit_item(&mut self, item: &'tcx rustc::hir::Item) {
        if let rustc::hir::ItemKind::Impl(rustc::hir::Unsafety::Unsafe, ..) = item.node {
            self.unsafe_traits_impls
                    .push(UnsafeTrait::new(get_node_name(self.cx,  self.cx.tcx.hir.local_def_id(item.id))));
        } else {
            if let rustc::hir::ItemKind::Trait(_, rustc::hir::Unsafety::Unsafe, ..) = item.node {
                self.unsafe_traits
                        .push(UnsafeTrait::new(::get_node_name(self.cx,  self.cx.tcx.hir.local_def_id(item.id))));
            }
        }
        rustc::hir::intravisit::walk_item(self, item);
    }

    fn nested_visit_map<'this>(
        &'this mut self,
    ) -> rustc::hir::intravisit::NestedVisitorMap<'this, 'tcx> {
        rustc::hir::intravisit::NestedVisitorMap::All(&self.cx.tcx.hir)
    }
}