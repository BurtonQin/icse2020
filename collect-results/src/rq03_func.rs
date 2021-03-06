use results;
use std::io::BufReader;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Write;
use results::unsafety_sources::SourceKind;

pub fn process_rq(crates: &Vec<(String,String)>) {
    let output_file = ::get_output_file("rq03-func");
    let mut writer = BufWriter::new(output_file);
    for (crate_name, version) in crates {
        info!("Processing Crate {:?}", crate_name);
        let dir_name = ::get_full_analysis_dir();
        let file_ops = results::FileOps::new( crate_name, &version, &dir_name );
        if let Some (files) = file_ops.open_files(results::FN_UNSAFETY_SOURCES_FILE_NAME) {
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
                        let res1: serde_json::Result<results::functions::UnsafeFnUsafetySources> = serde_json::from_str(&trimmed_line);

                        if let Ok(res) = res1 {
                            for src in res.sources() {
                                writeln!(writer, "{:?},{:?},{}"
                                         , src.kind
                                         , src.user_provided
                                         , crate_name
                                );
                            }
                            if res.from_trait() {
                                writeln!(writer, "{},{:?},{}"
                                         , "From Trait"
                                         , true
                                         , crate_name
                                );
                            }
                            if res.arguments().len() > 0 {
                                writeln!(writer, "{},{:?},{}"
                                         , "Raw Pointer Argument"
                                         , true
                                         , crate_name
                                );
                            }
                        } else {
                            error!("Could not process {:?} line: {:?}", crate_name, trimmed_line);
                        }
                    }
                }
            }
        } else {
            error!("Function unsafety sources files missing for crate {:?}", crate_name);
        }
    }
}