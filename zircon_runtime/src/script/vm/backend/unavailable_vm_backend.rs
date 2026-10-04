//! 未接入具体 VM 的占位后端。
//!
//! `VmPluginManager` 仍可用统一后端接口装载包；装载请求进入时由这里把“后端不可用”
//! 作为错误传回，装载阶段不会创建插件实例。

use crate::script::{VmBackend, VmError, VmPluginHostContext, VmPluginInstance, VmPluginPackage};

#[derive(Debug, Default)]
pub struct UnavailableVmBackend;

impl VmBackend for UnavailableVmBackend {
    fn backend_name(&self) -> &str {
        "unavailable"
    }

    fn load_package(
        &self,
        _package: &VmPluginPackage,
        _host: &VmPluginHostContext,
    ) -> Result<Box<dyn VmPluginInstance>, VmError> {
        Err(VmError::BackendUnavailable(
            "zr_vm integration is not wired yet".to_string(),
        ))
    }
}
