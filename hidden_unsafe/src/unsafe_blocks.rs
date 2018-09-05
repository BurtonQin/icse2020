use analysis::Analysis;
use fn_info::FnInfo;
use print::Print;
use rustc::hir;
use rustc::hir::intravisit;
use rustc::lint::LateContext;
use std::fs::File;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug)]
pub struct UnsafeInBody  {
    fn_info: String,
    has_unsafe: bool,
}

impl Print for UnsafeInBody {

    fn print<'a, 'tcx>(&self, _cx: &LateContext<'a, 'tcx>, file: &mut File) -> () {
        let serialized = serde_json::to_string(self).unwrap();
        writeln!(file, "{:?}", serialized);
    }
}

impl UnsafeInBody {
    fn new(fn_info: String) -> Self {
        UnsafeInBody { has_unsafe: false, fn_info }
    }

    pub fn get_output_filename() -> &'static str {
        "10_unsafe_in_call_tree"
    }
}

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
                println!("hir::BlockCheckMode::PushUnsafeBlock {:?}", unsafe_source);
            }
            hir::BlockCheckMode::PopUnsafeBlock(unsafe_source) => {
                println!("hir::BlockCheckMode::PopUnsafeBlock {:?}", unsafe_source);
            }
        }
        hir::intravisit::walk_block(self, b);
    }

    fn nested_visit_map<'this>(&'this mut self) -> intravisit::NestedVisitorMap<'this, 'tcx> {
        intravisit::NestedVisitorMap::All(self.hir)
    }
}

