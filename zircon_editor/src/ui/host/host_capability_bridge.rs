use zircon_runtime::core::CoreHandle;
use zircon_runtime::script::{
    HostHandle, PluginHostDriver, PluginSlotId, VmError, VmPluginManager, VmPluginPackage,
    PLUGIN_HOST_DRIVER_NAME, VM_PLUGIN_MANAGER_NAME,
};

use super::editor_ui_host::EditorUiHost;
use super::minimal_host_contract::editor_host_minimal_contract;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EditorHostCapabilityHandle {
    capability: String,
    handle: HostHandle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorHostVmBridgeReport {
    sandbox_enabled: bool,
    loaded_capabilities: Vec<EditorHostCapabilityHandle>,
    diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditorVmExtensionLoadReport {
    loaded_slot: Option<PluginSlotId>,
    diagnostics: Vec<String>,
}

pub(super) fn register_vm_host_capabilities(
    core: &CoreHandle,
    sandbox_enabled: bool,
) -> EditorHostVmBridgeReport {
    let mut report = EditorHostVmBridgeReport::new(sandbox_enabled);
    if !sandbox_enabled {
        report
            .diagnostics
            .push("runtime sandbox disabled by EntryConfig".to_string());
        return report;
    }
    let driver = match core.resolve_driver::<PluginHostDriver>(PLUGIN_HOST_DRIVER_NAME) {
        Ok(driver) => driver,
        Err(error) => {
            report.diagnostics.push(format!(
                "failed to register editor VM host capabilities through {PLUGIN_HOST_DRIVER_NAME}: {error:?}"
            ));
            return report;
        }
    };

    let registry = driver.registry();
    for capability in editor_host_minimal_contract().minimal_capability_ids() {
        let handle = registry.register_capability(capability.clone());
        report
            .loaded_capabilities
            .push(EditorHostCapabilityHandle { capability, handle });
    }
    report
}

impl Default for EditorHostVmBridgeReport {
    fn default() -> Self {
        Self::new(true)
    }
}

impl EditorHostVmBridgeReport {
    fn new(sandbox_enabled: bool) -> Self {
        Self {
            sandbox_enabled,
            loaded_capabilities: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn sandbox_enabled(&self) -> bool {
        self.sandbox_enabled
    }

    pub fn loaded_capabilities(&self) -> Vec<String> {
        self.loaded_capabilities
            .iter()
            .map(|capability| capability.capability.clone())
            .collect()
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub fn handle_for(&self, capability: &str) -> Option<HostHandle> {
        self.loaded_capabilities
            .iter()
            .find(|record| record.capability == capability)
            .map(|record| record.handle)
    }
}

impl EditorVmExtensionLoadReport {
    pub fn loaded_slot(&self) -> Option<PluginSlotId> {
        self.loaded_slot
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }
}

impl EditorUiHost {
    pub(super) fn load_vm_extension_package(
        &self,
        package: VmPluginPackage,
    ) -> EditorVmExtensionLoadReport {
        if !self.vm_bridge_report.sandbox_enabled {
            return EditorVmExtensionLoadReport {
                loaded_slot: None,
                diagnostics: vec!["runtime sandbox disabled by EntryConfig".to_string()],
            };
        }

        let manager = match self
            .core
            .resolve_manager::<VmPluginManager>(VM_PLUGIN_MANAGER_NAME)
        {
            Ok(manager) => manager,
            Err(error) => {
                return EditorVmExtensionLoadReport {
                    loaded_slot: None,
                    diagnostics: vec![format!(
                        "failed to resolve VM extension manager {VM_PLUGIN_MANAGER_NAME}: {error:?}"
                    )],
                };
            }
        };

        match manager.load_package(package) {
            Ok(slot) => EditorVmExtensionLoadReport {
                loaded_slot: Some(slot),
                diagnostics: Vec::new(),
            },
            Err(error) => EditorVmExtensionLoadReport {
                loaded_slot: None,
                diagnostics: vec![format_vm_error(&error)],
            },
        }
    }
}

fn format_vm_error(error: &VmError) -> String {
    format!("VM extension load failed: {error:?}")
}
