use results;
use std::io::BufReader;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Write;
use results::unsafety_sources::SourceKind;

struct SourceSummary {
    pub unsafe_fn_calls: usize,
    pub raw_ptr: usize,
    pub asm: usize,
    pub static_access: usize,
    pub borrow_packed: usize,
    pub assignment_union: usize,
    pub union: usize,
    pub extern_static: usize,
}

impl SourceSummary {
    pub fn new() -> Self {
        Self{ unsafe_fn_calls: 0
            , raw_ptr: 0
            , asm: 0
            , static_access: 0
            , borrow_packed: 0
            , assignment_union: 0
            , union: 0
            , extern_static: 0}
    }

}


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
                let all: results::blocks::BlockUnsafetySourcesAnalysis = serde_json::from_str(&trimmed_line).unwrap();
                for (block, sources) in all.sources() {
                    let mut block_sources = SourceSummary::new();
                    for src in sources {
                        match src.kind {
                            SourceKind::UnsafeFnCall(ref abi) => {
                                block_sources.unsafe_fn_calls+=1;
                                writeln!(calls_writer, "{:?}\t{}", abi, block);
                            },
                            SourceKind::DerefRawPointer => {block_sources.raw_ptr+=1;},
                            SourceKind::Asm => {block_sources.asm+=1;},
                            SourceKind::Static => {block_sources.static_access+=1;},
                            SourceKind::BorrowPacked => {block_sources.borrow_packed+=1;},
                            SourceKind::AssignmentToNonCopyUnionField => {block_sources.assignment_union+=1;},
                            SourceKind::AccessToUnionField => {block_sources.union+=1;},
                            SourceKind::ExternStatic => {block_sources.extern_static+=1;},
                        }
                    }
                    writeln!(writer, "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}"
                             , block_sources.unsafe_fn_calls
                             , block_sources.raw_ptr
                             , block_sources.asm
                             , block_sources.static_access
                             , block_sources.borrow_packed
                             , block_sources.assignment_union
                             , block_sources.union
                             , block_sources.extern_static
                             , crate_name
                             , block
                    );
                }

            }
        }
    }
}