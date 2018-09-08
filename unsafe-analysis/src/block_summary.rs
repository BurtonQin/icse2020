use rustc;
use rustc::lint::LateContext;
use rustc::mir::visit::Visitor;

use analysis::Analysis;
use fn_info::FnInfo;
use results::blocks::BlockSummary;

impl Analysis for BlockSummary {
    fn run_analysis<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, fn_info: &'a FnInfo) -> Self {
        let fn_def_id = cx.tcx.hir.local_def_id(fn_info.decl_id());
        //needed for the borrow checker
        let mir = &mut cx.tcx.optimized_mir(fn_def_id);
        let mut body_visitor = BasicBlocksVisitor::new(mir);
        body_visitor.visit_mir(mir);
        let body_id = cx.tcx.hir.body_owned_by(fn_info.decl_id());
        let body = cx.tcx.hir.body(body_id);
        let mut hir_visitor = BlockVisitor::new(&cx.tcx.hir);
        rustc::hir::intravisit::walk_body(&mut hir_visitor, body);
        BlockSummary::new(
            body_visitor.in_unsafe_bb,
            body_visitor.total_bb,
            hir_visitor.unsafe_blocks,
            hir_visitor.total_blocks,
        )
    }
}

pub fn collect(res: Vec<(&FnInfo, BlockSummary)>) -> BlockSummary {
    let mut in_unsafe_bb: usize = 0;
    let mut total_bb: usize = 0;
    let mut hir_unsafe_blocks: usize = 0;
    let mut hir_total: usize = 0;
    for (_, summary) in res.iter() {
        in_unsafe_bb = in_unsafe_bb + summary.in_unsafe_bb;
        total_bb = total_bb + summary.total_bb;
        hir_unsafe_blocks = hir_unsafe_blocks + summary.hir_unsafe_blocks;
        hir_total = hir_total + summary.hir_total;
    }
    BlockSummary {
        in_unsafe_bb,
        total_bb,
        hir_unsafe_blocks,
        hir_total,
    }
}

////////////////////// Hir Analysis

//////////////////// Mir Analysis
struct BasicBlocksVisitor<'a, 'tcx: 'a> {
    mir: &'a rustc::mir::Mir<'tcx>,
    in_unsafe_bb: usize,
    total_bb: usize,
}

impl<'a, 'tcx> BasicBlocksVisitor<'a, 'tcx> {
    fn new(mir: &'a rustc::mir::Mir<'tcx>) -> Self {
        BasicBlocksVisitor {
            mir,
            in_unsafe_bb: 0 as usize,
            total_bb: 0 as usize,
        }
    }
}

impl<'a, 'tcx> Visitor<'tcx> for BasicBlocksVisitor<'a, 'tcx> {
    fn visit_terminator(
        &mut self,
        _block: rustc::mir::BasicBlock,
        terminator: &rustc::mir::Terminator<'tcx>,
        _location: rustc::mir::Location,
    ) {
        self.total_bb = self.total_bb + 1;
        //terminator.source_info
        match self.mir.source_scope_local_data {
            rustc::mir::ClearCrossCrate::Set(ref local_data_set) => {
                if let Some(local_data) = local_data_set.get(terminator.source_info.scope) {
                    match local_data.safety {
                        // TODO think more about Safety::BuiltinUnsafe
                        rustc::mir::Safety::Safe | rustc::mir::Safety::FnUnsafe => {}
                        rustc::mir::Safety::BuiltinUnsafe
                        | rustc::mir::Safety::ExplicitUnsafe(_) => {
                            self.in_unsafe_bb = self.in_unsafe_bb + 1;
                        }
                    }
                }
            }
            rustc::mir::ClearCrossCrate::Clear => {
                println!("unsafety_violations: - remote, skipping");
            }
        }
    }
}

//////////////////////// Hir

struct BlockVisitor<'tcx> {
    unsafe_blocks: usize,
    total_blocks: usize,
    hir: &'tcx rustc::hir::map::Map<'tcx>,
}

impl<'tcx> BlockVisitor<'tcx> {
    pub fn new(hir: &'tcx rustc::hir::map::Map<'tcx>) -> Self {
        BlockVisitor {
            unsafe_blocks: 0 as usize,
            total_blocks: 0 as usize,
            hir,
        }
    }
}

impl<'a, 'tcx> rustc::hir::intravisit::Visitor<'tcx> for BlockVisitor<'tcx> {
    fn visit_block(&mut self, b: &'tcx rustc::hir::Block) {
        self.total_blocks = self.total_blocks + 1;
        match b.rules {
            rustc::hir::BlockCheckMode::DefaultBlock => {}
            rustc::hir::BlockCheckMode::UnsafeBlock(_unsafe_source) => {
                self.unsafe_blocks = self.unsafe_blocks + 1;
            }
            rustc::hir::BlockCheckMode::PushUnsafeBlock(unsafe_source) => {
                println!("hir::BlockCheckMode::PushUnsafeBlock {:?}", unsafe_source);
            }
            rustc::hir::BlockCheckMode::PopUnsafeBlock(unsafe_source) => {
                println!("hir::BlockCheckMode::PopUnsafeBlock {:?}", unsafe_source);
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
