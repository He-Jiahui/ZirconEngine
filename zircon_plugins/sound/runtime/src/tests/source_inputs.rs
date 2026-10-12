//! 本组测试挂接相同运行时边界的夹具和行为用例；具体限制由子模块的入口断言界定。
#[path = "source_inputs/external_lifecycle.rs"]
mod external_lifecycle;
#[path = "source_inputs/parameter_bindings.rs"]
mod parameter_bindings;
