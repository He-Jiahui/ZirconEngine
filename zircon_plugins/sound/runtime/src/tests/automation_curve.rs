//! 本组测试挂接相同运行时边界的夹具和行为用例；具体限制由子模块的入口断言界定。
#[path = "automation_curve/sampling.rs"]
mod sampling;
#[path = "automation_curve/timeline_loop.rs"]
mod timeline_loop;
#[path = "automation_curve/validation.rs"]
mod validation;
