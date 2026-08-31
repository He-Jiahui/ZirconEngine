use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiComponentDescriptor, UiHostCapability, UiHostCapabilitySet,
};

use super::super::descriptor::{validate_component_descriptor, UiComponentDescriptorError};

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
        let mut descriptors = Vec::with_capacity(self.descriptors.len());
        descriptors.extend(self.descriptors.values().filter(|descriptor| {
            host_capabilities.contains_all(&descriptor.required_host_capabilities)
        }));
        descriptors
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
    use super::{unique_component_categories, UiComponentCategory};

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

    #[test]
    fn optimization_batch_20260830cy_host_descriptors_reserve_registry_bound() {
        let source = include_str!("registry.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("component registry production source");

        assert!(production.contains("Vec::with_capacity(self.descriptors.len())"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830cy_host_descriptor_capacity_evidence() {
        const BATCH_COUNT: usize = 32_768;
        const DESCRIPTOR_COUNT: usize = 32;
        const MATCHED_DESCRIPTOR_COUNT: usize = 24;
        const MARKER: &str = "RUNTIME511_HOST_DESCRIPTOR_CAPACITY_BENCH_V1";

        let legacy_growth_events = descriptor_growth_events(
            BATCH_COUNT,
            DESCRIPTOR_COUNT,
            MATCHED_DESCRIPTOR_COUNT,
            false,
        );
        let optimized_growth_events = descriptor_growth_events(
            BATCH_COUNT,
            DESCRIPTOR_COUNT,
            MATCHED_DESCRIPTOR_COUNT,
            true,
        );

        assert!(legacy_growth_events > 0);
        assert_eq!(optimized_growth_events, 0);
        println!(
            "{MARKER} batches={BATCH_COUNT} descriptor_count={DESCRIPTOR_COUNT} \
             matched_descriptor_count={MATCHED_DESCRIPTOR_COUNT} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
        );
    }

    fn descriptor_growth_events(
        batch_count: usize,
        descriptor_count: usize,
        matched_descriptor_count: usize,
        reserve: bool,
    ) -> usize {
        let mut growth_events = 0;
        for _ in 0..batch_count {
            let mut descriptors = if reserve {
                Vec::with_capacity(descriptor_count)
            } else {
                Vec::new()
            };
            for descriptor in 0..matched_descriptor_count {
                let previous_capacity = descriptors.capacity();
                descriptors.push(descriptor);
                growth_events += usize::from(descriptors.capacity() != previous_capacity);
            }
        }
        growth_events
    }
}
