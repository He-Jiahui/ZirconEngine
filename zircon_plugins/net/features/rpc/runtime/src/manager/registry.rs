//! 注册 RPC 描述符、schema validator 与 handler，供调用准入检查和同步处理。
//! 用户 validator 与 handler 均在状态锁外执行；准入会在回锁后核验 descriptor 和 validator 版本。

use std::sync::Arc;

use zircon_runtime::core::framework::net::{NetError, RpcDescriptor, RpcInvocationDescriptor};

use super::{NetRpcRuntimeManager, RpcHandler};

impl NetRpcRuntimeManager {
    pub fn register_rpc(&self, descriptor: RpcDescriptor) -> Result<(), NetError> {
        self.lock_state()?
            .rpc_descriptors
            .insert(descriptor.id.clone(), descriptor);
        Ok(())
    }

    pub fn register_schema_validator(
        &self,
        schema: impl Into<String>,
        validator: impl Fn(&[u8]) -> bool + Send + Sync + 'static,
    ) -> Result<(), NetError> {
        self.lock_state()?
            .schema_validators
            .insert(schema.into(), Arc::new(validator));
        Ok(())
    }

    /// 同时登记描述符与同步 handler；handler 会在状态锁外执行，可访问 manager，但无法被超时强制中断。
    pub fn register_rpc_handler(
        &self,
        descriptor: RpcDescriptor,
        handler: impl Fn(&RpcInvocationDescriptor) -> Result<Vec<u8>, String> + Send + Sync + 'static,
    ) -> Result<(), NetError> {
        let rpc_id = descriptor.id.clone();
        let mut state = self.lock_state()?;
        let handler: RpcHandler = Arc::new(handler);
        state.rpc_descriptors.insert(rpc_id.clone(), descriptor);
        state.rpc_handlers.insert(rpc_id, handler);
        Ok(())
    }

    pub fn rpc_descriptor(&self, id: &str) -> Option<RpcDescriptor> {
        self.lock_state()
            .ok()?
            .rpc_descriptors
            .get(id)
            .cloned()
    }
}
