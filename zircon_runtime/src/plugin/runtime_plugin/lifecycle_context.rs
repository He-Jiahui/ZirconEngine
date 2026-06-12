use std::collections::{HashMap, HashSet};

use crate::core::framework::bridge::PluginInterface;
use crate::core::CoreHandle;
use crate::plugin::{CapabilityStatus, RuntimeExtensionRegistry};
use crate::plugin::{RuntimeExtensionRegistryError, StrongBridge, WeakBridge};
use crate::scene::World;

#[derive(Clone, Debug, Default)]
pub struct CapabilityView {
    provided: HashSet<String>,
    statuses: HashMap<String, CapabilityStatus>,
}

impl CapabilityView {
    pub fn from_capabilities(capabilities: impl IntoIterator<Item = String>) -> Self {
        Self {
            provided: capabilities.into_iter().collect(),
            statuses: HashMap::new(),
        }
    }

    pub fn has(&self, capability: &str) -> bool {
        self.provided.contains(capability)
    }

    pub fn status(&self, capability: &str) -> Option<CapabilityStatus> {
        self.statuses.get(capability).copied()
    }

    pub fn with_status(mut self, capability: impl Into<String>, status: CapabilityStatus) -> Self {
        let capability = capability.into();
        self.provided.insert(capability.clone());
        self.statuses.insert(capability, status);
        self
    }
}

pub struct PluginFinishContext<'a> {
    pub registry: &'a mut RuntimeExtensionRegistry,
    pub capabilities: &'a CapabilityView,
}

impl<'a> PluginFinishContext<'a> {
    pub fn new(
        registry: &'a mut RuntimeExtensionRegistry,
        capabilities: &'a CapabilityView,
    ) -> Self {
        Self {
            registry,
            capabilities,
        }
    }

    pub fn resolve_strong<T>(&self) -> Result<StrongBridge<T>, RuntimeExtensionRegistryError>
    where
        T: PluginInterface + ?Sized,
    {
        self.registry.frozen_bridge_table().resolve_strong::<T>()
    }

    pub fn resolve_weak<T>(&self) -> WeakBridge<T>
    where
        T: PluginInterface + ?Sized,
    {
        let bridge_table = self.registry.frozen_bridge_table();
        WeakBridge::owned(bridge_table)
    }
}

pub struct PluginRuntimeContext<'a> {
    pub world: &'a mut World,
    pub core: &'a CoreHandle,
}

impl<'a> PluginRuntimeContext<'a> {
    pub fn new(world: &'a mut World, core: &'a CoreHandle) -> Self {
        Self { world, core }
    }
}
