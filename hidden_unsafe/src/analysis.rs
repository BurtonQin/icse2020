use fn_info::FnInfo;
use rustc::lint::LateContext;

pub trait Analysis {

    fn is_set(&self) -> bool {
        false
    }

    fn set(&mut self) -> () {}

    fn run_analysis<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, fn_info: &'a FnInfo) -> Self;
}

pub fn run_all<'a, 'tcx, T: Analysis>(
    cx: &LateContext<'a, 'tcx>,
    data: &'a Vec<FnInfo>,
    propagate: bool,
) -> Vec<(&'a FnInfo, T)> {
    let mut result = Vec::new();
    for fn_info in data {
        let fn_res = Analysis::run_analysis(cx, fn_info);
        result.push((fn_info, fn_res));
    }
    if propagate {
        propagate_predicate(&mut result);
    }
    result
}

// TODO change this to an efficient algorithm
fn propagate_predicate<T: Analysis>(graph: &mut Vec<(&FnInfo, T)>) {
    let mut changes = true;
    let mut with_unsafe = Vec::new();
    {
        // to pass borrow checker
        for (ref fn_info, ref mut t) in graph.iter_mut() {
            if t.is_set() {
                with_unsafe.push(fn_info.decl_id());
            }
        }
    }
    while changes {
        changes = false;
        for &mut (ref fn_info, ref mut t) in graph.iter_mut() {
            if !t.is_set() {
                if (&fn_info.local_calls())
                    .into_iter()
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
