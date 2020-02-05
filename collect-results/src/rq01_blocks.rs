use results;
use results::blocks;
use std::io::BufReader;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Write;

pub fn process_rq(crates: &Vec<(String,String)>) {

    let output_file = ::get_output_file( "rq01-blocks" );
    let mut writer = BufWriter::new(output_file);

    for (crate_name, version) in crates {
        info!("Processing Crate {:?}", crate_name);
        let dir_name = ::get_full_analysis_dir();
        let file_ops = results::FileOps::new( crate_name, &version, &dir_name );
        if let Some (files) = file_ops.open_files(results::BLOCK_SUMMARY_BB) {
            if (files.is_empty()) {
                error!("No files for crate {:?}", crate_name);
                assert!(false);
            }

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
                        if trimmed_line.len() > 0 { // ignore empty lines
                            let block_summary: blocks::BlockSummary = serde_json::from_str(&trimmed_line).unwrap();
                            writeln!(writer, "{}\t{}\t{}\t{}",
                                     block_summary.unsafe_blocks,
                                     block_summary.user_unsafe_blocks,
                                     block_summary.total,
                                     crate_name);
                        }
                    }

                }
            }
                
        } else {
            error!("Block summary files missing for crate {:?}", crate_name);
        }
    }

}
