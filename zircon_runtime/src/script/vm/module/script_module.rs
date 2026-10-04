use crate::engine_module::{EngineModule, ModuleDescriptor};

use super::module_descriptor;
use crate::script::SCRIPT_MODULE_NAME;

/// 核心运行时的脚本模块身份；服务创建和依赖声明集中在描述符，动态脚本系统随后从该模块解析管理器。
#[derive(Clone, Copy, Debug, Default)]
pub struct ScriptModule;

impl EngineModule for ScriptModule {
    fn module_name(&self) -> &'static str {
        SCRIPT_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "VM plugin hosting and hot reload"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}
