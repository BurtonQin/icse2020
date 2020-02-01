use results;
use std::io::BufReader;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Write;
use results::unsafety_sources::SourceKind;

pub fn process_rq(crates: &Vec<(String,String)>) {
    let output_file = ::get_output_file("rq04");
    let mut writer = BufWriter::new(output_file);
    let calls_file = ::get_output_file("rq04-calls");
    let mut calls_writer = BufWriter::new(calls_file);
    for (crate_name, version) in crates {

        //error!("Processing crate {:?}", crate_name);

        let dir_name = ::get_full_analysis_dir();
        let file_ops = results::FileOps::new( crate_name, &version, &dir_name );
        if let Some (files) = file_ops.open_files(results::BLOCK_UNSAFETY_SOURCES_FILE_NAME) {
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
                        let bus_res: serde_json::Result<results::blocks::BlockUnsafetySource> = serde_json::from_str(&trimmed_line);
                        if let Ok(bus) = bus_res {

                            writeln!(writer, "{},{:?},{},{}"
                                     , bus.block_id
                                     , bus.source.kind
                                     , bus.source.user_provided
                                     , crate_name
                            );
                            if let SourceKind::UnsafeFnCall(ref abi) = bus.source.kind {
                                writeln!(calls_writer, "{:?}\t{}\t{}", abi, bus.block_id, bus.source.user_provided);
                            }
                        } else {
                            error!("Could not process {:?} line: {:?}", crate_name, trimmed_line);
                        }
                    }
                }
            }
        } else {
            error!("Block unsafety sources files missing for crate {:?}", crate_name);
        }
    }
}