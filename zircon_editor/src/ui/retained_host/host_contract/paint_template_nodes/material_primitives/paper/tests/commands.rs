use super::*;

#[test]
fn non_finite_paper_origins_do_not_emit_surface_or_shadow_commands() {
    let node = TemplatePaneNodeData {
        component_role: "paper".to_owned(),
        elevation: 4.0,
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 8.0,
        y: f32::NEG_INFINITY,
        width: 48.0,
        height: 24.0,
    };
    let mut commands = Vec::new();

    assert!(push_paper_primitive_commands(
        &mut commands,
        &node,
        &rect,
        &rect,
        0,
        1.0,
    ));
    assert!(commands.is_empty());
}
