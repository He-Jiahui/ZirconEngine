//! Project-manifest selection policy for manager-owned editor packages.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ProjectPluginManifest;

use super::{
    apply_enablement_request, has_failed_disabled_lifecycle, validate_enablement_request,
    EditorPluginManager, EditorPluginManagerEntry, EditorPluginManagerSnapshot,
    EditorPluginTransitionError,
};

/// Applies all editor-package selections after checking every state transition without side
/// effects. Lifecycle callbacks therefore only begin once the complete manifest is admissible.
pub(super) fn apply_project_manifest(
    manager: &EditorPluginManager,
    manifest: &ProjectPluginManifest,
) -> Result<Arc<EditorPluginManagerSnapshot>, EditorPluginTransitionError> {
    let _mutation = manager
        .lifecycle_mutation
        .try_lock()
        .map_err(|_| EditorPluginTransitionError::MutationInProgress)?;
    let previous = manager.state_snapshot();
    let enablement = editor_package_enablement(manifest, previous.entries())?;
    for entry in previous.entries() {
        let enabled = enablement
            .get(entry.package_id.as_str())
            .copied()
            .unwrap_or(false);
        validate_enablement_request(
            entry,
            enabled,
            previous.reached_loading_phase,
            has_failed_disabled_lifecycle(previous.catalog_snapshot(), entry),
        )?;
    }

    let mut entries = previous.entries().to_vec();
    let mut catalog = previous.catalog_snapshot().clone_catalog();
    let mut entries_changed = false;
    let mut catalog_changed = false;
    for (index, entry) in entries.iter_mut().enumerate() {
        let previous_entry = &previous.entries()[index];
        let previous_state = entry.state;
        let enabled = enablement
            .get(entry.package_id.as_str())
            .copied()
            .unwrap_or(false);
        let changed = apply_enablement_request(
            &mut catalog,
            entry,
            enabled,
            previous.reached_loading_phase,
            has_failed_disabled_lifecycle(previous.catalog_snapshot(), previous_entry),
        )?;
        entries_changed |= entry.state != previous_state;
        catalog_changed |= changed;
    }

    if !entries_changed && !catalog_changed {
        return Ok(previous);
    }
    Ok(manager.publish_manager_snapshot(
        &previous,
        catalog_changed.then_some(catalog),
        entries,
        previous.reached_loading_phase,
    ))
}

/// Returns the desired enablement state for catalog packages covered by this editor host.
///
/// Project manifests also select runtime-only packages. Those rows are deliberately ignored
/// here; this manager owns only the catalog packages it has already discovered.
pub(super) fn editor_package_enablement(
    manifest: &ProjectPluginManifest,
    entries: &[EditorPluginManagerEntry],
) -> Result<BTreeMap<String, bool>, EditorPluginTransitionError> {
    let editor_package_ids = entries
        .iter()
        .map(|entry| entry.package_id().to_string())
        .collect::<BTreeSet<_>>();
    let mut enablement = BTreeMap::new();

    for selection in &manifest.selections {
        if !editor_package_ids.contains(selection.id.as_str()) {
            continue;
        }
        let enabled = selection.enabled && selection.supports_target(RuntimeTargetMode::EditorHost);
        if enablement.insert(selection.id.clone(), enabled).is_some() {
            return Err(EditorPluginTransitionError::DuplicateProjectSelection {
                package_id: selection.id.clone(),
            });
        }
    }

    Ok(enablement)
}
