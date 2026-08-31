use std::collections::BTreeSet;

use crate::scene::ecs::SceneSystemThreadAffinity;

use super::{
    NativeSystemAccessAuthorityError, NativeSystemAccessDeclaration, NativeSystemAccessDomain,
    NativeSystemAccessPlan, NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY,
};

pub(in crate::plugin::native_plugin_loader) struct NativeSystemAccessAuthority {
    plugin_id: String,
    known_component_ids: BTreeSet<String>,
    known_resource_ids: BTreeSet<String>,
    granted_capabilities: BTreeSet<String>,
}

impl NativeSystemAccessAuthority {
    pub(in crate::plugin::native_plugin_loader) fn new(
        plugin_id: impl Into<String>,
        known_component_ids: impl IntoIterator<Item = String>,
        known_resource_ids: impl IntoIterator<Item = String>,
        granted_capabilities: impl IntoIterator<Item = String>,
    ) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            known_component_ids: known_component_ids.into_iter().collect(),
            known_resource_ids: known_resource_ids.into_iter().collect(),
            granted_capabilities: granted_capabilities.into_iter().collect(),
        }
    }

    pub(in crate::plugin::native_plugin_loader) fn authorize(
        &self,
        plan: &NativeSystemAccessPlan,
    ) -> Result<(), NativeSystemAccessAuthorityError> {
        if plan.affinity == SceneSystemThreadAffinity::WorkerSafe
            && !self
                .granted_capabilities
                .contains(NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY)
        {
            return Err(NativeSystemAccessAuthorityError::WorkerSafeCapabilityNotGranted);
        }
        for declaration in &plan.declarations {
            let known = match declaration.domain {
                NativeSystemAccessDomain::Component => {
                    self.known_component_ids.contains(&declaration.stable_id)
                }
                NativeSystemAccessDomain::Resource => {
                    self.known_resource_ids.contains(&declaration.stable_id)
                }
            };
            if !known {
                return Err(NativeSystemAccessAuthorityError::UnknownStableId {
                    domain: declaration.domain,
                    stable_id: declaration.stable_id.clone(),
                });
            }
            if self.owns(&declaration.stable_id) {
                continue;
            }
            let required_capability = declaration.required_capability();
            if !self.granted_capabilities.contains(&required_capability) {
                return Err(NativeSystemAccessAuthorityError::CapabilityNotGranted {
                    stable_id: declaration.stable_id.clone(),
                    required_capability,
                });
            }
        }
        Ok(())
    }

    fn owns(&self, stable_id: &str) -> bool {
        stable_id == self.plugin_id
            || stable_id
                .strip_prefix(&self.plugin_id)
                .is_some_and(|suffix| suffix.starts_with('.'))
    }
}

impl NativeSystemAccessDeclaration {
    fn required_capability(&self) -> String {
        format!(
            "runtime.native.ecs.{}.{}.{}",
            self.domain.label(),
            self.mode.label(),
            self.stable_id
        )
    }
}
