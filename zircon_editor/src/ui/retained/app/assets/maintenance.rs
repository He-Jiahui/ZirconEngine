use super::super::*;
use crate::core::asset::{
    EditorAssetFolderOperation, EditorAssetFolderResult, EditorAssetFolderTicket,
    EditorAssetReimportResult, EditorAssetReimportTicket,
};
use crate::core::jobs::{JobError, JobId};
use zircon_runtime::asset::AssetUri;

enum AssetMaintenanceTicket {
    Folder(EditorAssetFolderTicket),
    Reimport(EditorAssetReimportTicket),
}

enum AssetMaintenanceResult {
    Folder(EditorAssetFolderResult),
    Reimport(EditorAssetReimportResult),
}

pub(in crate::ui::retained_host::app) struct PendingAssetMaintenance {
    ticket: AssetMaintenanceTicket,
    close_requested: bool,
}

impl AssetMaintenanceTicket {
    fn id(&self) -> JobId {
        match self {
            Self::Folder(ticket) => ticket.id(),
            Self::Reimport(ticket) => ticket.id(),
        }
    }

    fn try_take(&self) -> Option<Result<AssetMaintenanceResult, JobError>> {
        match self {
            Self::Folder(ticket) => ticket
                .try_take()
                .map(|result| result.map(AssetMaintenanceResult::Folder)),
            Self::Reimport(ticket) => ticket
                .try_take()
                .map(|result| result.map(AssetMaintenanceResult::Reimport)),
        }
    }
}

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn ensure_asset_mutation_idle(
        &self,
    ) -> Result<(), String> {
        if self.pending_asset_maintenance.is_some()
            || self.pending_asset_relocation.is_some()
            || self.pending_asset_deletion.is_some()
            || self.pending_model_import.is_some()
        {
            return Err(self.asset_operation_message("asset.operation.busy"));
        }
        Ok(())
    }

    pub(in crate::ui::retained_host::app) fn ensure_asset_source_not_open_scene(
        &self,
        source: &str,
        folder: bool,
    ) -> Result<(), String> {
        let identity = self
            .editor_manager
            .enter()
            .map_err(|error| error.to_string())?
            .active_scene_identity_for_session();
        if identity.is_some_and(|identity| source_contains(source, identity.scene_uri(), folder)) {
            return Err(self.asset_operation_message("asset.operation.open_scene"));
        }
        Ok(())
    }

    pub(in crate::ui::retained_host::app) fn request_folder_mutation(
        &mut self,
        operation: EditorAssetFolderOperation,
        source: &str,
        target: Option<&str>,
    ) -> Result<(), String> {
        self.ensure_asset_mutation_idle()?;
        if operation == EditorAssetFolderOperation::Rename {
            self.ensure_asset_source_not_open_scene(source, true)?;
        }
        let source = AssetUri::parse(source).map_err(|error| error.to_string())?;
        let target = target
            .map(AssetUri::parse)
            .transpose()
            .map_err(|error| error.to_string())?;
        let ticket = self
            .editor_asset_manager_at_use_point()
            .map_err(|error| error.to_string())?
            .submit_project_folder_mutation(operation, source, target)
            .map_err(|error| error.to_string())?;
        self.pending_asset_maintenance = Some(PendingAssetMaintenance {
            ticket: AssetMaintenanceTicket::Folder(ticket),
            close_requested: false,
        });
        self.ui
            .set_lifecycle_frame_update(Some(std::time::Instant::now()));
        Ok(())
    }

    pub(in crate::ui::retained_host::app) fn request_asset_reimport(
        &mut self,
        asset_uuid: &str,
    ) -> Result<(), String> {
        self.ensure_asset_mutation_idle()?;
        let ticket = self
            .editor_asset_manager_at_use_point()
            .map_err(|error| error.to_string())?
            .submit_project_asset_reimport(asset_uuid)
            .map_err(|error| error.to_string())?;
        self.pending_asset_maintenance = Some(PendingAssetMaintenance {
            ticket: AssetMaintenanceTicket::Reimport(ticket),
            close_requested: false,
        });
        self.ui
            .set_lifecycle_frame_update(Some(std::time::Instant::now()));
        Ok(())
    }

    pub(in crate::ui::retained_host::app) fn poll_asset_maintenance(&mut self) {
        let Some(pending) = self.pending_asset_maintenance.take() else {
            return;
        };
        let Some(result) = pending.ticket.try_take() else {
            self.pending_asset_maintenance = Some(pending);
            return;
        };
        if pending.close_requested {
            if let Err(error) = self.commit_project_close() {
                self.set_status_line(error.to_string());
            }
            return;
        }
        match result {
            Ok(AssetMaintenanceResult::Folder(result)) => {
                self.sync_asset_workspace();
                match result.operation() {
                    EditorAssetFolderOperation::Create => {
                        self.runtime
                            .shell()
                            .lock()
                            .state
                            .select_asset_folder(result.source().to_string());
                        self.set_status_line(
                            self.asset_operation_message("asset.operation.folder_created")
                                .replace("{source}", &result.source().to_string()),
                        );
                    }
                    EditorAssetFolderOperation::Rename => {
                        if let Some(target) = result.target() {
                            self.runtime
                                .shell()
                                .lock()
                                .state
                                .select_asset_folder(target.to_string());
                            self.set_status_line(
                                self.asset_operation_message("asset.operation.folder_renamed")
                                    .replace("{target}", &target.to_string()),
                            );
                        }
                    }
                    EditorAssetFolderOperation::Delete => {
                        self.set_status_line(
                            self.asset_operation_message("asset.operation.folder_deleted")
                                .replace("{source}", &result.source().to_string()),
                        );
                    }
                }
                self.presentation_dirty = true;
            }
            Ok(AssetMaintenanceResult::Reimport(result)) => {
                self.sync_asset_workspace();
                if result.status().is_some_and(|status| status.imported) {
                    self.set_status_line(
                        self.asset_operation_message("asset.operation.reimported")
                            .replace("{source}", &result.source().to_string()),
                    );
                } else {
                    self.set_status_line(
                        self.asset_operation_message("asset.operation.reimport_failed")
                            .replace("{source}", &result.source().to_string()),
                    );
                }
            }
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    fn asset_operation_message(&self, key: &str) -> String {
        self.runtime.context().i18n().translate(key).to_string()
    }

    pub(in crate::ui::retained_host::app) fn cancel_pending_asset_maintenance(&mut self) -> bool {
        let Some(mut pending) = self.pending_asset_maintenance.take() else {
            return true;
        };
        let _ = self.runtime.context().jobs().cancel(pending.ticket.id());
        if pending.ticket.try_take().is_some() {
            return true;
        }
        pending.close_requested = true;
        self.pending_asset_maintenance = Some(pending);
        false
    }
}

fn source_contains(source: &str, scene: &str, folder: bool) -> bool {
    scene == source
        || (folder
            && scene
                .strip_prefix(source)
                .is_some_and(|suffix| suffix.starts_with('/')))
}

#[cfg(test)]
mod tests {
    use super::source_contains;

    #[test]
    fn folder_rename_guard_matches_components_and_asset_identity() {
        assert!(source_contains(
            "res://Scenes",
            "res://Scenes/main.zscene",
            true
        ));
        assert!(!source_contains(
            "res://Scene",
            "res://Scenes/main.zscene",
            true
        ));
        assert!(!source_contains(
            "res://Scenes",
            "res://Scenes/main.zscene",
            false
        ));
        assert!(source_contains(
            "res://Scenes/main.zscene",
            "res://Scenes/main.zscene",
            false
        ));
    }
}
