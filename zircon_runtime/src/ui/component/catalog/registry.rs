use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiComponentDescriptor, UiHostCapability, UiHostCapabilitySet,
};

use super::super::descriptor::{validate_component_descriptor, UiComponentDescriptorError};

use super::palette_view::UiComponentPaletteEntry;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiComponentDescriptorRegistry {
    descriptors: BTreeMap<String, UiComponentDescriptor>,
    revision: u64,
}

impl UiComponentDescriptorRegistry {
    /// Creates an empty component descriptor registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers or replaces a descriptor by component id.
    pub fn register(
        &mut self,
        descriptor: UiComponentDescriptor,
    ) -> Result<bool, UiComponentDescriptorError> {
        self.try_register(descriptor)
    }

    pub fn try_register(
        &mut self,
        descriptor: UiComponentDescriptor,
    ) -> Result<bool, UiComponentDescriptorError> {
        validate_component_descriptor(&descriptor)?;
        if self.descriptors.get(&descriptor.id) == Some(&descriptor) {
            return Ok(false);
        }
        self.descriptors.insert(descriptor.id.clone(), descriptor);
        self.revision = self.revision.saturating_add(1);
        Ok(true)
    }

    /// Returns the monotonic registry revision for descriptor-set changes.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns the descriptor for a component id.
    pub fn descriptor(&self, component_id: &str) -> Option<&UiComponentDescriptor> {
        self.descriptors.get(component_id)
    }

    /// Returns whether the registry has a descriptor for a component id.
    pub fn contains(&self, component_id: &str) -> bool {
        self.descriptors.contains_key(component_id)
    }

    /// Returns the number of registered component descriptors.
    pub fn len(&self) -> usize {
        self.descriptors.len()
    }

    /// Returns whether the registry has no registered component descriptors.
    pub fn is_empty(&self) -> bool {
        self.descriptors.is_empty()
    }

    /// Iterates registered component ids in deterministic order.
    pub fn component_ids(&self) -> impl Iterator<Item = &str> {
        self.descriptors.keys().map(String::as_str)
    }

    /// Iterates component categories represented by the registry.
    pub fn categories(&self) -> impl Iterator<Item = UiComponentCategory> {
        self.descriptors
            .values()
            .map(|descriptor| descriptor.category)
            .collect::<BTreeSet<_>>()
            .into_iter()
    }

    /// Iterates all registered descriptors in deterministic component-id order.
    pub fn descriptors(&self) -> impl Iterator<Item = &UiComponentDescriptor> {
        self.descriptors.values()
    }

    /// Iterates registered descriptors that belong to a component category.
    pub fn descriptors_in_category(
        &self,
        category: UiComponentCategory,
    ) -> impl Iterator<Item = &UiComponentDescriptor> {
        self.descriptors
            .values()
            .filter(move |descriptor| descriptor.category == category)
    }

    pub fn descriptors_for_host(
        &self,
        host_capabilities: &UiHostCapabilitySet,
    ) -> Vec<&UiComponentDescriptor> {
        self.descriptors
            .values()
            .filter(|descriptor| {
                host_capabilities.contains_all(&descriptor.required_host_capabilities)
            })
            .collect()
    }

    pub fn palette_entries_for_host(
        &self,
        host_capabilities: &UiHostCapabilitySet,
    ) -> Vec<UiComponentPaletteEntry> {
        super::palette_view::palette_entries_for_host(self, host_capabilities)
    }

    pub fn missing_capabilities(
        &self,
        component_id: &str,
        host_capabilities: &UiHostCapabilitySet,
    ) -> Option<BTreeSet<UiHostCapability>> {
        self.descriptor(component_id)
            .map(|descriptor| host_capabilities.missing(&descriptor.required_host_capabilities))
    }
}
