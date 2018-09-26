use results;
use std::io::BufReader;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Write;
use results::unsafety_sources::SourceKind;
use results::functions::ShortFnInfo;

struct SourceSummary {
    pub unsafe_fn_calls: bool,
    pub raw_ptr: bool,
    pub asm: bool,
    pub static_access: bool,
    pub borrow_packed: bool,
    pub assignment_union: bool,
    pub union: bool,
    pub extern_static: bool,
    pub argument: bool,
    pub from_trait: bool,
}

impl SourceSummary {
    pub fn new() -> Self {
        Self{ unsafe_fn_calls: false
            , raw_ptr: false
            , asm: false
            , static_access: false
            , borrow_packed: false
            , assignment_union: false
            , union: false
            , extern_static: false
            , argument : false
            , from_trait: false
        }
    }

    pub fn has_reason(&self) -> bool {
        self.unsafe_fn_calls ||
            self.raw_ptr ||
            self.asm ||
            self.static_access ||
            self.borrow_packed ||
            self.assignment_union ||
            self.union ||
            self.extern_static ||
            self.argument ||
            self.from_trait
    }

    pub fn save(&self) {
        let output_file = ::get_output_file("rqfalse4-summary");
        let mut writer = BufWriter::new(output_file);
        writeln!(writer, "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}"
                 , self.has_reason()
                 , self.unsafe_fn_calls
                 , self.raw_ptr
                 , self.asm
                 , self.static_access
                 , self.borrow_packed
                 , self.assignment_union
                 , self.union
                 , self.extern_static
                 , self.argument
                 , self.from_trait
        );
    }

}


pub fn process_rq(crates: &Vec<(String,String)>) {
    let output_file = ::get_output_file("rq09");
    let mut writer = BufWriter::new(output_file);

    let mut summary = SourceSummary::new();

    for (crate_name, version) in crates {
        let dir_name = ::get_full_analysis_dir();
        let file_ops = results::FileOps::new( crate_name, &version, &dir_name );
        let file = file_ops.get_fn_unsafety_sources_file(false);
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
                let res: results::functions::UnsafeFnUsafetySources = serde_json::from_str(&trimmed_line).unwrap();
                let mut fn_summary = SourceSummary::new();
                fn_summary.from_trait = res.from_trait();
                fn_summary.argument = !res.arguments().is_empty();

                for src in res.sources() {
                    match src.kind {
                        SourceKind::UnsafeFnCall(_) => {fn_summary.unsafe_fn_calls=true;},
                        SourceKind::DerefRawPointer => {fn_summary.raw_ptr=true;},
                        SourceKind::Asm => {fn_summary.asm=true;},
                        SourceKind::Static => {fn_summary.static_access=true;},
                        SourceKind::BorrowPacked => {fn_summary.borrow_packed=true;},
                        SourceKind::AssignmentToNonCopyUnionField => {fn_summary.assignment_union=true;},
                        SourceKind::AccessToUnionField => {fn_summary.union=true;},
                        SourceKind::ExternStatic => {fn_summary.extern_static=true;},
                    }
                }
                writeln!(writer, "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}"
                         , crate_name
                         , res.name()
                         , fn_summary.unsafe_fn_calls
                         , fn_summary.raw_ptr
                         , fn_summary.asm
                         , fn_summary.static_access
                         , fn_summary.borrow_packed
                         , fn_summary.assignment_union
                         , fn_summary.union
                         , fn_summary.extern_static
                         , fn_summary.argument
                         , fn_summary.from_trait
                );
            }
        }
    }
}