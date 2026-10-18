use std::path::Path;

use thiserror::Error;
use zircon_plugin_zr_vm_language_runtime::build_zr_vm_native_host_modules;
use zircon_runtime::script::{
    register_builtin_host_modules, CapabilitySet, HostExportRegistry, HostRegistry,
};

use crate::{
    inspect_project, read_toml, VmTickError, WocHostRole, WocProjectIdentity,
    WocProjectIdentityError, ZrVmProjectOptions, ZrVmProjectVm,
};

const ALLOWED_CAPABILITIES: &[&str] = &["foundation.log", "math.scalar"];

#[derive(Debug, Error)]
pub enum WocEngineHostError {
    #[error(transparent)]
    Identity(#[from] WocProjectIdentityError),
    #[error("invalid WOC capability declaration")]
    InvalidCapabilities,
    #[error("WOC host does not grant capability {0}")]
    CapabilityDenied(String),
    #[error("WOC Runtime host registration failed: {0}")]
    Registration(String),
    #[error("WOC project VM loading failed: {0:?}")]
    Vm(VmTickError),
}

/// Loads the authored project with Runtime's canonical capability-checked host
/// exports. The caller activates the VM before creating a transactional session.
pub fn load_engine_project_vm(
    project_root: impl AsRef<Path>,
    role: WocHostRole,
    options: ZrVmProjectOptions,
) -> Result<(WocProjectIdentity, ZrVmProjectVm), WocEngineHostError> {
    let root = project_root.as_ref();
    let identity = inspect_project(root, role)?;
    let package_root = root.join("scripts/woc_game");
    let manifest: toml::Value = read_toml(&package_root.join("plugin.toml"))?;
    let requested = manifest
        .get("capabilities")
        .and_then(|section| section.get("capabilities"))
        .and_then(toml::Value::as_array)
        .ok_or(WocEngineHostError::InvalidCapabilities)?;
    let mut capabilities = CapabilitySet::default();
    for requested in requested {
        let capability = requested
            .as_str()
            .ok_or(WocEngineHostError::InvalidCapabilities)?;
        if !ALLOWED_CAPABILITIES.contains(&capability) {
            return Err(WocEngineHostError::CapabilityDenied(capability.to_owned()));
        }
        capabilities = capabilities.with(capability);
    }
    let registry = HostRegistry::default();
    let exports = HostExportRegistry::new(registry.clone());
    register_builtin_host_modules(&exports, &registry)
        .map_err(|error| WocEngineHostError::Registration(error.to_string()))?;
    let modules = build_zr_vm_native_host_modules(&exports, &capabilities)
        .map_err(|error| WocEngineHostError::Registration(error.to_string()))?;
    let vm = ZrVmProjectVm::load(package_root.join(&identity.zr_vm_project), options, modules)
        .map_err(WocEngineHostError::Vm)?;
    Ok((identity, vm))
}
