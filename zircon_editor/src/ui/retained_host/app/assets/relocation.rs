use super::super::*;
use crate::core::asset::EditorAssetRelocationTicket;
use zircon_runtime::asset::AssetUri;

pub(in crate::ui::retained_host::app) struct PendingAssetRelocation {
    ticket: EditorAssetRelocationTicket,
    close_requested: bool,
}

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn request_asset_relocation(
        &mut self,
        asset_uuid: &str,
        target_locator: &str,
    ) -> Result<(), String> {
        if self.pending_asset_relocation.is_some() {
            return Err("an asset move is already running".to_owned());
        }
        if self.pending_asset_deletion.is_some() {
            return Err("an asset deletion is already running".to_owned());
        }
        if self.pending_model_import.is_some() {
            return Err("an asset import is already running".to_owned());
        }
        let target = AssetUri::parse(target_locator).map_err(|error| error.to_string())?;
        let ticket = self
            .editor_asset_manager_at_use_point()
            .map_err(|error| error.to_string())?
            .submit_project_source_relocation(asset_uuid, target)
            .map_err(|error| error.to_string())?;
        self.pending_asset_relocation = Some(PendingAssetRelocation {
            ticket,
            close_requested: false,
        });
        self.ui
            .set_lifecycle_frame_update(Some(std::time::Instant::now()));
        Ok(())
    }

    pub(in crate::ui::retained_host::app) fn poll_asset_relocation(&mut self) {
        let Some(pending) = self.pending_asset_relocation.take() else {
            return;
        };
        let Some(result) = pending.ticket.try_take() else {
            self.pending_asset_relocation = Some(pending);
            return;
        };
        if pending.close_requested {
            if let Err(error) = self.commit_project_close() {
                self.set_status_line(error.to_string());
            }
            return;
        }
        match result {
            Ok(result) if result.changed() => {
                self.sync_asset_workspace();
                self.set_status_line(format!("Moved asset to {}", result.target()));
            }
            Ok(result) => {
                self.set_status_line(format!("Asset is already at {}", result.target()));
            }
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(in crate::ui::retained_host::app) fn cancel_pending_asset_relocation(&mut self) -> bool {
        let Some(mut pending) = self.pending_asset_relocation.take() else {
            return true;
        };
        let _ = self
            .editor_manager
            .context()
            .jobs()
            .cancel(pending.ticket.id());
        if pending.ticket.try_take().is_some() {
            return true;
        }
        pending.close_requested = true;
        self.pending_asset_relocation = Some(pending);
        false
    }
}
