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
        let dir_name = ::get_full_analysis_dir();
        let file_ops = results::FileOps::new( crate_name, &version, &dir_name );
        let file = file_ops.get_blocks_unsafety_sources_file(false);
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
                if let Ok (bus) = bus_res {
                    writeln!(writer, "{}\t{:?}\t{}\t{}"
                                , bus.block_id
                                , bus.source.kind
                                , bus.source.user_provided
                                , crate_name
                    );
                    if let  SourceKind::UnsafeFnCall(ref abi) = bus.source.kind {
                        writeln!(calls_writer, "{:?}\t{}\t{}", abi, bus.block_id, bus.source.user_provided);
                    }

                } else {
                    error!("Could not process {:?} line: {:?}", crate_name, trimmed_line);
                }
            }
        }
    }
}