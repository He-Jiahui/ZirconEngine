#[path = "../../../parser/pending/dependency/append.rs"]
mod append;
#[path = "../../../parser/dependency/sources_entry.rs"]
mod entry;
#[path = "dependency/root.rs"]
mod root;
#[path = "../../../parser/pending/dependency/signature.rs"]
mod signature;

pub(in super::super::super::super) use append::*;
pub(in super::super::super::super) use entry::*;
pub(in super::super::super::super) use root::*;
pub(in super::super::super::super) use signature::*;
