use results;
use std::io::BufReader;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Write;
use std::fs::File;

pub fn process_rq(crates: &Vec<(String,String)>) {
    let output_file = ::get_output_file("rq07_coarse_opt");
    let mut writer_opt = BufWriter::new(output_file);
    let mut writer_pes = BufWriter::new( ::get_output_file("rq07_coarse_pes"));
    for (crate_name, version) in crates {
        info!("Processing crate {:?}", crate_name);
        let dir_name = ::get_full_analysis_dir();
        let file_ops = results::FileOps::new( crate_name, &version, &dir_name );
        let file = file_ops.get_implicit_unsafe_coarse_opt_file(false);
        process_file(file, &mut writer_opt, crate_name);
        let file_pes = file_ops.get_implicit_unsafe_coarse_pes_file(false);
        process_file(file_pes, &mut writer_pes, crate_name);
    }
}

fn process_file( input_file: File, writer: &mut BufWriter<File>, crate_name: &String) {
    let mut reader = BufReader::new(input_file);
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
            let res: results::implicit::UnsafeInBody = serde_json::from_str(&trimmed_line).unwrap();
            writeln!(writer, "{}\t{}\t{}\t{}"
                     , crate_name
                     , res.def_path
                     , res.has_unsafe
                     , res.name
            );
        }
    }
}
