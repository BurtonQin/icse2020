use rustc::hir;

pub mod coarse;
mod deps;
pub mod rta;
//pub mod rta1;


struct UnsafeBlocksVisitorData<'tcx> {
    hir: &'tcx hir::map::Map<'tcx>,
    has_unsafe: bool,
}

impl<'a, 'tcx> hir::intravisit::Visitor<'tcx> for UnsafeBlocksVisitorData<'tcx> {
    fn visit_block(&mut self, b: &'tcx hir::Block) {
        match b.rules {
            hir::BlockCheckMode::DefaultBlock => {
                hir::intravisit::walk_block(self, b);
            }
            hir::BlockCheckMode::UnsafeBlock(_unsafe_source)
            | hir::BlockCheckMode::PushUnsafeBlock(_unsafe_source)
            | hir::BlockCheckMode::PopUnsafeBlock(_unsafe_source) => {
                self.has_unsafe = true;
            }
        }
    }

    fn nested_visit_map<'this>(&'this mut self) -> hir::intravisit::NestedVisitorMap<'this, 'tcx> {
        hir::intravisit::NestedVisitorMap::All(self.hir)
    }
}

pub fn is_library_crate(crate_name: &String) -> bool {
    crate_name.as_str() == "alloc" || crate_name.as_str() == "std" || crate_name.as_str() == "core" || crate_name.as_str() == "proc_macro"
    //false
}