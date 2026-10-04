//! Atomic lifecycle cleanup and activation for plugin instance replacement.

use std::collections::HashSet;

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

// 仅旧实例仍可能持有生命周期资源且新目录实例身份变化时需要清理；项目原生登记按新宿主实例处理。
pub(super) fn replaced_live_package_ids(
    previous: &EditorPluginManagerSnapshot,
    previous_catalog: &EditorPluginCatalog,
    candidate_catalog: &EditorPluginCatalog,
) -> HashSet<String> {
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

// 在新目录可见前完成旧实例 Disabled/Unloaded；失败会将旧目录及故障状态重新发布，以便下一次替换重试。
pub(super) fn retire_replaced_active_entries(
    catalog: &mut EditorPluginCatalog,
    entries: &mut [EditorPluginManagerEntry],
    replaced_live_package_ids: &HashSet<String>,
) -> Result<(), EditorPluginDiscoveryError> {
    let retirement_ids = entries
        .iter()
        .filter(|entry| {
            replaced_live_package_ids.contains(entry.package_id())
                && instance_requires_retirement(catalog, entry)
        })
        .map(|entry| entry.package_id.clone())
        .collect::<HashSet<_>>();
    for entry in entries
        .iter_mut()
        .filter(|entry| retirement_ids.contains(entry.package_id()))
    {
        entry.state = EditorPluginState::Revoking;
        for stage in [
            EditorPluginLifecycleStage::Disabled,
            EditorPluginLifecycleStage::Unloaded,
        ] {
            // BUG: [CR-EDITOR-SERVICES-0002] 这里的成功标记覆盖插件整个登记历史，不代表当前激活周期已完成清理。Active→Disabled→Active 后替换会误跳过本轮 Disabled 回调。
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

// 旧实例退役和新实例激活完成后才向本轮成功 Active 的替换包发 HotReloaded；失败只故障化对应条目。
pub(super) fn dispatch_hot_reloaded_replacements(
    catalog: &mut EditorPluginCatalog,
    entries: &mut [EditorPluginManagerEntry],
    replaced_live_package_ids: &HashSet<String>,
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

#[cfg(test)]
#[path = "tests/lifecycle_replacement_optimization_tests.rs"]
mod optimization_tests;
