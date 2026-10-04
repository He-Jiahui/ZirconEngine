//! 保存组件复制描述符作为快照、调度、表编译的共同配置源。
//! 重新登记同名组件会替换描述符；与既有快照的 schema 兼容性由调用方负责。

use zircon_runtime::core::framework::net::SyncComponentDescriptor;

use super::NetReplicationRuntimeManager;

impl NetReplicationRuntimeManager {
    pub(in crate::manager) fn register_component_impl(&self, descriptor: SyncComponentDescriptor) {
        self.state
            .lock()
            .expect("net replication state mutex poisoned")
            .descriptors
            .insert(descriptor.component_type.clone(), descriptor);
    }
}
