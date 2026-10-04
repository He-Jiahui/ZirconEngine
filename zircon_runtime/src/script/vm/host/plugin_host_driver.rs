//! 模块启动时此驱动建立宿主能力与导出注册表，并一次性安装内置模块；管理器从同一驱动取得注册表以维持调用表身份。
use crate::script::{register_builtin_host_modules, HostExportRegistry, HostRegistry};

#[derive(Debug)]
pub struct PluginHostDriver {
    registry: HostRegistry,
    host_exports: HostExportRegistry,
}

impl Default for PluginHostDriver {
    fn default() -> Self {
        let registry = HostRegistry::default();
        let host_exports = HostExportRegistry::new(registry.clone());
        register_builtin_host_modules(&host_exports, &registry)
            .expect("builtin script host modules should be valid");
        Self {
            registry,
            host_exports,
        }
    }
}

impl PluginHostDriver {
    pub fn registry(&self) -> HostRegistry {
        self.registry.clone()
    }

    pub fn host_exports(&self) -> HostExportRegistry {
        self.host_exports.clone()
    }
}
