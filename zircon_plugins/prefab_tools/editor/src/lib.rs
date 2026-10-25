mod authoring;
#[path = "capability.rs"]
mod capability;
#[path = "extension_ids.rs"]
mod extension_ids;
#[path = "plugin.rs"]
mod plugin;

pub use authoring::{
    effective_prefab_overrides, prefab_source_snapshot, prefab_source_snapshot_expected,
    validate_prefab_instance, PrefabAuthoringDiagnostic, PrefabAuthoringDiagnosticKind,
    PrefabInstanceAuthoringSnapshot, PrefabOverrideRow, PrefabSourceAuthoringSnapshot,
    PrefabSourceComponentConflict, PrefabSourceComponentDescriptor, PrefabSourceComponentLookup,
    PrefabSourceDiagnostic, PrefabSourceDiagnosticKind, PrefabSourceEntityLookup,
    PrefabSourceEntityParent, PrefabSourceEntityRow, PrefabSourceHierarchyRow,
    PrefabSourceReadSnapshot, PrefabSourceSnapshotError, PrefabSourceState,
};
pub use capability::{CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID};
pub use extension_ids::{PREFAB_AUTHORING_VIEW_ID, PREFAB_DRAWER_ID, PREFAB_TEMPLATE_ID};
pub use plugin::{
    editor_capabilities, editor_plugin, editor_plugin_descriptor, package_manifest,
    plugin_registration, PrefabToolsEditorPlugin,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/authoring_snapshot.rs"]
mod authoring_snapshot_tests;
