//! Atomic lifecycle cleanup and activation for plugin instance replacement.

use std::collections::BTreeSet;

use super::super::catalog::EditorPluginCatalog;
use super::super::sdk::lifecycle::{EditorPluginLifecycleEvent, EditorPluginLifecycleStage};
use super::discovery::EditorPluginDiscoveryError;
use super::snapshot::{EditorPluginManagerEntry, EditorPluginManagerSnapshot};
use super::state::EditorPluginState;

pub(super) fn reset_replaced_active_entries(
    previous_catalog: &EditorPluginCatalog,
    candidate_catalog: &EditorPluginCatalog,
    entries: &mut [EditorPluginManagerEntry],
) {
    for entry in entries.iter_mut().filter(|entry| {
        matches!(
            entry.state,
            EditorPluginState::Faulted | EditorPluginState::Active | EditorPluginState::Revoking
        )
    }) {
        if !candidate_catalog.is_package_faulted(entry.package_id())
            && !candidate_catalog.has_same_lifecycle_plugin(previous_catalog, entry.package_id())
        {
            entry.state = EditorPluginState::Validated;
        }
    }
}

pub(super) fn replaced_live_package_ids(
    previous: &EditorPluginManagerSnapshot,
    previous_catalog: &EditorPluginCatalog,
    candidate_catalog: &EditorPluginCatalog,
) -> BTreeSet<String> {
    previous
        .entries()
        .iter()
        .filter(|entry| instance_requires_retirement(previous_catalog, entry))
        .filter(|entry| {
            !candidate_catalog.has_same_lifecycle_plugin(previous_catalog, entry.package_id())
        })
        .map(|entry| entry.package_id.clone())
        .collect()
}

fn instance_requires_retirement(
    catalog: &EditorPluginCatalog,
    entry: &EditorPluginManagerEntry,
) -> bool {
    entry.state == EditorPluginState::Active
        || entry.state == EditorPluginState::Revoking
        || (entry.state == EditorPluginState::Faulted
            && catalog.lifecycle_stage_succeeded(
                entry.package_id(),
                &EditorPluginLifecycleStage::Enabled,
            ))
}

pub(super) fn retire_replaced_active_entries(
    catalog: &mut EditorPluginCatalog,
    entries: &mut [EditorPluginManagerEntry],
    replaced_live_package_ids: &BTreeSet<String>,
) -> Result<(), EditorPluginDiscoveryError> {
    for entry in entries.iter_mut().filter(|entry| {
        replaced_live_package_ids.contains(entry.package_id())
            && instance_requires_retirement(catalog, entry)
    }) {
        entry.state = EditorPluginState::Revoking;
        for stage in [
            EditorPluginLifecycleStage::Disabled,
            EditorPluginLifecycleStage::Unloaded,
        ] {
            if catalog.lifecycle_stage_succeeded(entry.package_id(), &stage) {
                continue;
            }
            let report = catalog.record_lifecycle_event(
                entry.package_id(),
                EditorPluginLifecycleEvent::new(stage.clone()),
            );
            if !report.is_success() {
                entry.state = EditorPluginState::Faulted;
                return Err(EditorPluginDiscoveryError::LifecycleCleanupFailed {
                    package_id: entry.package_id.clone(),
                    stage,
                });
            }
        }
    }
    Ok(())
}

pub(super) fn dispatch_hot_reloaded_replacements(
    catalog: &mut EditorPluginCatalog,
    entries: &mut [EditorPluginManagerEntry],
    replaced_live_package_ids: &BTreeSet<String>,
) {
    for entry in entries.iter_mut().filter(|entry| {
        replaced_live_package_ids.contains(entry.package_id())
            && entry.state == EditorPluginState::Active
    }) {
        let report = catalog.record_lifecycle_event(
            entry.package_id(),
            EditorPluginLifecycleEvent::new(EditorPluginLifecycleStage::HotReloaded),
        );
        if !report.is_success() {
            entry.state = EditorPluginState::Faulted;
        }
    }
}
