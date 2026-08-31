use std::collections::BTreeSet;
use std::fs;
use std::io::ErrorKind;

use crate::ui::host::editor_error::EditorError;
use crate::ui::host::editor_ui_host::EditorUiHost;
use crate::ui::host::project_access::normalize_ui_asset_asset_id;
use crate::ui::workbench::view::ViewInstanceId;

use super::super::{ui_asset_source_digest, UiAssetExternalConflict};
use super::normalize::rebuild_ui_asset_session_from_source;

impl EditorUiHost {
    pub(super) fn apply_direct_ui_asset_changes(
        &self,
        changed_asset_ids: &BTreeSet<String>,
        direct_instances: &BTreeSet<ViewInstanceId>,
    ) -> Result<BTreeSet<ViewInstanceId>, EditorError> {
        let entries = {
            let sessions = self.lock_ui_asset_sessions();
            direct_instances
                .iter()
                .filter_map(|instance_id| {
                    let entry = sessions.get(instance_id)?;
                    let asset_id =
                        normalize_ui_asset_asset_id(&entry.session.route().asset_id).to_string();
                    changed_asset_ids
                        .contains(&asset_id)
                        .then(|| (instance_id.clone(), asset_id, entry.source_path.clone()))
                })
                .collect::<Vec<_>>()
        };

        let mut sync_instances = BTreeSet::new();
        for (instance_id, asset_id, source_path) in entries {
            let external_source = match fs::read_to_string(&source_path) {
                Ok(source) => source,
                Err(error) if error.kind() == ErrorKind::NotFound => {
                    let mut sessions = self.lock_ui_asset_sessions();
                    let entry = sessions.get_mut(&instance_id).ok_or_else(|| {
                        EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
                    })?;
                    let local_source = entry.session.source_buffer().text().to_string();
                    entry.conflict = Some(UiAssetExternalConflict::new(
                        asset_id,
                        source_path,
                        entry.disk_source_digest,
                        local_source,
                        String::new(),
                    ));
                    entry.diff_snapshot = None;
                    let _ = sync_instances.insert(instance_id);
                    continue;
                }
                Err(error) => return Err(EditorError::UiAsset(error.to_string())),
            };
            let external_digest = ui_asset_source_digest(&external_source);
            let route = {
                let mut sessions = self.lock_ui_asset_sessions();
                let entry = sessions.get_mut(&instance_id).ok_or_else(|| {
                    EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
                })?;
                if external_digest == entry.disk_source_digest {
                    if entry.conflict.is_some() || entry.diff_snapshot.is_some() {
                        entry.conflict = None;
                        entry.diff_snapshot = None;
                        let _ = sync_instances.insert(instance_id.clone());
                    }
                    continue;
                }
                if entry.session.reflection_model().source_dirty {
                    let local_source = entry.session.source_buffer().text().to_string();
                    entry.conflict = Some(UiAssetExternalConflict::new(
                        asset_id,
                        source_path,
                        entry.disk_source_digest,
                        local_source,
                        external_source,
                    ));
                    entry.diff_snapshot = None;
                    let _ = sync_instances.insert(instance_id.clone());
                    continue;
                }
                entry.session.route().clone()
            };

            let session = rebuild_ui_asset_session_from_source(route, external_source.clone())?;
            self.replace_ui_asset_session_from_disk(&instance_id, session, external_source)?;
            self.hydrate_ui_asset_editor_imports(&instance_id)?;
            let _ = sync_instances.insert(instance_id);
        }
        Ok(sync_instances)
    }
}
