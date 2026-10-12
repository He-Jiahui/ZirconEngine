// 汇集本域源码快照供结构守卫检查职责归属；这些编译时文本不参与运行时清单构造。
#[path = "../../../sources/values/boolean/dependency.rs"]
mod dependency;
#[path = "../../../sources/values/boolean/feature.rs"]
mod feature;
#[path = "boolean/raw.rs"]
mod raw;

pub(in super::super::super) use dependency::*;
pub(in super::super::super) use feature::*;
pub(in super::super::super) use raw::*;
