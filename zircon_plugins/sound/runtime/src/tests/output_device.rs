//! 本组测试挂接相同运行时边界的夹具和行为用例；具体限制由子模块的入口断言界定。
#[path = "output_device/software_null.rs"]
mod software_null;
#[path = "output_device/validation.rs"]
mod validation;
