use super::super::*;
use crate::core::asset::{
    AssetDeleteDisposition, AssetDeletePreflight, AssetSourceWritePolicy, EditorAssetDeletionTicket,
};
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::retained_host::HostAssetDeletionBlockerData;

pub(in crate::ui::retained_host::app) struct PendingAssetDeletion {
    ticket: EditorAssetDeletionTicket,
    close_requested: bool,
}

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn request_asset_deletion(
        &mut self,
        asset_uuid: &str,
    ) -> Result<(), String> {
        if self.pending_asset_deletion.is_some() {
            return Err("an asset deletion is already running".to_owned());
        }
        if self.pending_asset_relocation.is_some() {
            return Err("an asset move is already running".to_owned());
        }
        if self.pending_model_import.is_some() {
            return Err("an asset import is already running".to_owned());
        }
        let manager = self
            .editor_asset_manager_at_use_point()
            .map_err(|error| error.to_string())?;
        let preflight = manager
            .asset_delete_preflight(asset_uuid, AssetSourceWritePolicy::ProjectOnly)
            .map_err(|error| error.to_string())?;
        self.dismiss_asset_deletion_blocker();
        match preflight.disposition() {
            AssetDeleteDisposition::Allowed => {}
            AssetDeleteDisposition::BlockedByReferencers => {
                self.show_asset_deletion_blocker(asset_uuid, &preflight);
                return Ok(());
            }
            disposition => {
                return Err(format!("Asset deletion is not available: {disposition:?}"));
            }
        }
        let ticket = manager
            .submit_project_source_deletion(asset_uuid)
            .map_err(|error| error.to_string())?;
        self.pending_asset_deletion = Some(PendingAssetDeletion {
            ticket,
            close_requested: false,
        });
        self.ui
            .set_lifecycle_frame_update(Some(std::time::Instant::now()));
        Ok(())
    }

    fn show_asset_deletion_blocker(&self, requested_uuid: &str, preflight: &AssetDeletePreflight) {
        let target = preflight
            .target()
            .map(|asset| asset.locator().to_string())
            .unwrap_or_else(|| requested_uuid.to_owned());
        let mut referencers = preflight
            .referencers()
            .iter()
            .map(|asset| asset.locator().to_string())
            .collect::<Vec<_>>();
        referencers.sort_unstable();
        referencers.dedup();
        let size = self.ui.window().size();
        self.ui
            .set_asset_deletion_blocker(HostAssetDeletionBlockerData::for_window(
                size.width as f32,
                size.height as f32,
                target,
                ModelRc::with_metadata(referencers, ()),
            ));
    }

    pub(in crate::ui::retained_host::app) fn dismiss_asset_deletion_blocker(&self) {
        self.ui.clear_asset_deletion_blocker();
    }

    pub(in crate::ui::retained_host::app) fn poll_asset_deletion(&mut self) {
        let Some(pending) = self.pending_asset_deletion.take() else {
            return;
        };
        let Some(result) = pending.ticket.try_take() else {
            self.pending_asset_deletion = Some(pending);
            return;
        };
        if pending.close_requested {
            if let Err(error) = self.commit_project_close() {
                self.set_status_line(error.to_string());
            }
            return;
        }
        match result {
            Ok(result) => {
                self.sync_asset_workspace();
                self.set_status_line(format!("Deleted asset {}", result.target_uuid()));
            }
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(in crate::ui::retained_host::app) fn cancel_pending_asset_deletion(&mut self) -> bool {
        let Some(mut pending) = self.pending_asset_deletion.take() else {
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
        self.pending_asset_deletion = Some(pending);
        false
    }
}
