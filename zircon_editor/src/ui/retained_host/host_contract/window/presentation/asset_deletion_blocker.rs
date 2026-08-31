use super::super::super::data::HostAssetDeletionBlockerData;
use super::super::super::redraw::HostRedrawRequest;
use super::super::UiHostWindow;

impl UiHostWindow {
    pub(crate) fn set_asset_deletion_blocker(&self, blocker: HostAssetDeletionBlockerData) {
        let damage = {
            let mut state = self.state.borrow_mut();
            let current = state.host_presentation.asset_deletion_blocker.clone();
            let damage = if current.visible {
                current.overlay_frame
            } else {
                blocker.overlay_frame.clone()
            };
            state.update_host_presentation(|presentation| {
                presentation.asset_deletion_blocker = blocker;
            });
            damage
        };
        self.queue_external_redraw(HostRedrawRequest::region(damage));
    }

    pub(crate) fn clear_asset_deletion_blocker(&self) {
        self.set_asset_deletion_blocker(HostAssetDeletionBlockerData::default());
    }
}
