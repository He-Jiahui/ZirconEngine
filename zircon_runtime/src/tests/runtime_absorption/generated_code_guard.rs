//! 生成的导出模板应只承担薄适配，启动和注册行为归手写拥有者。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "generated_code_guard/behavior.rs"]
mod behavior;
#[path = "generated_code_guard/delegation.rs"]
mod delegation;
#[path = "generated_code_guard/markers.rs"]
mod markers;
#[path = "generated_code_guard/scope.rs"]
mod scope;
#[path = "generated_code_guard/split_layout.rs"]
mod split_layout;
#[path = "generated_code_guard/support.rs"]
mod support;
