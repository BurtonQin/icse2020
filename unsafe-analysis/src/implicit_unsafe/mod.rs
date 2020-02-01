use rustc::hir;
mod deps;
pub mod rta;
use rustc;

struct UnsafeBlocksVisitorData<'tcx> {
    hir: &'tcx hir::map::Map<'tcx>,
    has_unsafe: bool,
}

impl<'a, 'tcx> hir::intravisit::Visitor<'tcx> for UnsafeBlocksVisitorData<'tcx> {
    fn visit_block(&mut self, b: &'tcx hir::Block) {
        match b.rules {
            rustc::hir::BlockCheckMode::DefaultBlock => {}
            rustc::hir::BlockCheckMode::UnsafeBlock(unsafe_source) |
            rustc::hir::BlockCheckMode::PushUnsafeBlock(unsafe_source) |
            rustc::hir::BlockCheckMode::PopUnsafeBlock(unsafe_source) => {
                match unsafe_source {
                    rustc::hir::UnsafeSource::UserProvided => {
                        self.has_unsafe = true;
                    }
                    rustc::hir::UnsafeSource::CompilerGenerated => {
                    }
                }
            }
        }
    }

    fn nested_visit_map<'this>(&'this mut self) -> hir::intravisit::NestedVisitorMap<'this, 'tcx> {
        hir::intravisit::NestedVisitorMap::All(self.hir)
    }
}

pub fn is_library_crate(crate_name: &String) -> bool {
    crate_name.as_str() == "alloc" ||
        crate_name.as_str() == "std" ||
        crate_name.as_str() == "core" ||
        crate_name.as_str() == "proc_macro" ||
        crate_name.as_str() == "clippy"
    //false
}