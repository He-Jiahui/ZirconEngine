//! 公开脚本模块身份 ScriptModule 及其描述符；服务工厂、初始化阶段和依赖声明由子模块集中定义。

#[path = "module_descriptor.rs"]
mod module_descriptor;
#[path = "script_module.rs"]
mod script_module;

pub use module_descriptor::module_descriptor;
pub use script_module::ScriptModule;
