
use rustc;
use rustc::lint::LateContext;

use results::blocks::BlockSummary;

pub fn run_analysis<'a, 'tcx>(cx: &'a LateContext<'a, 'tcx>) -> BlockSummary  {
    let mut visitor = BlockVisitor::new(&cx.tcx.hir);
    rustc::hir::intravisit::walk_crate(&mut visitor, cx.tcx.hir.krate());
    BlockSummary::new(visitor.unsafe_blocks)
}

struct BlockVisitor<'tcx> {
    hir: &'tcx rustc::hir::map::Map<'tcx>,
    unsafe_blocks: usize,
}

impl<'tcx> BlockVisitor<'tcx> {
    pub fn new(hir: &'tcx rustc::hir::map::Map<'tcx>) -> Self {
        BlockVisitor {
            unsafe_blocks: 0 as usize,
            hir,
        }
    }
}

impl<'a, 'tcx> rustc::hir::intravisit::Visitor<'tcx> for BlockVisitor<'tcx> {
    fn visit_block(&mut self, b: &'tcx rustc::hir::Block) {
        match b.rules {
            rustc::hir::BlockCheckMode::DefaultBlock => {}
            rustc::hir::BlockCheckMode::UnsafeBlock(unsafe_source) => {
                match unsafe_source {
                    rustc::hir::UnsafeSource::UserProvided => {
                        self.unsafe_blocks = self.unsafe_blocks + 1;
                    }
                    rustc::hir::UnsafeSource::CompilerGenerated => {
                        info!("hir::UnsafeSource::CompilerGenerated");
                    }
                }
            }
            rustc::hir::BlockCheckMode::PushUnsafeBlock(unsafe_source) => {
                error!("hir::BlockCheckMode::PushUnsafeBlock {:?}", unsafe_source);
            }
            rustc::hir::BlockCheckMode::PopUnsafeBlock(unsafe_source) => {
                error!("hir::BlockCheckMode::PopUnsafeBlock {:?}", unsafe_source);
            }
        }
        rustc::hir::intravisit::walk_block(self, b);
    }

    fn nested_visit_map<'this>(
        &'this mut self,
    ) -> rustc::hir::intravisit::NestedVisitorMap<'this, 'tcx> {
        rustc::hir::intravisit::NestedVisitorMap::All(self.hir)
    }
}

//struct BasicBlocksVisitor<'a, 'tcx: 'a> {
//    mir: &'a rustc::mir::Mir<'tcx>,
//    in_unsafe_bb: usize,
//    total_bb: usize,
//}
//
//pub fn run_analysis<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, node_ids: &Vec<NodeId>) -> BlockSummary {
//    let mut result = BlockSummary::new(0,0);
//    for node_id in node_ids {
//        let fn_def_id = cx.tcx.hir.local_def_id(*node_id);
//        let mir = &mut cx.tcx.optimized_mir(fn_def_id);
//        let mut body_visitor = BasicBlocksVisitor::new(mir);
//        body_visitor.visit_mir(mir);
//        result.add(body_visitor.in_unsafe_bb,
//                   body_visitor.total_bb);
//    }
//    result
//}
//
//impl<'a, 'tcx> BasicBlocksVisitor<'a, 'tcx> {
//    fn new(mir: &'a rustc::mir::Mir<'tcx>) -> Self {
//        BasicBlocksVisitor {
//            mir,
//            in_unsafe_bb: 0 as usize,
//            total_bb: 0 as usize,
//        }
//    }
//}
//
//impl<'a, 'tcx> Visitor<'tcx> for BasicBlocksVisitor<'a, 'tcx> {
//    fn visit_terminator(
//        &mut self,
//        _block: rustc::mir::BasicBlock,
//        terminator: &rustc::mir::Terminator<'tcx>,
//        _location: rustc::mir::Location,
//    ) {
//        self.total_bb = self.total_bb + 1;
//        //terminator.source_info
//        match self.mir.source_scope_local_data {
//            rustc::mir::ClearCrossCrate::Set(ref local_data_set) => {
//                if let Some(local_data) = local_data_set.get(terminator.source_info.scope) {
//                    match local_data.safety {
//                        // TODO think more about Safety::BuiltinUnsafe
//                        rustc::mir::Safety::Safe | rustc::mir::Safety::FnUnsafe => {}
//                        rustc::mir::Safety::BuiltinUnsafe
//                        | rustc::mir::Safety::ExplicitUnsafe(_) => {
//                            self.in_unsafe_bb = self.in_unsafe_bb + 1;
//                        }
//                    }
//                }
//            }
//            rustc::mir::ClearCrossCrate::Clear => {
//                error!("unsafety_violations: - remote, skipping");
//            }
//        }
//    }
//}