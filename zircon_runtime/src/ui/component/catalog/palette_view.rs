use serde::{Deserialize, Serialize};

use super::registry::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiDefaultNodeTemplate, UiHostCapabilitySet,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiComponentPaletteEntry {
    pub component_id: String,
    pub display_name: String,
    pub category: UiComponentCategory,
    #[serde(default)]
    pub icon: Option<String>,
    pub sort_key: String,
    pub default_node: UiDefaultNodeTemplate,
}

pub(super) fn palette_entries_for_host(
    registry: &UiComponentDescriptorRegistry,
    host_capabilities: &UiHostCapabilitySet,
) -> Vec<UiComponentPaletteEntry> {
    let mut entries = Vec::with_capacity(registry.len());
    entries.extend(
        registry
            .descriptors()
            .filter(|descriptor| {
                host_capabilities.contains_all(&descriptor.required_host_capabilities)
            })
            .filter_map(|descriptor| {
                let metadata = descriptor.palette.as_ref()?;
                Some(UiComponentPaletteEntry {
                    component_id: descriptor.id.clone(),
                    display_name: metadata.display_name.clone(),
                    category: metadata.category,
                    icon: metadata.icon.clone(),
                    sort_key: metadata.sort_key.clone(),
                    default_node: metadata.default_node.clone(),
                })
            }),
    );
    entries.sort_by(|left, right| {
        left.category
            .cmp(&right.category)
            .then_with(|| left.sort_key.cmp(&right.sort_key))
            .then_with(|| left.display_name.cmp(&right.display_name))
            .then_with(|| left.component_id.cmp(&right.component_id))
    });
    entries
}

#[cfg(test)]
mod tests {
    use super::{UiComponentDescriptorRegistry, UiHostCapabilitySet, palette_entries_for_host};

    #[test]
    fn preallocated_palette_projection_reserves_registry_upper_bound() {
        let registry = UiComponentDescriptorRegistry::editor_showcase_shared();
        let entries = palette_entries_for_host(registry, &UiHostCapabilitySet::editor_authoring());

        assert!(!entries.is_empty());
        assert!(entries.len() <= registry.len());
        assert_eq!(entries.capacity(), registry.len());
    }

    #[test]
    fn preallocated_palette_projection_preserves_sort_order() {
        let registry = UiComponentDescriptorRegistry::editor_showcase_shared();
        let entries = palette_entries_for_host(registry, &UiHostCapabilitySet::editor_authoring());

        assert!(entries.windows(2).all(|window| {
            (
                window[0].category,
                &window[0].sort_key,
                &window[0].display_name,
                &window[0].component_id,
            ) <= (
                window[1].category,
                &window[1].sort_key,
                &window[1].display_name,
                &window[1].component_id,
            )
        }));
    }
}
