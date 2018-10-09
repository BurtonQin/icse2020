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
}

#[derive(Serialize, Deserialize, Debug)]
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
