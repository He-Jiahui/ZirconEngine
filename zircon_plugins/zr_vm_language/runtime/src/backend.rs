use std::sync::{Arc, LazyLock};

use zircon_runtime::script::{
    VmBackend, VmBackendFamily, VmError, VmPluginHostContext, VmPluginInstance, VmPluginPackage,
};

static ZR_VM_BACKEND: LazyLock<Arc<dyn VmBackend>> = LazyLock::new(|| Arc::new(ZrVmBackend));

#[derive(Debug, Default)]
pub struct ZrVmBackendFamily;

impl VmBackendFamily for ZrVmBackendFamily {
    fn family_name(&self) -> &str {
        "zr_vm"
    }

    fn resolve(&self, selector: &str) -> Result<Arc<dyn VmBackend>, VmError> {
        match selector {
            "zr_vm:project" | "project" => Ok(Arc::clone(&ZR_VM_BACKEND)),
            other => Err(VmError::UnknownBackend(other.to_string())),
        }
    }

    fn visit_selectors(&self, visitor: &mut dyn FnMut(&str)) {
        visitor("zr_vm:project");
        visitor("project");
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

#[cfg(feature = "backend-zr-vm")]
fn load_project_package(
    package: &VmPluginPackage,
    host: &VmPluginHostContext,
) -> Result<Box<dyn VmPluginInstance>, VmError> {
    crate::real_backend::load_project_package(package, host)
}

#[cfg(not(feature = "backend-zr-vm"))]
fn load_project_package(
    _package: &VmPluginPackage,
    _host: &VmPluginHostContext,
) -> Result<Box<dyn VmPluginInstance>, VmError> {
    Err(VmError::BackendUnavailable(
        "zr_vm runtime binding is disabled; build zircon_plugin_zr_vm_language_runtime with feature backend-zr-vm and set ZR_VM_RUST_BINDING_LIB_DIR".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{VmBackendFamily, ZrVmBackendFamily};

    #[test]
    fn zr_vm_backend_resolutions_share_arc_storage() {
        let family = ZrVmBackendFamily;
        let canonical = family.resolve("zr_vm:project").unwrap();
        let alias = family.resolve("project").unwrap();

        assert!(Arc::ptr_eq(&canonical, &alias));
    }
}
