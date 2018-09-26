use results;
use std::io::BufReader;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Write;
use std::collections::BTreeMap;
use results::functions;

pub fn process_rq(crates: &Vec<(String,String)>) {
    let mut map = BTreeMap::new();

    for (crate_name, version) in crates {
        let dir_name = ::get_full_analysis_dir();
        let file_ops = results::FileOps::new( crate_name, &version, &dir_name );
        let file = file_ops.get_external_calls_summary_file(false);

        let mut reader = BufReader::new(file);
        let mut line = String::new();
        let len = reader.read_line(&mut line).expect("Error reading file");
        if len == 0 {
            //EOF reached
            break;
        } else {
            //process line
            let trimmed_line = line.trim_right();
            let calls_summary: functions::ExternalCallsSummary = serde_json::from_str(&trimmed_line).unwrap();
            for call in calls_summary.calls() {
                let count = map.entry(call.0.clone()).or_insert(0 as usize);
                *count = *count + call.1;
            }
        }
    }

    // write
    let output_file = ::get_output_file( "rq05" );
    let mut writer = BufWriter::new(output_file);
    for (call,count) in map {
        writeln!(writer
                 , "{}\t{}"
                 , call
                 , count);
    }


}