use results;
use std::io::BufReader;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Write;

pub fn process_rq(crates: &Vec<(String,String)>) {
    let output_file = ::get_output_file("rq06");
    let mut writer = BufWriter::new(output_file);

    for (crate_name, version) in crates {
        info!("Processing Crate {:?}", crate_name);
        let dir_name = ::get_full_analysis_dir();
        let file_ops = results::FileOps::new( crate_name, &version, &dir_name );
        let file = file_ops.get_unsafe_calls_file(false);
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
                let res: results::calls::ExternalCall = serde_json::from_str(&trimmed_line).unwrap();
                writeln!(writer, "{:?}\t{}\t{}\t{}\t{}"
                            , res.abi
                            , res.crate_name
                            , res.def_path
                            , res.name
                             , res.user_provided
                );
            }
        }
    }
}