use std::fmt::Write as _;

use crate::core::framework::bridge::{BridgeOwnerTransitionMode, InterfaceSlot};
use crate::plugin::{BridgeOwnerTransitionReport, FrozenBridgeTable, RuntimeExtensionRegistry};

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
        let mut affected_slots = Vec::with_capacity(self.affected_slot_count());
        for report in &self.owner_reports {
            affected_slots.extend_from_slice(&report.affected_slots);
        }
        affected_slots
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
        let mut diagnostic = String::with_capacity(self.diagnostic_capacity());
        write!(
            diagnostic,
            "bridge.provider_lifecycle_blocked: provider plugin `{}` {:?} blocked by {} strong dependent(s): ",
            self.provider_package_id,
            self.mode,
            self.blockers.len(),
        )
        .expect("writing bridge lifecycle diagnostic to String cannot fail");
        for (index, blocker) in self.blockers.iter().enumerate() {
            if index != 0 {
                diagnostic.push_str("; ");
            }
            blocker.write_diagnostic(&mut diagnostic);
        }
        diagnostic
    }

    fn diagnostic_capacity(&self) -> usize {
        const PREFIX: &str = "bridge.provider_lifecycle_blocked: provider plugin `";
        const MODE_PREFIX: &str = "` ";
        const COUNT_PREFIX: &str = " blocked by ";
        const BLOCKER_PREFIX: &str = " strong dependent(s): ";
        const BLOCKER_SEPARATOR: &str = "; ";

        PREFIX
            .len()
            .saturating_add(self.provider_package_id.len())
            .saturating_add(MODE_PREFIX.len())
            .saturating_add(bridge_transition_mode_debug_len(self.mode))
            .saturating_add(COUNT_PREFIX.len())
            .saturating_add(decimal_len(self.blockers.len()))
            .saturating_add(BLOCKER_PREFIX.len())
            .saturating_add(
                self.blockers
                    .iter()
                    .map(RuntimePluginBridgeDisableBlocker::diagnostic_len)
                    .sum::<usize>(),
            )
            .saturating_add(
                self.blockers
                    .len()
                    .saturating_sub(1)
                    .saturating_mul(BLOCKER_SEPARATOR.len()),
            )
    }
}

fn bridge_transition_mode_debug_len(mode: BridgeOwnerTransitionMode) -> usize {
    match mode {
        BridgeOwnerTransitionMode::Activate => "Activate".len(),
        BridgeOwnerTransitionMode::Disable => "Disable".len(),
        BridgeOwnerTransitionMode::Deactivate => "Deactivate".len(),
        BridgeOwnerTransitionMode::Reload => "Reload".len(),
    }
}

fn decimal_len(mut value: usize) -> usize {
    let mut digits = 1;
    while value >= 10 {
        value /= 10;
        digits += 1;
    }
    digits
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
        let runtime_modules = self
            .projection
            .runtime_modules_for_provider(provider_package_id);
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
                        bridge_table.reload_owner_exports_with_report(
                            owner,
                            replacement_exports.iter().copied(),
                        )
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
        let runtime_modules = self
            .projection
            .runtime_modules_for_provider(provider_package_id);
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

    pub fn provider_package_id_for_runtime_module(
        &self,
        runtime_module_name: &str,
    ) -> Option<String> {
        self.projection
            .provider_for_runtime_module(runtime_module_name)
            .map(ToOwned::to_owned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streaming_bridge_lifecycle_block_diagnostic_preserves_contract() {
        let block = RuntimePluginBridgeLifecycleBlock {
            provider_package_id: "rendering".to_string(),
            mode: BridgeOwnerTransitionMode::Disable,
            blockers: vec![
                RuntimePluginBridgeDisableBlocker {
                    provider_package_id: "rendering".to_string(),
                    dependent_package_id: "editor_a".to_string(),
                    interface_ids: vec!["render.api".to_string()],
                },
                RuntimePluginBridgeDisableBlocker {
                    provider_package_id: "rendering".to_string(),
                    dependent_package_id: "editor_b".to_string(),
                    interface_ids: vec!["render.debug".to_string()],
                },
            ],
        };

        assert_eq!(
            block.diagnostic(),
            "bridge.provider_lifecycle_blocked: provider plugin `rendering` Disable blocked by 2 strong dependent(s): bridge.strong_target_disable_blocked: provider plugin `rendering` cannot be disabled while dependent plugin `editor_a` requires interfaces [`render.api`]; bridge.strong_target_disable_blocked: provider plugin `rendering` cannot be disabled while dependent plugin `editor_b` requires interfaces [`render.debug`]"
        );
    }
}
