use results;
use std::io::BufReader;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Write;


pub fn process_rq(crates: &Vec<(String,String)>) {
    let output_file = ::get_output_file("rq02");
    let mut writer = BufWriter::new(output_file);

    for (crate_name, version) in crates {
        let file_ops = results::FileOps::new( crate_name, &version );
        let file = file_ops.get_summary_functions_file(false);
        let mut reader = BufReader::new(file);
        //read line by line
        loop {
            let mut line = String::new();
            let len = reader.read_line(&mut line).expect("Error reading file");
            if len == 0 {
                //EOF reached
                break;
            } else {
                //process line
                let trimmed_line = line.trim_right();
                let fn_summary: results::functions::Summary = serde_json::from_str(&trimmed_line).unwrap();
                if fn_summary.total() == 0 {
                    error!("Processing {:?}: {:?}", crate_name, line);
                } else {
                    writeln!(writer, "{}\t{:.2}\t{}"
                             , crate_name
                             , fn_summary.unsafe_no() as f32 / fn_summary.total() as f32
                             , fn_summary.total());
                }
            }
        }
    }
}