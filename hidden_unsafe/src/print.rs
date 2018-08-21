use rustc::lint::LateContext;
use std::fs::File;

pub static ROOT_DIR: &'static str = "/tmp/unsafe_analysis/";

pub trait Print {
    fn print<'a, 'tcx>(&self, cx: &LateContext<'a, 'tcx>, file: &mut File) -> ();
}

pub struct EmptyPrinter {}

impl Print for EmptyPrinter {
    fn print<'a, 'tcx>(&self, _cx: &LateContext<'a, 'tcx>, _file: &mut File) -> () {}
}
