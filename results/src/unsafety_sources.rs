use  std::fmt;

use Abi;

#[derive(Serialize, Deserialize, Debug)]
pub enum FnCallInfo {
    Local(String, Abi),            // fn_name, abi
    External(String, String, Abi), // crate_name, fn_name, abi
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Source {
    pub loc: String,
    pub kind: SourceKind,
    pub user_provided: bool,
}

#[derive(Serialize, Deserialize)]
pub enum SourceKind {
    UnsafeFnCall(Abi),
    DerefRawPointer,
    Asm,
    Static,
    BorrowPacked,
    AssignmentToNonCopyUnionField,
    AccessToUnionField,
    ExternStatic,
}


impl fmt::Debug for SourceKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SourceKind::BorrowPacked => {write!(f, "Borrow Packed")}
            SourceKind::AssignmentToNonCopyUnionField => {write!(f, "Assign to Union")}
            SourceKind::AccessToUnionField => {write!(f, "Access to Union")}
            SourceKind::ExternStatic => {write!(f, "Extern Static Variable")}
            SourceKind::UnsafeFnCall(_) => {write!(f, "Unsafe Function Call")}
            SourceKind::DerefRawPointer => {write!(f, "Derefence Raw Pointer")}
            SourceKind::Asm => {write!(f, "Assembly")}
            SourceKind::Static => {write!(f, "Global Variable")}
        }

    }
}