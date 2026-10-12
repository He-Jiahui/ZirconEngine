//! 运行时吸收完成后，嵌套兼容包的移除状态由此处约束。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "compatibility_shells/nested_crates.rs"]
mod nested_crates;
#[path = "compatibility_shells/split_layout.rs"]
mod split_layout;
