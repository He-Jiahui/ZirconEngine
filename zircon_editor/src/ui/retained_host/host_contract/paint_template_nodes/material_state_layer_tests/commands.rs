use super::super::{TemplatePaneNodeData, push_state_layer_commands};
use super::support::frame;

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

#[test]
fn idle_state_layer_checks_paint_eligibility_before_resolving_color() {
    let source = include_str!("../material_state_layer.rs");
    let opacity = source
        .find("let overlay_opacity = state_layer_opacity(node)")
        .expect("entry should resolve overlay eligibility");
    let color = source
        .find("let color = state_layer_color(node)")
        .expect("entry should resolve color only for visible paint");

    assert!(opacity < color);
    assert!(source[opacity..color].contains("return"));
}
