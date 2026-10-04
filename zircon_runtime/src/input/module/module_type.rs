use crate::engine_module::{EngineModule, ModuleDescriptor};

use crate::core::framework::input::INPUT_MODULE_NAME;

use super::module_descriptor;

/// 默认输入模块入口；宿主注册并激活其描述符后，通过管理器服务取得输入状态。
#[derive(Clone, Copy, Debug, Default)]
pub struct InputModule;

impl EngineModule for InputModule {
    fn module_name(&self) -> &'static str {
        INPUT_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "High-level input routing and action maps"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}
