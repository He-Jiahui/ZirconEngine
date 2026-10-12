// 集中暴露源码快照常量，结构断言只依赖这些静态输入，不实例化运行时插件。
#[path = "sources/parser.rs"]
mod parser;
#[path = "sources/runtime.rs"]
mod runtime;
#[path = "sources/types.rs"]
mod types;
#[path = "sources/values.rs"]
mod values;

pub(super) use parser::*;
pub(super) use runtime::*;
pub(super) use types::*;
pub(super) use values::*;
