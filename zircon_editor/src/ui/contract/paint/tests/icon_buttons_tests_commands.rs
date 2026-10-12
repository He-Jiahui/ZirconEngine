use super::super::push_icon_button_commands;
use super::support::{frame_rect, icon_node};
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

#[test]
fn pressed_toolbar_icon_button_offsets_glyph_without_overlay_command() {
    let (normal_count, normal_y) = toolbar_icon_command_count_and_image_y(false);
    let (pressed_count, pressed_y) = toolbar_icon_command_count_and_image_y(true);

    assert_eq!(pressed_count, normal_count);
    assert!((pressed_y - normal_y - METRICS.button_pressed_offset_y).abs() < 0.001);
}

fn toolbar_icon_command_count_and_image_y(pressed: bool) -> (usize, f32) {
    let mut node = icon_node(
        "WorkbenchToolbarMenu",
        "zircon_editor_shell/toolbar/menu.svg",
        false,
        30.0,
        30.0,
    );
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
