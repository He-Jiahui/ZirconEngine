use super::*;

#[test]
fn focused_generic_interaction_surface_uses_pressed_not_selected_surface() {
    let node = TemplatePaneNodeData {
        focused: true,
        ..TemplatePaneNodeData::default()
    };

    assert_eq!(
        interaction_surface_color(&node),
        Some(PALETTE.surface_pressed)
    );
    assert_ne!(
        interaction_surface_color(&node),
        Some(PALETTE.surface_selected)
    );
}

#[test]
fn pressed_generic_interaction_surface_still_uses_pressed_surface() {
    let node = TemplatePaneNodeData {
        pressed: true,
        ..TemplatePaneNodeData::default()
    };

    assert_eq!(
        interaction_surface_color(&node),
        Some(PALETTE.surface_pressed)
    );
}

#[test]
fn selected_asset_thumbnail_name_area_still_uses_selected_surface() {
    let node = TemplatePaneNodeData {
        selected: true,
        surface_variant: "asset-thumbnail-name-area".into(),
        ..TemplatePaneNodeData::default()
    };

    assert_eq!(
        interaction_surface_color(&node),
        Some(PALETTE.surface_selected)
    );
}
