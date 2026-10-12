//! 结构回归测试读取可选功能辅助层的源码，固定模块职责与转发边界；行为仍由 parity 用例比较。
#[path = "structure/parser.rs"]
mod parser;
#[path = "structure/runtime.rs"]
mod runtime;
#[path = "structure/sources.rs"]
mod sources;
#[path = "structure/types.rs"]
mod types;
#[path = "structure/values.rs"]
mod values;
