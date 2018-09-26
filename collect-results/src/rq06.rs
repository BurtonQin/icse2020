use results;
use std::io::BufReader;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Write;


pub fn process_rq(crates: &Vec<(String,String)>) {
    let output_file = ::get_output_file("rq06");
    let mut writer = BufWriter::new(output_file);

    for (crate_name, version) in crates {
        let dir_name = ::get_full_analysis_dir();
        let file_ops = results::FileOps::new( crate_name, &version, &dir_name );
        let file = file_ops.get_no_reason_for_unsafety_file(false);
        let mut reader = BufReader::new(file);
        let mut counter = 0;
        //read line by line
        loop {
            let mut line = String::new();
            let len = reader.read_line(&mut line).expect("Error reading file");
            if len == 0 {
                //EOF reached
                break;
            } else {
                //process line
                counter += 1;
            }
        }
        writeln!(writer, "{}\t{}"
                 , crate_name
                 , counter);
    }
}