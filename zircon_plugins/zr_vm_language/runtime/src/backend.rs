use std::sync::Arc;

use zircon_runtime::script::{
    VmBackend, VmBackendFamily, VmError, VmPluginHostContext, VmPluginInstance, VmPluginPackage,
};

#[derive(Debug, Default)]
pub struct ZrVmBackendFamily;

impl VmBackendFamily for ZrVmBackendFamily {
    fn family_name(&self) -> &str {
        "zr_vm"
    }

    fn resolve(&self, selector: &str) -> Result<Arc<dyn VmBackend>, VmError> {
        match selector {
            "zr_vm:project" | "project" => Ok(Arc::new(ZrVmBackend)),
            other => Err(VmError::UnknownBackend(other.to_string())),
        }
    }

    fn selectors(&self) -> Vec<String> {
        vec!["zr_vm:project".to_string(), "project".to_string()]
    }
}

#[derive(Debug, Default)]
pub struct ZrVmBackend;

impl VmBackend for ZrVmBackend {
    fn backend_name(&self) -> &str {
        "zr_vm"
    }

    fn load_package(
        &self,
        package: &VmPluginPackage,
        host: &VmPluginHostContext,
    ) -> Result<Box<dyn VmPluginInstance>, VmError> {
        validate_zr_vm_project_package(package)?;
        load_project_package(package, host)
    }
}

fn validate_zr_vm_project_package(package: &VmPluginPackage) -> Result<(), VmError> {
    if package.zr_vm_project.is_none() {
        return Err(VmError::Parse(
            "zr_vm backend requires a package discovered from backend = \"zr_vm:project\""
                .to_string(),
        ));
    }
    Ok(())
}

#[cfg(feature = "real-zr-vm")]
fn load_project_package(
    package: &VmPluginPackage,
    host: &VmPluginHostContext,
) -> Result<Box<dyn VmPluginInstance>, VmError> {
    crate::real_backend::load_project_package(package, host)
}

#[cfg(not(feature = "real-zr-vm"))]
fn load_project_package(
    _package: &VmPluginPackage,
    _host: &VmPluginHostContext,
) -> Result<Box<dyn VmPluginInstance>, VmError> {
    Err(VmError::BackendUnavailable(
        "zr_vm runtime binding is disabled; build zircon_plugin_zr_vm_language_runtime with feature real-zr-vm and set ZR_VM_RUST_BINDING_LIB_DIR".to_string(),
    ))
}
