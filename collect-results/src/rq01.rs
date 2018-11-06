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

        info!("Processing Crate {:?}", crate_name);

        let dir_name = ::get_full_analysis_dir();
        let file_ops = results::FileOps::new( crate_name, &version, &dir_name );
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
                writeln!(writer, "{}\t{}", block_summary.unsafe_blocks, crate_name);

            }
        }
    }

}