use analysis::Analysis;
use fn_info::FnInfo;
use rustc::hir;
use rustc::lint::LateContext;
use rustc::mir::visit::Visitor;
use rustc::mir::{BasicBlock, Location, Operand, Terminator, TerminatorKind};
use rustc::ty::TyKind;
use std::fs::File;
use std::io::Write;



