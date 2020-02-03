use results;
use std::io::BufReader;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Write;


pub fn process_rq(crates: &Vec<(String,String)>, restricted: bool) {
    let output_file = ::get_output_file(if restricted {"rq01-restricted-func"} else {"rq01-func"});
    let mut writer = BufWriter::new(output_file);

    for (crate_name, version) in crates {

        info!("Processing Crate {:?}", crate_name);

        let dir_name = ::get_full_analysis_dir();
        let file_ops = results::FileOps::new( crate_name, &version, &dir_name );

        if let Some (files) = file_ops.open_files( if restricted {results::NO_REASON_FOR_UNSAFE} else {results::SUMMARY_FUNCTIONS_FILE_NAME}) {
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
                        if trimmed_line.len() > 0 {
                            let fn_summary: results::functions::Summary = serde_json::from_str(&trimmed_line).unwrap();
                            if fn_summary.total() == 0 {
                                info!("Processing {:?}: {:?}", crate_name, line);
                            } else {

                                writeln!(writer, "{}\t{}"
                                             , fn_summary.unsafe_no()
                                             , crate_name);

                            }
                        }
                    }

                }

            }
        } else {
            error!("Function summary files missing for crate {:?}", crate_name);
        }
    }
}