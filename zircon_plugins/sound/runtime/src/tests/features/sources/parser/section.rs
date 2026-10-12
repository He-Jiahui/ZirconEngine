// 汇集本域源码快照供结构守卫检查职责归属；这些编译时文本不参与运行时清单构造。
#[path = "section/kind.rs"]
mod kind;
#[path = "section/root.rs"]
mod root;
#[path = "../../../sources/parser/section/table_header.rs"]
mod table_header;

pub(in super::super::super) use kind::*;
pub(in super::super::super) use root::*;
pub(in super::super::super) use table_header::*;
