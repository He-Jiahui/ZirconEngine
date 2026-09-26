use std::sync::Arc;

use super::super::super::super::descriptors::RegistryName;

// 归属、即时启动和卸载分别消费不同顺序的服务列表；卸载按插件、管理器、驱动层级回退。
pub(in crate::core::runtime::handle::registration) struct ModuleServiceLists {
    pub(in crate::core::runtime::handle::registration) service_names: Arc<[RegistryName]>,
    pub(in crate::core::runtime::handle::registration) startup_service_names: Arc<[RegistryName]>,
    pub(in crate::core::runtime::handle::registration) shutdown_service_names: Arc<[RegistryName]>,
}
