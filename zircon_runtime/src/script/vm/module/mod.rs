//! 公开脚本模块身份 ScriptModule 及其描述符；服务工厂、初始化阶段和依赖声明由子模块集中定义。

mod module_descriptor;
mod script_module;

pub use module_descriptor::module_descriptor;
pub use script_module::ScriptModule;
