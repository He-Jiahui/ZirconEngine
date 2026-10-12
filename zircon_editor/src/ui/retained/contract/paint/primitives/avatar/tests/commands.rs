use super::*;

#[test]
fn non_finite_avatar_origins_do_not_emit_paint_commands() {
    let node = TemplatePaneNodeData {
        component_role: "avatar".to_owned(),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: f32::INFINITY,
        y: 8.0,
        width: 24.0,
        height: 24.0,
    };
    let mut commands = Vec::new();

    assert!(push_avatar_primitive_commands(
        &mut commands,
        &node,
        &rect,
        &rect,
        0,
        1.0,
    ));
    assert!(commands.is_empty());
}
