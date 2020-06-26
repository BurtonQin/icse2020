use results;
use std::io::BufReader;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Write;
use std::fs::File;

pub fn process_rq(crates: &Vec<(String,String)>) {
    process_rq_count(crates, true);
    process_rq_count(crates, false);
    process_rq_impls(crates);
}

pub fn process_rq_count(crates: &Vec<(String,String)>, traits:bool) {
    let output_file = ::get_output_file(if traits {"rq01-traits"} else {"rq01-impls"});
    let mut writer = BufWriter::new(output_file);

    for (crate_name, version) in crates {
        info!("traits::Processing crate {:?}", crate_name);
        let dir_name = ::get_full_analysis_dir();
        let file_ops = results::FileOps::new(crate_name, &version, &dir_name);
        if let Some (files) =
            if traits {
                file_ops.open_files(results::UNSAFE_TRAITS)
            } else {
                file_ops.open_files(results::UNSAFE_TRAITS_IMPLS)
            } {
            let mut counter = 0;
            for file in files.iter() {
                let mut reader = BufReader::new(file);
                //read line by line

                loop {
                    let mut line = String::new();
                    let len = reader.read_line(&mut line).expect("Error reading file");
                    if len == 0 {
                        //EOF reached
                        break
                    } else {
                        //process line
                        if line.len() > 0 {
                            counter += 1;
                        }
                    }
                }
            }// for
            writeln!(writer, "{}\t{}"
                     , crate_name
                     , counter
            );
        } else {
            if traits {
                error!("Unsafe traits files missing for crate {:?}", crate_name);
            } else {
                error!("Unsafe trait impls files missing for crate {:?}", crate_name);
            }

        }
    }
}

pub fn process_rq_impls(crates: &Vec<(String,String)>) {
    let output_file = ::get_output_file("rq01-impls-names");
    let mut writer = BufWriter::new(output_file);

    for (crate_name, version) in crates {
        info!("impls::Processing crate {:?}", crate_name);
        let dir_name = ::get_full_analysis_dir();
        let file_ops = results::FileOps::new(crate_name, &version, &dir_name);
        if let Some (files) = file_ops.open_files(results::UNSAFE_TRAITS_IMPLS) {
            for file in files.iter() {
                let mut reader = BufReader::new(file);
                //read line by line
                let mut counter = 0;
                loop {
                    let mut line = String::new();
                    let len = reader.read_line(&mut line).expect("Error reading file");
                    if len == 0 {
                        //EOF reached
                        break
                    } else {
                        //process line
                        let trimmed_line = line.trim_right();
                        let res: results::traits::UnsafeTrait = serde_json::from_str(&trimmed_line).unwrap();
                        writeln!(writer, "{}\t{}"
                                 , crate_name
                                 , res.name
                        );
                    }
                }
            }
        }
    }
}

