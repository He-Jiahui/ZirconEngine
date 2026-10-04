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
