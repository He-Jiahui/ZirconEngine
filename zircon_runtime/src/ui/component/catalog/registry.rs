use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiComponentDescriptor, UiHostCapability, UiHostCapabilitySet,
};

use super::super::descriptor::{UiComponentDescriptorError, validate_component_descriptor};

use super::palette_view::UiComponentPaletteEntry;

const COMPONENT_CATEGORIES: [UiComponentCategory; 8] = [
    UiComponentCategory::Visual,
    UiComponentCategory::Input,
    UiComponentCategory::Numeric,
    UiComponentCategory::Selection,
    UiComponentCategory::Reference,
    UiComponentCategory::Collection,
    UiComponentCategory::Container,
    UiComponentCategory::Feedback,
];

type UiComponentCategoryIter =
    std::iter::Flatten<std::array::IntoIter<Option<UiComponentCategory>, 8>>;

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
    pub fn categories(&self) -> UiComponentCategoryIter {
        unique_component_categories(
            self.descriptors
                .values()
                .map(|descriptor| descriptor.category),
        )
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

fn unique_component_categories(
    categories: impl IntoIterator<Item = UiComponentCategory>,
) -> UiComponentCategoryIter {
    let mut ordered = [None; COMPONENT_CATEGORIES.len()];
    for category in categories {
        ordered[component_category_index(category)] = Some(category);
    }
    ordered.into_iter().flatten()
}

const fn component_category_index(category: UiComponentCategory) -> usize {
    match category {
        UiComponentCategory::Visual => 0,
        UiComponentCategory::Input => 1,
        UiComponentCategory::Numeric => 2,
        UiComponentCategory::Selection => 3,
        UiComponentCategory::Reference => 4,
        UiComponentCategory::Collection => 5,
        UiComponentCategory::Container => 6,
        UiComponentCategory::Feedback => 7,
    }
}

#[cfg(test)]
mod tests {
    use super::{UiComponentCategory, unique_component_categories};

    #[test]
    fn allocation_free_categories_preserve_enum_order() {
        let categories = unique_component_categories([
            UiComponentCategory::Feedback,
            UiComponentCategory::Numeric,
            UiComponentCategory::Visual,
        ])
        .collect::<Vec<_>>();

        assert_eq!(
            categories,
            [
                UiComponentCategory::Visual,
                UiComponentCategory::Numeric,
                UiComponentCategory::Feedback,
            ]
        );
    }

    #[test]
    fn allocation_free_categories_deduplicate_repeated_values() {
        let categories = unique_component_categories([
            UiComponentCategory::Input,
            UiComponentCategory::Input,
            UiComponentCategory::Selection,
            UiComponentCategory::Input,
        ])
        .collect::<Vec<_>>();

        assert_eq!(
            categories,
            [UiComponentCategory::Input, UiComponentCategory::Selection]
        );
    }

    #[test]
    fn allocation_free_categories_handle_empty_input() {
        assert_eq!(unique_component_categories([]).next(), None);
    }
}
