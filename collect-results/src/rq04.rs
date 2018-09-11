use results;
use std::io::BufReader;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Write;
use results::unsafety_sources::SourceKind;
use results::functions::ShortFnInfo;

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

    pub fn total(&self) -> usize {
        self.unsafe_fn_calls +
            self.raw_ptr +
            self.asm +
            self.static_access +
            self.borrow_packed +
            self.assignment_union +
            self.union +
            self.extern_static
    }

    pub fn save(&self) {
        let output_file = ::get_output_file("rq04-summary");
        let mut writer = BufWriter::new(output_file);
        writeln!(writer, "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}"
                 , self.total()
                 , self.unsafe_fn_calls
                 , self.raw_ptr
                 , self.asm
                 , self.static_access
                 , self.borrow_packed
                 , self.assignment_union
                 , self.union
                 , self.extern_static
        );
    }

    pub fn add(&mut self, other: &SourceSummary) {
        self.unsafe_fn_calls += other.unsafe_fn_calls;
        self.raw_ptr += other.raw_ptr;
        self.asm += other.asm;
        self.static_access += other.static_access;
        self.borrow_packed += other.borrow_packed;
        self.assignment_union += other.assignment_union;
        self.union += other.union;
        self.extern_static += other.extern_static;
    }
}


pub fn process_rq(crates: &Vec<(String,String)>) {
    let output_file = ::get_output_file("rq04-fn");
    let mut writer = BufWriter::new(output_file);

    let mut summary = SourceSummary::new();

    for (crate_name, version) in crates {
        let file_ops = results::FileOps::new( crate_name, &version );
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
                let all: (ShortFnInfo,results::blocks::BlockUnsafetySourcesAnalysis) = serde_json::from_str(&trimmed_line).unwrap();
                let fn_name = all.0.name();
                let mut fn_summary = SourceSummary::new();
                for (_, sources) in all.1.sources() {
                    for src in sources {
                        match src.kind {
                            SourceKind::UnsafeFnCall(_) => {fn_summary.unsafe_fn_calls+=1;},
                            SourceKind::DerefRawPointer(_) => {fn_summary.raw_ptr+=1;},
                            SourceKind::Asm => {fn_summary.asm+=1;},
                            SourceKind::Static(_) => {fn_summary.static_access+=1;},
                            SourceKind::BorrowPacked => {fn_summary.borrow_packed+=1;},
                            SourceKind::AssignmentToNonCopyUnionField(_) => {fn_summary.assignment_union+=1;},
                            SourceKind::AccessToUnionField(_) => {fn_summary.union+=1;},
                            SourceKind::ExternStatic(_) => {fn_summary.extern_static+=1;},
                        }
                    }
                    writeln!(writer, "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}"
                             , crate_name
                             , fn_name
                             , fn_summary.unsafe_fn_calls
                             , fn_summary.raw_ptr
                             , fn_summary.asm
                             , fn_summary.static_access
                             , fn_summary.borrow_packed
                             , fn_summary.assignment_union
                             , fn_summary.union
                             , fn_summary.extern_static
                    );
                    summary.add(&fn_summary);
                }

            }
        }
    }
    // save summary
    summary.save();
}