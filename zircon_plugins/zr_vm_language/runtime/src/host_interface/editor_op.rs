use zircon_runtime::script::{VmCallbackHandle, VmHostInterfaceError, VmPluginHostContext};

/// Registers a VM export as a three-segment editor operation.
pub fn register_editor_operation(
    host: &VmPluginHostContext,
    operation: impl Into<String>,
    module: &str,
    function: &str,
) -> Result<VmCallbackHandle, VmHostInterfaceError> {
    host.host_interfaces.register_editor_operation(
        &host.interface_caller()?,
        operation,
        module,
        function,
    )
}
