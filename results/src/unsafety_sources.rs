#[derive(Serialize, Deserialize, Debug)]
pub enum Abi {
    Cdecl,
    Stdcall,
    Fastcall,
    Vectorcall,
    Thiscall,
    Aapcs,
    Win64,
    SysV64,
    PtxKernel,
    Msp430Interrupt,
    X86Interrupt,
    AmdGpuKernel,
    Rust,
    C,
    System,
    RustIntrinsic,
    RustCall,
    PlatformIntrinsic,
    Unadjusted,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum FnCallInfo {
    Local(String,Abi), // fn_name, abi
    External(String, String, Abi), // crate_name, fn_name, abi
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Source {
    pub loc: String,
    pub kind: SourceKind,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SourceKind {
    UnsafeFnCall(FnCallInfo),
    DerefRawPointer(String), // TODO find a better solution
    Asm,
    Static(String),
    //ForeignItem, //TODO check what is this
    BorrowPacked,
    AssignmentToNonCopyUnionField(String),
    AccessToUnionField(String),
    ExternStatic(String),
}