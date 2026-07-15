use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use crate::core::framework::script::{
    ScriptBehaviorBridge, ScriptBehaviorCallbackRef, ScriptHostError, ScriptHostValue,
};

use super::{VmCallbackHandle, VmPluginManager};

/// Script-owned implementation exported by the ZrVM runtime plugin through
/// `script.behavior.v1`.
#[derive(Default)]
pub struct VmScriptBehaviorBridge {
    manager: Mutex<Weak<VmPluginManager>>,
    callbacks: Mutex<BTreeMap<ScriptBehaviorCallbackRef, VmCallbackHandle>>,
}

impl VmScriptBehaviorBridge {
    pub fn new() -> Self {
        Self::default()
    }

    /// Binds the active script manager without extending its lifecycle.
    pub fn bind_manager(&self, manager: &Arc<VmPluginManager>) {
        let next = Arc::downgrade(manager);
        let changed = {
            let mut current = self.lock_manager();
            let changed = !current.ptr_eq(&next);
            *current = next;
            changed
        };
        if changed {
            self.lock_callbacks().clear();
        }
    }

    fn manager(&self) -> Result<Arc<VmPluginManager>, ScriptHostError> {
        self.lock_manager().upgrade().ok_or_else(|| {
            ScriptHostError::new("script behavior bridge is not bound to an active VM manager")
        })
    }

    fn resolve_callback(
        &self,
        manager: &VmPluginManager,
        callback: &ScriptBehaviorCallbackRef,
    ) -> Result<VmCallbackHandle, ScriptHostError> {
        let slot = manager
            .slot_for_package_name(callback.package_id())
            .map_err(|error| ScriptHostError::new(error.to_string()))?;
        let generation = manager
            .slot(slot)
            .map_err(|error| ScriptHostError::new(error.to_string()))?
            .generation;
        if let Some(cached) = self.lock_callbacks().get(callback).copied() {
            if cached.slot == slot && cached.generation == generation {
                return Ok(cached);
            }
        }

        let mut matches = manager
            .registered_behavior_nodes()
            .into_iter()
            .filter(|registration| {
                registration.callback.slot == slot && registration.id == callback.node_id()
            });
        let Some(registration) = matches.next() else {
            return Err(ScriptHostError::new(format!(
                "script behavior callback `{}` is not registered",
                callback.stable_id()
            )));
        };
        if matches.next().is_some() {
            return Err(ScriptHostError::new(format!(
                "script behavior callback `{}` is ambiguous",
                callback.stable_id()
            )));
        }
        self.lock_callbacks()
            .insert(callback.clone(), registration.callback);
        Ok(registration.callback)
    }

    fn lock_manager(&self) -> MutexGuard<'_, Weak<VmPluginManager>> {
        self.manager
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_callbacks(
        &self,
    ) -> MutexGuard<'_, BTreeMap<ScriptBehaviorCallbackRef, VmCallbackHandle>> {
        self.callbacks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl ScriptBehaviorBridge for VmScriptBehaviorBridge {
    fn invoke(
        &self,
        callback: &ScriptBehaviorCallbackRef,
        arguments: &[ScriptHostValue],
    ) -> Result<Option<ScriptHostValue>, ScriptHostError> {
        let manager = self.manager()?;
        let mut handle = self.resolve_callback(&manager, callback)?;
        let result = manager
            .invoke_callback(&mut handle, arguments)
            .map_err(|error| ScriptHostError::new(error.to_string()));
        if result.is_ok() {
            self.lock_callbacks().insert(callback.clone(), handle);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::{
        CapabilitySet, VmInterfaceCaller, VmPluginManagementPolicy, VmPluginManifest,
        VmPluginPackage, VM_BT_NODE_CAPABILITY,
    };

    #[test]
    fn provider_qualified_callback_selects_slot_and_refreshes_generation() {
        let manager = VmPluginManager::mock();
        let first_package = package("first");
        let second_package = package("second");
        let first_slot = manager.load_package(first_package.clone()).unwrap();
        let second_slot = manager.load_package(second_package).unwrap();
        register_same_node(&manager, first_slot, 1);
        register_same_node(&manager, second_slot, 1);
        let bridge = VmScriptBehaviorBridge::new();
        bridge.bind_manager(&manager);

        let first_ref = ScriptBehaviorCallbackRef::parse("first::shared.task").unwrap();
        let second_ref = ScriptBehaviorCallbackRef::parse("second::shared.task").unwrap();
        let first = bridge.resolve_callback(&manager, &first_ref).unwrap();
        let second = bridge.resolve_callback(&manager, &second_ref).unwrap();
        assert_eq!(first.slot, first_slot);
        assert_eq!(second.slot, second_slot);

        manager.hot_reload_slot(first_slot, first_package).unwrap();
        register_same_node(&manager, first_slot, 2);
        let reloaded = bridge.resolve_callback(&manager, &first_ref).unwrap();
        assert_eq!(reloaded.slot, first_slot);
        assert_eq!(reloaded.generation, 2);
    }

    #[test]
    fn duplicate_active_package_names_are_rejected_as_ambiguous() {
        let manager = VmPluginManager::mock();
        manager.load_package(package("duplicate")).unwrap();
        manager.load_package(package("duplicate")).unwrap();
        let bridge = VmScriptBehaviorBridge::new();
        bridge.bind_manager(&manager);

        let error = bridge
            .resolve_callback(
                &manager,
                &ScriptBehaviorCallbackRef::parse("duplicate::shared.task").unwrap(),
            )
            .unwrap_err();
        assert!(error.message.contains("ambiguous"), "{}", error.message);
    }

    fn register_same_node(
        manager: &VmPluginManager,
        slot: super::super::PluginSlotId,
        generation: u32,
    ) {
        let caller = VmInterfaceCaller::new(
            slot,
            generation,
            CapabilitySet::default().with(VM_BT_NODE_CAPABILITY),
        );
        manager
            .host_interfaces()
            .register_behavior_node(&caller, "shared.task", "Shared", "ai", "tick")
            .unwrap();
    }

    fn package(name: &str) -> VmPluginPackage {
        VmPluginPackage {
            manifest: VmPluginManifest {
                name: name.to_string(),
                version: "1.0.0".to_string(),
                entry: "main".to_string(),
                capabilities: CapabilitySet::default().with(VM_BT_NODE_CAPABILITY),
                management: VmPluginManagementPolicy::default(),
            },
            zr_vm_project: None,
            bytecode: vec![1, 2, 3],
        }
    }
}
