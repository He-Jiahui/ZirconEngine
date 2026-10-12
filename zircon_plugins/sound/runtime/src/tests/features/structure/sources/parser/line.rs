// 汇集本域源码快照供结构守卫检查职责归属；这些编译时文本不参与运行时清单构造。
#[path = "../../../../sources/parser/line/dependency.rs"]
mod dependency;
#[path = "../../../sources/parser/line/feature.rs"]
mod feature;
#[path = "../../../sources/parser/line/field.rs"]
mod field;
#[path = "../../../sources/parser/line/module.rs"]
mod module;

pub(in super::super::super) use dependency::*;
pub(in super::super::super) use feature::*;
pub(in super::super::super) use field::*;
pub(in super::super::super) use module::*;
