use super::*;

#[test]
fn ripple_diameter_matches_slint_width_based_source_contract() {
    let rect = frame(4.0, 8.0, 20.0, 80.0);
    let ripple = ripple_rect(&TemplatePaneNodeData::default(), &rect);
    let expected = 20.0 * RIPPLE_DIAMETER_EXPANSION;

    assert_eq!(ripple.width, expected);
    assert_eq!(ripple.height, expected);
}

#[test]
fn ripple_origin_preserves_zero_press_coordinates() {
    let node = TemplatePaneNodeData {
        ripple_pressed_x: 0.0,
        ripple_pressed_y: 0.0,
        ..TemplatePaneNodeData::default()
    };
    let rect = frame(10.0, 20.0, 24.0, 24.0);
    let ripple = ripple_rect(&node, &rect);
    let radius = ripple_diameter(&rect) * 0.5;

    assert_eq!(ripple.x + radius, rect.x);
    assert_eq!(ripple.y + radius, rect.y);
}

#[test]
fn ripple_enabled_does_not_imply_full_state_layer_overlay() {
    let node = TemplatePaneNodeData {
        ripple_enabled: true,
        pressed: true,
        ..TemplatePaneNodeData::default()
    };
    let rect = frame(0.0, 0.0, 32.0, 20.0);
    let clip = rect.clone();
    let mut commands = Vec::new();

    push_state_layer_commands(&mut commands, &node, &rect, &clip, 4.0, 0, 1.0);

    assert_eq!(commands.len(), 1);
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}
