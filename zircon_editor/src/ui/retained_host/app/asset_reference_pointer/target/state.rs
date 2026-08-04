use super::super::super::RetainedEditorHost;
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
            clear_inactive_reference_hover(
                &mut surface.references.state,
                &mut surface.used_by.state,
                list_kind,
            );
        }
        self.apply_asset_pointer_state_to_ui(surface_mode);
    }

    pub(in crate::ui::retained_host::app) fn clear_asset_reference_pointer_hover(
        &mut self,
        surface_mode: &str,
    ) {
        let mut cleared = false;
        if let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) {
            cleared =
                clear_reference_hover(&mut surface.references.state, &mut surface.used_by.state);
        }
        if cleared {
            self.apply_asset_pointer_state_to_ui(surface_mode);
        }
    }
}

fn clear_inactive_reference_hover(
    references: &mut AssetListPointerState,
    used_by: &mut AssetListPointerState,
    active_list_kind: &str,
) {
    match active_list_kind {
        "references" => used_by.hovered_row_index = None,
        "used_by" => references.hovered_row_index = None,
        _ => {}
    }
}

fn clear_reference_hover(
    references: &mut AssetListPointerState,
    used_by: &mut AssetListPointerState,
) -> bool {
    let cleared = references.hovered_row_index.is_some() || used_by.hovered_row_index.is_some();
    references.hovered_row_index = None;
    used_by.hovered_row_index = None;
    cleared
}

#[cfg(test)]
mod tests {
    use super::{clear_inactive_reference_hover, clear_reference_hover};
    use crate::ui::retained_host::asset_pointer::AssetListPointerState;

    #[test]
    fn moving_between_reference_lists_clears_the_inactive_hover() {
        let mut references = AssetListPointerState {
            hovered_row_index: Some(1),
            scroll_offset: 24.0,
        };
        let mut used_by = AssetListPointerState {
            hovered_row_index: Some(2),
            scroll_offset: 52.0,
        };

        clear_inactive_reference_hover(&mut references, &mut used_by, "references");
        assert_eq!(references.hovered_row_index, Some(1));
        assert_eq!(used_by.hovered_row_index, None);

        used_by.hovered_row_index = Some(2);
        clear_inactive_reference_hover(&mut references, &mut used_by, "used_by");
        assert_eq!(references.hovered_row_index, None);
        assert_eq!(used_by.hovered_row_index, Some(2));
    }

    #[test]
    fn leaving_reference_lists_clears_both_hover_states_without_scrolling_them() {
        let mut references = AssetListPointerState {
            hovered_row_index: Some(1),
            scroll_offset: 24.0,
        };
        let mut used_by = AssetListPointerState {
            hovered_row_index: Some(2),
            scroll_offset: 52.0,
        };

        assert!(clear_reference_hover(&mut references, &mut used_by));

        assert_eq!(references.hovered_row_index, None);
        assert_eq!(used_by.hovered_row_index, None);
        assert_eq!(references.scroll_offset, 24.0);
        assert_eq!(used_by.scroll_offset, 52.0);
        assert!(!clear_reference_hover(&mut references, &mut used_by));
    }
}
