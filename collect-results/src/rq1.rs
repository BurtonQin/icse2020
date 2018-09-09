use std::ffi::OsString;

use results;
use results::blocks;
use std::io::BufReader;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Write;

pub fn process_rq1(crates: &Vec<(String,String)>) {

    let output_file = ::get_output_file( "rq01" );
    let mut writer = BufWriter::new(output_file);

    for (crate_name, version) in crates {
        debug!("Crates {:?}", crate_name);
        let file_ops = results::FileOps::new( crate_name, &version );
        let file = file_ops.get_blocks_summary_file(false);
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
                debug!("Processing {:?}: {:?}", crate_name, line);
                let trimmed_line = line.trim_right();
                let block_summary: blocks::BlockSummary = serde_json::from_str(&trimmed_line).unwrap();
                writeln!(writer,"{},{},{},{},{}"
                         , crate_name
                         , block_summary.in_unsafe_bb/block_summary.total_bb
                         , block_summary.in_unsafe_bb
                         , block_summary.hir_unsafe_blocks/block_summary.hir_total
                         , block_summary.hir_unsafe_blocks
                );
            }
        }
    }

}