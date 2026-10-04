use super::*;

#[test]
fn primary_workbench_button_paints_low_emphasis_surface_and_center_text() {
    let bytes = paint_template_nodes_for_test(
        152,
        48,
        model_rc(vec![positioned_button_node(
            "WorkbenchPrimaryButton",
            "Primary",
            "filled",
            12.0,
            8.0,
            120.0,
            34.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 152, 24, 24), PALETTE.surface_pressed);
    assert!(changed_pixel_count(&bytes, 152, 48, 16, 56, 18) > 0);
    assert_eq!(pixel_at(&bytes, 152, 140, 24), [0, 0, 0, 255]);
}

#[test]
fn danger_workbench_button_paints_neutral_chrome_instead_of_error_slab() {
    let mut node = positioned_button_node(
        "WorkbenchDangerButton",
        "Delete",
        "danger",
        12.0,
        8.0,
        120.0,
        34.0,
    );
    node.validation_level = "danger".into();
    node.button_style =
        resolved_button_style(PALETTE.error_container, PALETTE.error, PALETTE.error, 1.0);

    let bytes = paint_template_nodes_for_test(152, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 152, 24, 24), PALETTE.surface_pressed);
    assert_eq!(pixel_at(&bytes, 152, 72, 8), PALETTE.border);
    assert_ne!(pixel_at(&bytes, 152, 24, 24), PALETTE.error_container);
    assert_ne!(pixel_at(&bytes, 152, 72, 8), PALETTE.error);
}

#[test]
fn outlined_workbench_button_paints_dark_surface_and_border() {
    let bytes = paint_template_nodes_for_test(
        152,
        48,
        model_rc(vec![positioned_button_node(
            "WorkbenchSecondaryButton",
            "Secondary",
            "outlined",
            12.0,
            8.0,
            120.0,
            34.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 152, 24, 24), PALETTE.surface_pressed);
    assert_eq!(pixel_at(&bytes, 152, 72, 8), PALETTE.border);
    assert!(changed_pixel_count(&bytes, 152, 42, 16, 70, 18) > 0);
}

#[test]
fn outlined_button_state_layer_paints_between_surface_and_content_when_enabled() {
    let mut enabled = positioned_button_node(
        "WorkbenchStateLayerButton",
        "State Layer",
        "outlined",
        12.0,
        8.0,
        120.0,
        34.0,
    );
    enabled.hovered = true;
    enabled.state_layer_enabled = true;
    let mut disabled = enabled.clone();
    disabled.state_layer_enabled = false;
    let origin = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 152.0,
        height: 48.0,
    };
    let clip = origin.clone();
    let mut enabled_commands = Vec::new();
    let mut disabled_commands = Vec::new();

    push_template_node_commands(&mut enabled_commands, &enabled, &origin, &clip, None, 0);
    push_template_node_commands(&mut disabled_commands, &disabled, &origin, &clip, None, 0);

    let button_frame = enabled.frame_rect();
    let is_full_frame_borderless_overlay = |command: &HostPaintCommand| {
        command.frame == button_frame
            && command.background_color.is_some()
            && command.border_color.is_none()
            && command.border_width == 0.0
            && command.text.is_none()
            && command.image_key.is_none()
            && command.image_pixels.is_none()
    };
    let enabled_overlays: Vec<_> = enabled_commands
        .iter()
        .enumerate()
        .filter(|(_, command)| is_full_frame_borderless_overlay(command))
        .collect();
    let disabled_overlays: Vec<_> = disabled_commands
        .iter()
        .enumerate()
        .filter(|(_, command)| is_full_frame_borderless_overlay(command))
        .collect();
    let &(overlay_index, overlay) = enabled_overlays
        .first()
        .expect("enabled button should add one full-frame borderless overlay");
    let base = enabled_commands
        .iter()
        .enumerate()
        .find(|(_, command)| command.frame == button_frame && command.border_color.is_some())
        .expect("outlined button base surface command");
    let content = enabled_commands
        .iter()
        .enumerate()
        .find(|(_, command)| command.text.as_deref() == Some("State Layer"))
        .expect("button content command");
    let base_key = (base.1.z_index, base.0);
    let overlay_key = (overlay.z_index, overlay_index);
    let content_key = (content.1.z_index, content.0);

    assert_eq!(enabled_commands.len(), disabled_commands.len() + 1);
    assert_eq!(enabled_overlays.len(), 1);
    assert!(disabled_overlays.is_empty());
    assert_eq!(overlay.frame, button_frame);
    assert!(base_key < overlay_key);
    assert!(overlay_key < content_key);
}

#[test]
fn pressed_selected_tab_orders_state_ripple_indicator_before_content() {
    let mut node = positioned_button_node(
        "DockTabStateLayer",
        "Layered Tab",
        "outlined",
        12.0,
        8.0,
        120.0,
        34.0,
    );
    node.selected = true;
    node.pressed = true;
    node.state_layer_enabled = true;
    node.ripple_enabled = true;
    node.ripple_pressed_x = 60.0;
    node.ripple_pressed_y = 17.0;
    let origin = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 152.0,
        height: 52.0,
    };
    let clip = origin.clone();
    let button_frame = node.frame_rect();
    let mut commands = Vec::new();

    push_template_node_commands(&mut commands, &node, &origin, &clip, None, 0);

    let surface_order = commands
        .iter()
        .map(|command| command.z_index)
        .min()
        .expect("button commands should include a base surface");
    let base = commands
        .iter()
        .enumerate()
        .find(|(_, command)| command.frame == button_frame && command.z_index == surface_order)
        .expect("tab base surface command");
    let state_layer = commands
        .iter()
        .enumerate()
        .find(|(_, command)| {
            command.frame == button_frame
                && command.z_index > surface_order
                && command.border_color.is_none()
                && command.border_width == 0.0
                && command.text.is_none()
        })
        .expect("full-frame state-layer command");
    let ripple = commands
        .iter()
        .enumerate()
        .find(|(_, command)| {
            command.frame.width > button_frame.width
                && command.frame.height > button_frame.height
                && command.border_color.is_none()
                && command.text.is_none()
        })
        .expect("expanded ripple command");
    let indicator = commands
        .iter()
        .enumerate()
        .find(|(_, command)| {
            command.frame.y > button_frame.y
                && command.frame.width <= button_frame.width
                && command.frame.height < button_frame.height
                && command.background_color.is_some()
                && command.border_color.is_none()
                && command.text.is_none()
        })
        .expect("selected tab indicator command");
    let content = commands
        .iter()
        .enumerate()
        .find(|(_, command)| command.text.as_deref() == Some("Layered Tab"))
        .expect("tab content command");

    let base_key = (base.1.z_index, base.0);
    let state_layer_key = (state_layer.1.z_index, state_layer.0);
    let ripple_key = (ripple.1.z_index, ripple.0);
    let indicator_key = (indicator.1.z_index, indicator.0);
    let content_key = (content.1.z_index, content.0);

    assert_eq!(ripple.1.z_index, indicator.1.z_index);
    assert!(ripple.0 < indicator.0);
    assert!(base_key < state_layer_key);
    assert!(state_layer_key < ripple_key);
    assert!(ripple_key < indicator_key);
    assert!(indicator_key < content_key);
}
