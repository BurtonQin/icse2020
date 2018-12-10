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
        if let Some (files) = file_ops.open_files(results::UNSAFE_CALLS) {
            for file in files.iter() {
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
                        let res1: serde_json::Result<results::calls::ExternalCall> = serde_json::from_str(&trimmed_line);
                        if let Ok(res) = res1 {
                            writeln!(writer, "{:?}\t{}\t{}\t{}\t{}"
                                     , res.abi
                                     , res.crate_name
                                     , res.def_path
                                     , res.name
                                     , res.user_provided
                            );
                        } else {
                            error!("Could not process {:?} line: {:?}", crate_name, trimmed_line);
                        }
                    }
                }
            }
        } else {
            error!("Unsafe function calls files missing for crate {:?}", crate_name);
        }
    }
}