use zircon_runtime::script::{VmCallbackHandle, VmHostInterfaceError, VmPluginHostContext};

/// Registers a VM export as a behavior-tree node contribution.
pub fn register_bt_node(
    host: &VmPluginHostContext,
    id: impl Into<String>,
    display_name: impl Into<String>,
    module: &str,
    function: &str,
) -> Result<VmCallbackHandle, VmHostInterfaceError> {
    host.host_interfaces.register_behavior_node(
        &host.interface_caller()?,
        id,
        display_name,
        module,
        function,
    )
}
