use std::collections::BTreeSet;
use std::path::PathBuf;

use zircon_runtime::asset::project::ProjectManager;

use crate::ui::asset_editor::UiAssetEditorRoute;
use crate::ui::host::editor_error::EditorError;
use crate::ui::host::editor_ui_host::EditorUiHost;
use crate::ui::host::project_access::normalize_ui_asset_asset_id;
use crate::ui::workbench::view::ViewInstanceId;

use super::super::super::ui_asset_source_hash;
use super::queue::UiAssetRefreshRequest;

pub(in crate::ui::host::asset_editor_sessions) struct UiAssetRefreshPlan {
    pub(super) generation: u64,
    pub(super) retry_attempt: u8,
    pub(super) dependency_generation: u64,
    pub(super) changed_asset_ids: BTreeSet<String>,
    pub(super) project: Option<ProjectManager>,
    pub(super) project_root: Option<PathBuf>,
    pub(super) direct_instances: Vec<UiAssetDirectRefreshPlan>,
    pub(super) import_instances: Vec<UiAssetImportRefreshPlan>,
}

impl UiAssetRefreshPlan {
    pub(super) fn is_empty(&self) -> bool {
        self.direct_instances.is_empty() && self.import_instances.is_empty()
    }

    pub(super) fn request(&self) -> UiAssetRefreshRequest {
        UiAssetRefreshRequest {
            generation: self.generation,
            changed_asset_ids: self.changed_asset_ids.clone(),
            retry_attempt: self.retry_attempt,
        }
    }
}

pub(super) struct UiAssetDirectRefreshPlan {
    pub(super) instance_id: ViewInstanceId,
    pub(super) asset_id: String,
    pub(super) source_path: PathBuf,
    pub(super) route: UiAssetEditorRoute,
    pub(super) disk_source_hash: u64,
    pub(super) source_fingerprint: u64,
    pub(super) source_dirty: bool,
}

pub(super) struct UiAssetImportRefreshPlan {
    pub(super) instance_id: ViewInstanceId,
    pub(super) source_fingerprint: u64,
    pub(super) widget_refs: Vec<String>,
    pub(super) style_refs: Vec<String>,
}

impl EditorUiHost {
    pub(in crate::ui::host::asset_editor_sessions) fn build_ui_asset_refresh_plan(
        &self,
        request: UiAssetRefreshRequest,
    ) -> Result<UiAssetRefreshPlan, EditorError> {
        let dependency_generation = self.lock_ui_asset_dependency_generation();
        let impact = dependency_generation.impact(&request.changed_asset_ids);
        drop(dependency_generation);

        let sessions = self.lock_ui_asset_sessions();
        let direct_instances = impact
            .direct_instances
            .iter()
            .filter_map(|instance_id| {
                let entry = sessions.get(instance_id)?;
                Some(UiAssetDirectRefreshPlan {
                    instance_id: instance_id.clone(),
                    asset_id: normalize_ui_asset_asset_id(&entry.session.route().asset_id)
                        .to_string(),
                    source_path: entry.source_path.clone(),
                    route: entry.session.route().clone(),
                    disk_source_hash: entry.disk_source_hash,
                    source_fingerprint: ui_asset_source_hash(entry.session.source_buffer().text()),
                    source_dirty: entry.session.source_buffer().is_dirty(),
                })
            })
            .collect();
        let import_instances = impact
            .import_instances
            .iter()
            .filter_map(|instance_id| {
                let entry = sessions.get(instance_id)?;
                let (widget_refs, style_refs) = entry.session.import_references();
                Some(UiAssetImportRefreshPlan {
                    instance_id: instance_id.clone(),
                    source_fingerprint: ui_asset_source_hash(entry.session.source_buffer().text()),
                    widget_refs,
                    style_refs,
                })
            })
            .collect();
        drop(sessions);

        let project = self.current_project_snapshot()?;
        let project_root = project
            .as_ref()
            .map(|project| project.paths().root().to_path_buf());
        Ok(UiAssetRefreshPlan {
            generation: request.generation,
            retry_attempt: request.retry_attempt,
            dependency_generation: impact.generation,
            changed_asset_ids: impact.changed_asset_ids,
            project,
            project_root,
            direct_instances,
            import_instances,
        })
    }
}
