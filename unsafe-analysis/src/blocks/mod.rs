
use rustc;
use syntax::ast::NodeId;
use rustc::lint::LateContext;
use rustc::mir::SourceInfo;
use rustc::mir::visit::Visitor;

use unsafety_sources::{UnsafetySourcesVisitor,UnsafetySourceCollector};

use results::blocks::BlockSummary;
use results::blocks::BlockUnsafetySourcesAnalysis;
use results::unsafety_sources::SourceKind;
use results::unsafety_sources::Source;

//////////////////// Summary

pub fn run_summary_analysis<'a, 'tcx>(cx: &'a LateContext<'a, 'tcx>) -> BlockSummary  {
    let mut visitor = BlockVisitor::new(&cx.tcx.hir);

    info!("Processing crate {:?}",  ::local_crate_name());

    rustc::hir::intravisit::walk_crate(&mut visitor, cx.tcx.hir.krate());
    BlockSummary::new(visitor.unsafe_blocks,visitor.total_blocks)
}

struct BlockVisitor<'tcx> {
    hir: &'tcx rustc::hir::map::Map<'tcx>,
    unsafe_blocks: usize,
    total_blocks: usize,
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
        match b.rules {
            rustc::hir::BlockCheckMode::DefaultBlock => {}
            rustc::hir::BlockCheckMode::UnsafeBlock(unsafe_source) => {
                match unsafe_source {
                    rustc::hir::UnsafeSource::UserProvided => {
                        self.unsafe_blocks = self.unsafe_blocks + 1;
                        info!("hir::UnsafeSource::UserProvided {:?}", self.unsafe_blocks);
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
        //count all the blocks, including the compiler generated ones
        self.total_blocks = self.total_blocks + 1;
        rustc::hir::intravisit::walk_block(self, b);
    }

    fn nested_visit_map<'this>(
        &'this mut self,
    ) -> rustc::hir::intravisit::NestedVisitorMap<'this, 'tcx> {
        rustc::hir::intravisit::NestedVisitorMap::All(self.hir)
    }
}

//////////////////// unsafety sources

impl UnsafetySourceCollector for BlockUnsafetySourcesAnalysis {
    fn add_unsafety_source<'a, 'tcx>(
        &mut self,
        cx: &LateContext<'a, 'tcx>,
        kind: SourceKind,
        source_info: SourceInfo,
        block_id: NodeId,
    ) {
        let source = Source {
            kind,
            loc: ::get_file_and_line(cx, source_info.span),
        };
        self.add_source(block_id.to_string(), source)
    }
}

pub fn run_unsafety_sources_analysis<'a, 'tcx>(cx: &'a LateContext<'a, 'tcx>, fns: &Vec<NodeId>,
            user_defined_only: bool) -> Vec<BlockUnsafetySourcesAnalysis> {
    let mut res =Vec::new();
    for &node_id in fns {
        let mut sources= BlockUnsafetySourcesAnalysis::new();
        let fn_def_id = cx.tcx.hir.local_def_id(node_id);
        // closures are handled by their parent fn.
        if !cx.tcx.is_closure(fn_def_id) {
            let mir = &mut cx.tcx.optimized_mir(fn_def_id);
            if let Some(mut body_visitor) =
            UnsafetySourcesVisitor::new(cx, mir, &mut sources, fn_def_id, user_defined_only)
                {
                    body_visitor.visit_mir(mir);
                }
        }
        if !sources.sources().is_empty() {
            res.push(sources);
        }
    }
    res
}
