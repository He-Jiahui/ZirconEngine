//! 脚本子系统归入运行时后，旧独立包的移除状态由此处约束。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "script_absorption/legacy_crate.rs"]
mod legacy_crate;
#[path = "script_absorption/split_layout.rs"]
mod split_layout;
