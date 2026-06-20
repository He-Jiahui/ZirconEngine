use super::super::super::RetainedEditorHost;
use crate::ui::retained_host::asset_pointer::AssetListPointerState;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn write_asset_content_pointer_state(
        &mut self,
        surface_mode: &str,
        state: AssetListPointerState,
    ) {
        if let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) {
            surface.content_state = state;
        }
        self.apply_asset_pointer_state_to_ui(surface_mode);
    }
}
