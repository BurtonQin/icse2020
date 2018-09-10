use results;
use results::blocks;
use std::io::BufReader;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Write;

pub fn process_rq(crates: &Vec<(String,String)>) {

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
                let trimmed_line = line.trim_right();
                let block_summary: blocks::BlockSummary = serde_json::from_str(&trimmed_line).unwrap();
                if block_summary.total_bb != 0
                    && block_summary.hir_total != 0 {
                    writeln!(writer, "{}\t{:.2}\t{}\t{:.2}\t{}"
                             , crate_name
                             , block_summary.in_unsafe_bb as f32 / block_summary.total_bb as f32
                             , block_summary.in_unsafe_bb
                             , block_summary.hir_unsafe_blocks as f32 / block_summary.hir_total as f32
                             , block_summary.hir_unsafe_blocks
                    );
                } else {
                    error!("Processing {:?}: {:?}", crate_name, line);
                }
            }
        }
    }

}