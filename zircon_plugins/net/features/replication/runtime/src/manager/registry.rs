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
