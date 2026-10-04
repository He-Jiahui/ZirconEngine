use super::*;

#[test]
fn held_button_paint_requires_the_exact_binding_and_physical_frame() {
    let origin = FrameRect {
        x: 20.0,
        y: 30.0,
        width: 400.0,
        height: 300.0,
    };
    let source = TemplatePaneNodeData {
        control_id: "Button".into(),
        action_id: "action".into(),
        dispatch_kind: "surface".into(),
        frame: FrameRect {
            x: 5.0,
            y: 7.0,
            width: 80.0,
            height: 24.0,
        },
        ..TemplatePaneNodeData::default()
    };
    let mut interaction = HostPaneInteractionStateData {
        pressed_template_control_id: source.control_id.clone(),
        pressed_template_action_id: source.action_id.clone(),
        pressed_template_dispatch_kind: source.dispatch_kind.clone(),
        pressed_template_frame: FrameRect {
            x: 25.0,
            y: 37.0,
            width: 80.0,
            height: 24.0,
        },
        ..HostPaneInteractionStateData::default()
    };
    let mut painted = source.clone();
    apply_template_press_to_node(&mut painted, &origin, &interaction);
    assert!(painted.pressed);
    assert!(!source.pressed, "paint must not alter the authored model");
    interaction.pressed_template_action_id = "replacement".into();
    assert!(!template_press_targets_node(&source, &origin, &interaction));
    interaction.pressed_template_action_id = source.action_id.clone();
    interaction.pressed_template_frame.x += 1.0;
    assert!(!template_press_targets_node(&source, &origin, &interaction));
    interaction.pressed_template_frame.x -= 1.0;
    let mut disabled = source.clone();
    disabled.disabled = true;
    assert!(!template_press_targets_node(
        &disabled,
        &origin,
        &interaction
    ));
    interaction.pressed_template_control_id.clear();
    assert!(!template_press_targets_node(&source, &origin, &interaction));
}
