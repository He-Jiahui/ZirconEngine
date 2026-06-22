use super::super::{push_state_layer_commands, TemplatePaneNodeData};
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
