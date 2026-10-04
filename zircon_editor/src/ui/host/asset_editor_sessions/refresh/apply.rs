use std::collections::BTreeSet;

use crate::ui::host::editor_error::EditorError;
use crate::ui::host::editor_ui_host::EditorUiHost;
use crate::ui::workbench::view::ViewInstanceId;

use super::normalize::normalize_ui_asset_change_set;

impl EditorUiHost {
    pub fn refresh_ui_asset_workspace_for_changes<I, S>(
        &self,
        changed_asset_ids: I,
    ) -> Result<(), EditorError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let changed_asset_ids = normalize_ui_asset_change_set(changed_asset_ids);
        if changed_asset_ids.is_empty() {
            return Ok(());
        }

        let dependency_generation = self.lock_ui_asset_dependency_generation();
        let impact = dependency_generation.impact(&changed_asset_ids);
        drop(dependency_generation);
        let sync_instances = self.apply_ui_asset_workspace_changes(&changed_asset_ids, &impact)?;
        self.sync_ui_asset_refresh_instances(sync_instances)
    }

    pub(super) fn apply_ui_asset_workspace_changes(
        &self,
        changed_asset_ids: &BTreeSet<String>,
        dependency_generation: &super::super::UiAssetDependencyImpact,
    ) -> Result<BTreeSet<ViewInstanceId>, EditorError> {
        let direct_sync = self.apply_direct_ui_asset_changes(
            changed_asset_ids,
            &dependency_generation.direct_instances,
        )?;
        let import_sync = self.apply_import_ui_asset_changes(
            changed_asset_ids,
            &dependency_generation.import_instances,
        )?;
        Ok(merge_refresh_instance_sets(direct_sync, import_sync))
    }

    pub(in crate::ui::host::asset_editor_sessions) fn sync_ui_asset_refresh_instances(
        &self,
        instance_ids: BTreeSet<ViewInstanceId>,
    ) -> Result<(), EditorError> {
        for instance_id in instance_ids {
            self.sync_ui_asset_editor_instance(&instance_id)?;
        }
        Ok(())
    }
}

fn merge_refresh_instance_sets(
    mut direct_sync: BTreeSet<ViewInstanceId>,
    mut import_sync: BTreeSet<ViewInstanceId>,
) -> BTreeSet<ViewInstanceId> {
    direct_sync.append(&mut import_sync);
    direct_sync
}

#[cfg(test)]
#[path = "tests/apply_optimization_tests.rs"]
mod optimization_tests;
