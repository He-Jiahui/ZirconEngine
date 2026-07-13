use zircon_runtime::script::{
    VmCallbackHandle, VmHostInterfaceError, VmPluginHostContext, VmSystemStage,
};

/// Registers a VM export for conservative execution in a supported scheduler stage.
pub fn register_system(
    host: &VmPluginHostContext,
    id: impl Into<String>,
    stage: &str,
    module: &str,
    function: &str,
) -> Result<VmCallbackHandle, VmHostInterfaceError> {
    let stage = VmSystemStage::parse(stage)
        .ok_or_else(|| VmHostInterfaceError::InvalidSystemStage(stage.to_string()))?;
    host.host_interfaces
        .register_system(&host.interface_caller()?, id, stage, module, function)
}
