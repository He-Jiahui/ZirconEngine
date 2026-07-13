use zircon_runtime::core::framework::net::RpcPayloadSchema;
use zircon_runtime::script::{VmCallbackHandle, VmHostInterfaceError, VmPluginHostContext};

/// Registers a VM export as an RPC handler with a reflection payload schema.
pub fn register_rpc_handler(
    host: &VmPluginHostContext,
    id: impl Into<String>,
    payload_schema: RpcPayloadSchema,
    module: &str,
    function: &str,
) -> Result<VmCallbackHandle, VmHostInterfaceError> {
    host.host_interfaces.register_rpc_handler(
        &host.interface_caller()?,
        id,
        payload_schema,
        module,
        function,
    )
}
