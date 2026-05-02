use std::collections::BTreeMap;

use zircon_runtime::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::{component::UiHostCapabilitySet, template::UiAssetDocument};

use super::{UiAssetPaletteEntry, UiAssetPaletteEntryKind};

pub(crate) fn build_palette_entries(
    document: &UiAssetDocument,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
) -> Vec<UiAssetPaletteEntry> {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let mut entries = registry
        .palette_entries_for_host(&UiHostCapabilitySet::editor_authoring())
        .into_iter()
        .map(|entry| UiAssetPaletteEntry {
            label: format!("Native / {}", entry.display_name),
            kind: UiAssetPaletteEntryKind::Native {
                widget_type: entry.component_id,
                default_node: entry.default_node,
            },
        })
        .collect::<Vec<_>>();

    for component_name in document.components.keys() {
        entries.push(UiAssetPaletteEntry {
            label: format!("Component / {component_name}"),
            kind: UiAssetPaletteEntryKind::Component {
                component: component_name.clone(),
            },
        });
    }
    for reference in widget_imports.keys() {
        let label = reference
            .split_once('#')
            .map(|(_, component)| component.to_string())
            .unwrap_or_else(|| reference.clone());
        entries.push(UiAssetPaletteEntry {
            label: format!("Reference / {label}"),
            kind: UiAssetPaletteEntryKind::Reference {
                component_ref: reference.clone(),
            },
        });
    }
    entries
}
