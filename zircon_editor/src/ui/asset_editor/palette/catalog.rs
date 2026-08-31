use std::{collections::BTreeMap, sync::Arc};

use zircon_runtime::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::{component::UiHostCapabilitySet, template::UiAssetDocument};

use super::{UiAssetPaletteEntry, UiAssetPaletteEntryKind};

/// Immutable palette entries published for one document/import generation.
#[derive(Clone, Debug)]
pub(crate) struct UiAssetPaletteCatalog {
    entries: Arc<[UiAssetPaletteEntry]>,
    reference_imports: Arc<BTreeMap<String, UiAssetDocument>>,
}

impl UiAssetPaletteCatalog {
    pub(crate) fn build(
        document: &UiAssetDocument,
        widget_imports: &BTreeMap<String, UiAssetDocument>,
    ) -> Self {
        let reference_imports = canonical_reference_imports(widget_imports);
        let registry = UiComponentDescriptorRegistry::editor_showcase_shared();
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
        for reference in reference_imports.keys() {
            entries.push(UiAssetPaletteEntry {
                label: reference_palette_label(reference),
                kind: UiAssetPaletteEntryKind::Reference {
                    component_ref: reference.clone(),
                },
            });
        }
        Self {
            entries: entries.into(),
            reference_imports: Arc::new(reference_imports),
        }
    }

    pub(crate) fn entries(&self) -> &[UiAssetPaletteEntry] {
        self.entries.as_ref()
    }

    pub(crate) fn entry(&self, index: usize) -> Option<&UiAssetPaletteEntry> {
        self.entries.get(index)
    }

    pub(crate) fn reference_imports(&self) -> &BTreeMap<String, UiAssetDocument> {
        self.reference_imports.as_ref()
    }
}

fn canonical_reference_imports(
    widget_imports: &BTreeMap<String, UiAssetDocument>,
) -> BTreeMap<String, UiAssetDocument> {
    let mut references = BTreeMap::new();
    for (reference, document) in widget_imports {
        if let Some(component_name) = reference_component_name(reference) {
            if document.components.contains_key(component_name) {
                let _ = references.insert(reference.clone(), document.clone());
            }
            continue;
        }
        for component_name in document.components.keys() {
            let _ = references.insert(format!("{reference}#{component_name}"), document.clone());
        }
    }
    references
}

fn reference_palette_label(reference: &str) -> String {
    let label = reference
        .split_once('#')
        .map_or(reference, |(_, component)| component);
    format!("Reference / {label}")
}

#[cfg(test)]
#[path = "catalog/reference_label_tests.rs"]
mod reference_label_tests;

fn reference_component_name(reference: &str) -> Option<&str> {
    reference
        .rsplit_once('#')
        .map(|(_, component_name)| component_name)
        .filter(|component_name| !component_name.is_empty())
}
