
use rustc;
use syntax::ast::NodeId;
use rustc::lint::LateContext;
use rustc::mir::SourceInfo;
use rustc::mir::visit::Visitor;
use rustc::hir::HirId;

use unsafety_sources::{UnsafetySourcesVisitor,UnsafetySourceCollector};

use results::blocks::BlockSummary;
use results::unsafety_sources::SourceKind;
use results::unsafety_sources::Source;
use results::blocks::BlockUnsafetySource;

//////////////////// Summary

pub fn run_summary_analysis<'a, 'tcx>(cx: &'a LateContext<'a, 'tcx>) -> BlockSummary  {
    let mut visitor = BlockVisitor::new(&cx.tcx.hir());
    rustc::hir::intravisit::walk_crate(&mut visitor, cx.tcx.hir().krate());
    BlockSummary::new( visitor.user_unsafe_blocks, visitor.unsafe_blocks,visitor.total_blocks)
}

struct BlockVisitor<'tcx> {
    hir: &'tcx rustc::hir::map::Map<'tcx>,
    user_unsafe_blocks: usize,
    unsafe_blocks: usize,
    total_blocks: usize,
}

impl<'tcx> BlockVisitor<'tcx> {
    pub fn new(hir: &'tcx rustc::hir::map::Map<'tcx>) -> Self {
        BlockVisitor {
            user_unsafe_blocks: 0 as usize,
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
            rustc::hir::BlockCheckMode::UnsafeBlock(unsafe_source) |
            rustc::hir::BlockCheckMode::PushUnsafeBlock(unsafe_source) |
            rustc::hir::BlockCheckMode::PopUnsafeBlock(unsafe_source) => {
                match unsafe_source {
                    rustc::hir::UnsafeSource::UserProvided => {
                        self.user_unsafe_blocks = self.user_unsafe_blocks + 1;
                    }
                    rustc::hir::UnsafeSource::CompilerGenerated => {
                    }
                }
                self.unsafe_blocks = self.unsafe_blocks + 1;
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

pub struct BlockUnsafetySourcesAnalysis {
    sources: Vec<BlockUnsafetySource>,
}

impl BlockUnsafetySourcesAnalysis {
    pub fn new() -> Self {
        BlockUnsafetySourcesAnalysis {
            sources: Vec::new(),
        }
    }

    pub fn sources(&self) -> &Vec<BlockUnsafetySource> { &&self.sources }

    pub fn add_source(&mut self, block_id: String, user_provided: bool, source: Source) {
        self.sources.push(BlockUnsafetySource{
            block_id, source
        })
    }
}

impl UnsafetySourceCollector for BlockUnsafetySourcesAnalysis {
    fn add_unsafety_source<'a, 'tcx>(
        &mut self,
        cx: &LateContext<'a, 'tcx>,
        kind: SourceKind,
        source_info: SourceInfo,
        block_id: NodeId,
        user_provided: bool,
    ) {
        let source = Source {
            kind,
            user_provided,
            loc: ::get_file_and_line(cx, source_info.span),
        };
        self.add_source(block_id.to_string(), user_provided, source)
    }
}

pub fn run_unsafety_sources_analysis<'a, 'tcx>(cx: &'a LateContext<'a, 'tcx>, fns: &Vec<HirId>,
            ) -> Vec<BlockUnsafetySource> {
    let mut res =Vec::new();
    for &node_id in fns {
        let mut sources= BlockUnsafetySourcesAnalysis::new();
        let hir_id = cx.tcx.hir().hir_to_node_id(node_id);
        let fn_def_id = cx.tcx.hir().local_def_id(hir_id);
        // closures are handled by their parent fn.
        if !cx.tcx.is_closure(fn_def_id) {
            let body = &mut cx.tcx.optimized_mir(fn_def_id);
            if let Some(mut body_visitor) =
            UnsafetySourcesVisitor::new(cx, body, &mut sources, fn_def_id)
                {
                    body_visitor.visit_body(body);
                }
        }
        if !sources.sources().is_empty() {
            res.append(&mut sources.sources);
        }
    }
    res
}
