use std::sync::Arc;

use zircon_runtime::core::framework::net::{NetError, RpcDescriptor, RpcInvocationDescriptor};

use super::{NetRpcRuntimeManager, RpcHandler};

impl NetRpcRuntimeManager {
    pub fn register_rpc(&self, descriptor: RpcDescriptor) -> Result<(), NetError> {
        self.state
            .lock()
            .expect("net RPC state mutex poisoned")
            .rpc_descriptors
            .insert(descriptor.id.clone(), descriptor);
        Ok(())
    }

    pub fn register_schema_validator(
        &self,
        schema: impl Into<String>,
        validator: impl Fn(&[u8]) -> bool + Send + Sync + 'static,
    ) {
        self.state
            .lock()
            .expect("net RPC state mutex poisoned")
            .schema_validators
            .insert(schema.into(), Arc::new(validator));
    }

    pub fn register_rpc_handler(
        &self,
        descriptor: RpcDescriptor,
        handler: impl Fn(&RpcInvocationDescriptor) -> Result<Vec<u8>, String> + Send + Sync + 'static,
    ) -> Result<(), NetError> {
        let rpc_id = descriptor.id.clone();
        let mut state = self.state.lock().expect("net RPC state mutex poisoned");
        let handler: RpcHandler = Arc::new(handler);
        state.rpc_descriptors.insert(rpc_id.clone(), descriptor);
        state.rpc_handlers.insert(rpc_id, handler);
        Ok(())
    }

    pub fn rpc_descriptor(&self, id: &str) -> Option<RpcDescriptor> {
        self.state
            .lock()
            .expect("net RPC state mutex poisoned")
            .rpc_descriptors
            .get(id)
            .cloned()
    }
}
