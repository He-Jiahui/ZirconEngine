use zircon_runtime::script::VmPluginPackage;

use super::editor_capabilities::EditorCapabilitySnapshot;
use super::editor_manager::EditorManager;
use super::editor_subsystems::EditorSubsystemReport;
use super::host_capability_bridge::{EditorHostVmBridgeReport, EditorVmExtensionLoadReport};
use super::minimal_host_contract::EditorHostMinimalReport;

impl EditorManager {
    pub fn minimal_host_report(&self) -> EditorHostMinimalReport {
        self.host.minimal_report.clone()
    }

    pub fn vm_extension_capability_report(&self) -> EditorHostVmBridgeReport {
        self.host.vm_bridge_report.clone()
    }

    pub fn subsystem_report(&self) -> EditorSubsystemReport {
        self.host.lock_subsystem_report().clone()
    }

    pub fn capability_snapshot(&self) -> EditorCapabilitySnapshot {
        self.host.lock_capability_snapshot().clone()
    }

    pub fn load_vm_extension_package(
        &self,
        package: VmPluginPackage,
    ) -> EditorVmExtensionLoadReport {
        self.host.load_vm_extension_package(package)
    }
}
