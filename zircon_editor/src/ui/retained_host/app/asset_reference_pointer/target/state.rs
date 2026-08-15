use super::super::super::{
    AssetReferenceListSurfacePointerState, AssetSurfacePointerState, RetainedEditorHost,
};
use crate::ui::retained_host::asset_pointer::AssetListPointerState;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn write_asset_reference_pointer_state(
        &mut self,
        surface_mode: &str,
        list_kind: &str,
        state: AssetListPointerState,
    ) {
        if let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) {
            if let Some(list) = surface.reference_list_mut(list_kind) {
                list.state = state;
            }
            let _ = clear_inactive_reference_hover(surface, list_kind);
        }
        self.apply_asset_pointer_state_to_ui(surface_mode);
    }

    pub(in crate::ui::retained_host::app) fn clear_inactive_asset_reference_pointer_hover(
        &mut self,
        surface_mode: &str,
        active_list_kind: &str,
    ) {
        let mut cleared = false;
        if let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) {
            cleared = clear_inactive_reference_hover(surface, active_list_kind);
        }
        if cleared {
            self.apply_asset_pointer_state_to_ui(surface_mode);
        }
    }

    pub(in crate::ui::retained_host::app) fn clear_asset_reference_pointer_hover(
        &mut self,
        surface_mode: &str,
    ) {
        let mut cleared = false;
        if let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) {
            cleared = clear_reference_hover(surface);
        }
        if cleared {
            self.apply_asset_pointer_state_to_ui(surface_mode);
        }
    }
}

fn clear_inactive_reference_hover(
    surface: &mut AssetSurfacePointerState,
    active_list_kind: &str,
) -> bool {
    match active_list_kind {
        "references" => clear_reference_list_hover(&mut surface.used_by),
        "used_by" => clear_reference_list_hover(&mut surface.references),
        _ => false,
    }
}

fn clear_reference_hover(surface: &mut AssetSurfacePointerState) -> bool {
    let references_cleared = clear_reference_list_hover(&mut surface.references);
    let used_by_cleared = clear_reference_list_hover(&mut surface.used_by);
    references_cleared || used_by_cleared
}

fn clear_reference_list_hover(list: &mut AssetReferenceListSurfacePointerState) -> bool {
    let state_cleared = list.state.hovered_row_index.take().is_some();
    let bridge_cleared = list.bridge.clear_hovered_row();
    state_cleared || bridge_cleared
}

#[cfg(test)]
mod tests {
    use super::{clear_inactive_reference_hover, clear_reference_hover};
    use crate::ui::retained_host::app::AssetSurfacePointerState;

    #[test]
    fn moving_between_reference_lists_clears_the_inactive_hover() {
        let mut surface = AssetSurfacePointerState::new();
        surface.references.state.hovered_row_index = Some(1);
        surface.references.state.scroll_offset = 24.0;
        surface.used_by.state.hovered_row_index = Some(2);
        surface.used_by.state.scroll_offset = 52.0;

        assert!(clear_inactive_reference_hover(&mut surface, "references"));
        assert_eq!(surface.references.state.hovered_row_index, Some(1));
        assert_eq!(surface.used_by.state.hovered_row_index, None);

        surface.used_by.state.hovered_row_index = Some(2);
        assert!(clear_inactive_reference_hover(&mut surface, "used_by"));
        assert_eq!(surface.references.state.hovered_row_index, None);
        assert_eq!(surface.used_by.state.hovered_row_index, Some(2));
        assert!(!clear_inactive_reference_hover(&mut surface, "used_by"));
    }

    #[test]
    fn leaving_reference_lists_clears_both_hover_states_without_scrolling_them() {
        let mut surface = AssetSurfacePointerState::new();
        surface.references.state.hovered_row_index = Some(1);
        surface.references.state.scroll_offset = 24.0;
        surface.used_by.state.hovered_row_index = Some(2);
        surface.used_by.state.scroll_offset = 52.0;

        assert!(clear_reference_hover(&mut surface));

        assert_eq!(surface.references.state.hovered_row_index, None);
        assert_eq!(surface.used_by.state.hovered_row_index, None);
        assert_eq!(surface.references.state.scroll_offset, 24.0);
        assert_eq!(surface.used_by.state.scroll_offset, 52.0);
        assert!(!clear_reference_hover(&mut surface));
    }
}
