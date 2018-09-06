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
    pub fn_info: String,
    pub has_unsafe: bool,
}

impl Print for UnsafeInBody {

    fn print<'a, 'tcx>(&self, _cx: &LateContext<'a, 'tcx>, file: &mut File) -> () {
        let serialized = serde_json::to_string(self as &UnsafeInBody).unwrap();
        writeln!(file, "{}", serialized);
    }

}

impl UnsafeInBody {
    pub fn new(fn_info: String) -> Self {
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

pub fn propagate_external<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, graph: &mut Vec<(&FnInfo, UnsafeInBody)>
                          , external_unsafety: &Vec<UnsafeInBody>) {
    let mut changes = true;
    let mut with_unsafe = Vec::new();
    {
        // to pass borrow checker
        for (ref fn_info, ref mut t) in graph.iter_mut() {
            if t.has_unsafe {
                with_unsafe.push(fn_info.decl_id());
            }
        }
    }
    {
        for (ref fn_info, ref mut t) in graph.iter_mut() {
            for (ext_crate, ext_call) in fn_info.external_calls() {
                if let Some (ext_unsafety_in_body) = external_unsafety.iter().find(
                    |&x| {
                        *ext_call == x.fn_info
                    }
                ) {
                    if ext_unsafety_in_body.has_unsafe {
                        t.set();
                        with_unsafe.push(fn_info.decl_id());
                    }
                } else {
                    //TODO do not warn for std, core, alloc
                    let crate_name = cx.tcx.crate_name(*ext_crate);
                    if crate_name.as_str() != "alloc"
                        && crate_name.as_str() != "std"
                        && crate_name.as_str() != "core" {
                        println!("Error external call NOT found {:?}", ext_call);
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