use std::{collections::BTreeMap, sync::Arc};

use zircon_runtime::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::{component::UiHostCapabilitySet, template::UiAssetDocument};

use super::{UiAssetPaletteEntry, UiAssetPaletteEntryKind};

/// Immutable entries and the current session's bounded palette page.
#[derive(Clone, Debug)]
pub(crate) struct UiAssetPaletteCatalog {
    entries: Arc<[UiAssetPaletteEntry]>,
    reference_imports: Arc<BTreeMap<String, UiAssetDocument>>,
    builtin_references: Arc<BTreeMap<usize, String>>,
    visible_start: usize,
    builtin_descriptors: Arc<BTreeMap<usize, crate::ui::template::EditorComponentDescriptor>>,
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
        let append_capacity = document
            .components
            .len()
            .saturating_add(reference_imports.len());
        entries.reserve_exact(append_capacity);

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
            builtin_references: Arc::new(BTreeMap::new()),
            visible_start: 0,
            builtin_descriptors: Arc::new(BTreeMap::new()),
        }
    }

    /// Catalog metadata supplies choices; selection loads only the chosen source closure.
    pub(crate) fn with_builtin_primitives(
        mut self,
        descriptors: Vec<crate::ui::template::EditorComponentDescriptor>,
    ) -> Self {
        let mut entries = self.entries.to_vec();
        let mut builtin_references = BTreeMap::new();
        let mut builtin_descriptors = BTreeMap::new();
        for descriptor in descriptors {
            if descriptor.tier != crate::ui::template::EditorComponentTier::Primitive {
                continue;
            }
            let reference = format!("{}#{}", descriptor.document_id, descriptor.component_id);
            builtin_descriptors.insert(entries.len(), descriptor.clone());
            builtin_references.insert(entries.len(), reference.clone());
            entries.push(UiAssetPaletteEntry {
                label: format!("Builtin / {}", descriptor.component_id),
                kind: UiAssetPaletteEntryKind::Reference {
                    component_ref: reference.clone(),
                },
            });
        }
        self.entries = entries.into();
        self.builtin_references = Arc::new(builtin_references);
        self.builtin_descriptors = Arc::new(builtin_descriptors);
        self
    }

    pub(crate) fn builtin_reference(&self, index: usize) -> Option<&str> {
        self.builtin_references.get(&index).map(String::as_str)
    }

    pub(crate) fn builtin_descriptor(
        &self,
        index: usize,
    ) -> Option<&crate::ui::template::EditorComponentDescriptor> {
        self.builtin_descriptors.get(&index)
    }

    pub(crate) fn visible_start(&self) -> usize {
        self.visible_start
    }

    pub(crate) fn set_visible_start(&mut self, index: usize) -> bool {
        let last_page = self.entries.len().saturating_sub(1) / 4 * 4;
        let next = (index / 4 * 4).min(last_page);
        let changed = self.visible_start != next;
        self.visible_start = next;
        changed
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

#[cfg(test)]
#[path = "catalog/tests/perf.rs"]
mod optimization_batch_jg_editor646_tests;

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
#[path = "catalog/tests/reference_label_tests.rs"]
mod reference_label_tests;

fn reference_component_name(reference: &str) -> Option<&str> {
    reference
        .rsplit_once('#')
        .map(|(_, component_name)| component_name)
        .filter(|component_name| !component_name.is_empty())
}
