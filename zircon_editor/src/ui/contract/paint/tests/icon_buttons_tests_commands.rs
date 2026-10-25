use super::super::push_icon_button_commands;
use super::support::{frame_rect, icon_node};
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

#[test]
fn pressed_toolbar_icon_button_offsets_glyph_without_overlay_command() {
    let (normal_count, normal_y) = toolbar_icon_command_count_and_image_y(false);
    let (pressed_count, pressed_y) = toolbar_icon_command_count_and_image_y(true);

    assert_eq!(normal_count, 2, "one surface and one glyph");
    assert_eq!(pressed_count, normal_count);
    assert!((pressed_y - normal_y - METRICS.button_pressed_offset_y).abs() < 0.001);
}

#[test]
fn dock_tab_close_emits_the_actual_16px_glyph_without_surface_or_pressed_offset() {
    for (hovered, pressed, disabled) in [
        (false, false, false),
        (true, false, false),
        (false, true, false),
        (true, true, false),
        (true, false, true),
    ] {
        let mut node = icon_node(
            "DockTabClose0",
            "editor_pages/unreal/close-small.svg",
            false,
            20.0,
            20.0,
        );
        node.surface_variant = "transparent".into();
        node.button_variant = "text".into();
        node.hovered = hovered;
        node.pressed = pressed;
        node.disabled = disabled;
        let rect = frame_rect(&node.frame);
        let mut commands = Vec::new();
        assert!(push_icon_button_commands(
            &mut commands,
            &node,
            &rect,
            &rect,
            10,
            1.0
        ));
        assert_eq!(
            commands.len(),
            1,
            "Close must emit only its actual SVG glyph"
        );
        let command = &commands[0];
        assert!(
            command.image_pixels.is_some(),
            "Close asset must be drawn through the real painter gate"
        );
        assert_eq!((command.frame.x, command.frame.y), (8.0, 8.0));
        assert_eq!((command.frame.width, command.frame.height), (16.0, 16.0));
        let expected_opacity = if disabled { 115.0 / 255.0 } else { 1.0 };
        assert!((command.opacity - expected_opacity).abs() < 0.0001);
    }
}

fn toolbar_icon_command_count_and_image_y(pressed: bool) -> (usize, f32) {
    let mut node = icon_node(
        "WorkbenchToolbarMenu",
        "zircon_editor_shell/toolbar/menu.svg",
        false,
        30.0,
        30.0,
    );
    // Compare hover with press: both states own the same single surface.
    node.hovered = true;
    node.pressed = pressed;
    let rect = frame_rect(&node.frame);
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 64.0,
        height: 64.0,
    };
    let mut commands = Vec::new();

    assert!(push_icon_button_commands(
        &mut commands,
        &node,
        &rect,
        &clip,
        10,
        1.0,
    ));
    let image_y = commands
        .iter()
        .find(|command| command.image_pixels.is_some())
        .expect("toolbar icon asset should emit an image command")
        .frame
        .y;

    (commands.len(), image_y)
}
