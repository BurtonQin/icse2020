use rustc::lint::LateContext;

pub trait Print {
    fn print<'a, 'tcx>(&self, cx: &LateContext<'a, 'tcx>) -> ();
}

pub struct EmptyPrinter {}

impl Print for EmptyPrinter {
    fn print<'a, 'tcx>(&self, _cx: &LateContext<'a, 'tcx>) -> () {}
}
