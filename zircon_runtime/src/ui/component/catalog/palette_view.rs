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
    let mut entries = registry
        .descriptors()
        .filter(|descriptor| host_capabilities.contains_all(&descriptor.required_host_capabilities))
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
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        left.category
            .cmp(&right.category)
            .then_with(|| left.sort_key.cmp(&right.sort_key))
            .then_with(|| left.display_name.cmp(&right.display_name))
            .then_with(|| left.component_id.cmp(&right.component_id))
    });
    entries
}
