use crate::core::framework::bridge::{BridgeOwnerTransitionMode, InterfaceSlot};
use crate::plugin::{
    BridgeOwnerTransitionReport, FrozenBridgeTable, PluginModuleKind, RuntimeExtensionRegistry,
};

use super::bridge_dependencies::RuntimePluginBridgeDisableBlocker;
use super::RuntimePluginCatalog;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginBridgeLifecycleReport {
    pub provider_package_id: String,
    pub mode: BridgeOwnerTransitionMode,
    pub owner_reports: Vec<BridgeOwnerTransitionReport>,
}

impl RuntimePluginBridgeLifecycleReport {
    pub fn affected_slots(&self) -> Vec<InterfaceSlot> {
        self.owner_reports
            .iter()
            .flat_map(|report| report.affected_slots.iter().copied())
            .collect()
    }

    pub fn affected_slot_count(&self) -> usize {
        self.owner_reports
            .iter()
            .map(|report| report.affected_slots.len())
            .sum()
    }

    pub fn diagnostic(&self) -> String {
        format!(
            "bridge.provider_lifecycle: provider plugin `{}` {:?} affected {} owner(s), {} interface(s)",
            self.provider_package_id,
            self.mode,
            self.owner_reports.len(),
            self.affected_slot_count()
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginBridgeLifecycleBlock {
    pub provider_package_id: String,
    pub mode: BridgeOwnerTransitionMode,
    pub blockers: Vec<RuntimePluginBridgeDisableBlocker>,
}

impl RuntimePluginBridgeLifecycleBlock {
    pub fn diagnostic(&self) -> String {
        format!(
            "bridge.provider_lifecycle_blocked: provider plugin `{}` {:?} blocked by {} strong dependent(s): {}",
            self.provider_package_id,
            self.mode,
            self.blockers.len(),
            self.blockers
                .iter()
                .map(RuntimePluginBridgeDisableBlocker::diagnostic)
                .collect::<Vec<_>>()
                .join("; ")
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimePluginBridgeLifecycleError {
    StrongDependentsBlocked(RuntimePluginBridgeLifecycleBlock),
}

impl RuntimePluginBridgeLifecycleError {
    pub fn diagnostic(&self) -> String {
        match self {
            Self::StrongDependentsBlocked(block) => block.diagnostic(),
        }
    }
}

impl RuntimePluginCatalog {
    pub fn activate_bridge_provider_at_frame_boundary(
        &self,
        registry: &RuntimeExtensionRegistry,
        bridge_table: &FrozenBridgeTable,
        provider_package_id: &str,
    ) -> RuntimePluginBridgeLifecycleReport {
        self.transition_bridge_provider_at_frame_boundary(
            registry,
            bridge_table,
            provider_package_id,
            BridgeOwnerTransitionMode::Activate,
        )
    }

    pub fn disable_bridge_provider_at_frame_boundary(
        &self,
        registry: &RuntimeExtensionRegistry,
        bridge_table: &FrozenBridgeTable,
        provider_package_id: &str,
    ) -> Result<RuntimePluginBridgeLifecycleReport, RuntimePluginBridgeLifecycleError> {
        self.reject_strong_dependents(provider_package_id, BridgeOwnerTransitionMode::Disable)?;
        Ok(self.transition_bridge_provider_at_frame_boundary(
            registry,
            bridge_table,
            provider_package_id,
            BridgeOwnerTransitionMode::Disable,
        ))
    }

    pub fn deactivate_bridge_provider_at_frame_boundary(
        &self,
        registry: &RuntimeExtensionRegistry,
        bridge_table: &FrozenBridgeTable,
        provider_package_id: &str,
    ) -> Result<RuntimePluginBridgeLifecycleReport, RuntimePluginBridgeLifecycleError> {
        self.reject_strong_dependents(provider_package_id, BridgeOwnerTransitionMode::Deactivate)?;
        Ok(self.transition_bridge_provider_at_frame_boundary(
            registry,
            bridge_table,
            provider_package_id,
            BridgeOwnerTransitionMode::Deactivate,
        ))
    }

    pub fn reload_bridge_provider_at_frame_boundary(
        &self,
        registry: &RuntimeExtensionRegistry,
        replacement_registry: &RuntimeExtensionRegistry,
        bridge_table: &FrozenBridgeTable,
        provider_package_id: &str,
    ) -> RuntimePluginBridgeLifecycleReport {
        let runtime_modules = self.runtime_module_names_for_provider(provider_package_id);
        let mut owner_reports = Vec::new();

        for runtime_module in runtime_modules {
            let replacement_exports = replacement_registry
                .interface_owners_for_runtime_modules([runtime_module.as_str()])
                .into_iter()
                .flat_map(|replacement_owner| {
                    replacement_registry.interface_exports_owned_by(replacement_owner)
                })
                .collect::<Vec<_>>();

            owner_reports.extend(
                registry
                    .interface_owners_for_runtime_modules([runtime_module.as_str()])
                    .into_iter()
                    .map(|owner| {
                        bridge_table
                            .reload_owner_exports_with_report(owner, replacement_exports.clone())
                    }),
            );
        }

        RuntimePluginBridgeLifecycleReport {
            provider_package_id: provider_package_id.to_string(),
            mode: BridgeOwnerTransitionMode::Reload,
            owner_reports,
        }
    }

    fn reject_strong_dependents(
        &self,
        provider_package_id: &str,
        mode: BridgeOwnerTransitionMode,
    ) -> Result<(), RuntimePluginBridgeLifecycleError> {
        let blockers = self.strong_bridge_disable_blockers(provider_package_id);
        if blockers.is_empty() {
            return Ok(());
        }

        Err(RuntimePluginBridgeLifecycleError::StrongDependentsBlocked(
            RuntimePluginBridgeLifecycleBlock {
                provider_package_id: provider_package_id.to_string(),
                mode,
                blockers,
            },
        ))
    }

    fn transition_bridge_provider_at_frame_boundary(
        &self,
        registry: &RuntimeExtensionRegistry,
        bridge_table: &FrozenBridgeTable,
        provider_package_id: &str,
        mode: BridgeOwnerTransitionMode,
    ) -> RuntimePluginBridgeLifecycleReport {
        let runtime_modules = self.runtime_module_names_for_provider(provider_package_id);
        let owner_reports = registry
            .interface_owners_for_runtime_modules(runtime_modules.iter().map(String::as_str))
            .into_iter()
            .map(|owner| match mode {
                BridgeOwnerTransitionMode::Activate => bridge_table
                    .restore_owner_exports_with_report(
                        owner,
                        registry.interface_exports_owned_by(owner),
                    ),
                BridgeOwnerTransitionMode::Disable => {
                    bridge_table.set_owner_enabled_with_report(owner, false)
                }
                BridgeOwnerTransitionMode::Deactivate => {
                    bridge_table.deactivate_owner_with_report(owner)
                }
                BridgeOwnerTransitionMode::Reload => bridge_table.reload_owner_exports_with_report(
                    owner,
                    registry.interface_exports_owned_by(owner),
                ),
            })
            .collect();

        RuntimePluginBridgeLifecycleReport {
            provider_package_id: provider_package_id.to_string(),
            mode,
            owner_reports,
        }
    }

    fn runtime_module_names_for_provider(&self, provider_package_id: &str) -> Vec<String> {
        let mut runtime_modules = self
            .registrations()
            .iter()
            .find(|registration| registration.package_manifest.id == provider_package_id)
            .map(|registration| {
                registration
                    .package_manifest
                    .modules
                    .iter()
                    .filter(|module| module.kind == PluginModuleKind::Runtime)
                    .map(|module| module.name.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if runtime_modules.is_empty() {
            runtime_modules.push(format!("{provider_package_id}.runtime"));
        }
        runtime_modules.sort();
        runtime_modules.dedup();
        runtime_modules
    }

    pub fn provider_package_id_for_runtime_module(
        &self,
        runtime_module_name: &str,
    ) -> Option<String> {
        self.registrations()
            .iter()
            .find(|registration| {
                let runtime_modules =
                    self.runtime_module_names_for_provider(&registration.package_manifest.id);
                runtime_modules
                    .iter()
                    .any(|module_name| module_name == runtime_module_name)
            })
            .map(|registration| registration.package_manifest.id.clone())
    }
}
