use toml::Value;
use zircon_runtime_interface::ui::{layout::UiFrame, tree::UiTemplateNodeMetadata};

use super::{anchored_popup_frame, PopupPlacement};

#[test]
fn popup_flips_from_trigger_frame_before_clamping() {
    let frame = anchored_popup_frame(
        &UiTemplateNodeMetadata::default(),
        UiFrame::new(10.0, 90.0, 20.0, 10.0),
        80.0,
        40.0,
        Some(UiFrame::new(0.0, 0.0, 120.0, 100.0)),
        PopupPlacement::BottomStart,
        4.0,
    )
    .expect("valid popup geometry");

    assert_eq!(frame, UiFrame::new(10.0, 46.0, 80.0, 40.0));
}

#[test]
fn oversized_popup_is_constrained_to_layout_bounds() {
    let bounds = UiFrame::new(0.0, 0.0, 100.0, 80.0);
    let frame = anchored_popup_frame(
        &UiTemplateNodeMetadata::default(),
        UiFrame::new(40.0, 30.0, 20.0, 20.0),
        240.0,
        200.0,
        Some(bounds),
        PopupPlacement::BottomStart,
        4.0,
    )
    .expect("valid popup geometry");

    assert_eq!(frame, bounds);
}

#[test]
fn center_placement_uses_the_surface_center() {
    let bounds = UiFrame::new(10.0, 20.0, 800.0, 600.0);
    let mut metadata = UiTemplateNodeMetadata::default();
    metadata
        .attributes
        .insert("placement".to_string(), Value::String("center".to_string()));

    let frame = anchored_popup_frame(
        &metadata,
        bounds,
        400.0,
        200.0,
        Some(bounds),
        PopupPlacement::Top,
        0.0,
    )
    .expect("valid centered popup geometry");

    assert_eq!(frame, UiFrame::new(210.0, 220.0, 400.0, 200.0));
}
