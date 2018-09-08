use std::fs::File;

use fn_info::FnInfo;

use rustc::lint::LateContext;

use std::io::Write;

pub trait Analysis {
    fn is_set(&self) -> bool {
        false
    }

    fn set(&mut self) -> () {}

    fn run_analysis<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, fn_info: &'a FnInfo) -> Self;
}

pub fn save_analysis<T>(analysis_results: &Vec<(&FnInfo, T)>, file: &mut File)
where
    T: serde::ser::Serialize,
{
    for (_, t) in analysis_results.iter() {
        let serialized = serde_json::to_string(t as &T).unwrap();
        writeln!(file, "{}", serialized);
    }
}

pub fn save_summary_analysis<T>(analysis_results: T, file: &mut File)
where
    T: serde::ser::Serialize,
{
    let serialized = serde_json::to_string(&analysis_results).unwrap();
    writeln!(file, "{}", serialized);
}

pub fn save_analysis_with_fn_info<'a, 'tcx, T>(
    cx: &LateContext<'a, 'tcx>,
    analysis_results: &Vec<(&FnInfo, T)>,
    file: &mut File,
) where
    T: serde::ser::Serialize,
{
    for (fn_info, t) in analysis_results.iter() {
        let fn_short_info = fn_info.build_short_fn_info(cx);
        let serialized = serde_json::to_string(&(fn_short_info, t as &T)).unwrap();
        writeln!(file, "{}", serialized);
    }
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
