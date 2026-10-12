use super::*;

#[test]
fn checked_checkbox_tick_uses_16px_shell_checkmark_asset() {
    let mark = FrameRect {
        x: 10.0,
        y: 6.0,
        width: 16.0,
        height: 16.0,
    };
    let mut commands = Vec::new();

    push_checkbox_tick(&mut commands, &mark, &mark, 3, 1.0);

    let icon_commands = commands
        .iter()
        .filter(|command| command.image_pixels.is_some())
        .collect::<Vec<_>>();
    assert_eq!(icon_commands.len(), 1);
    let icon = icon_commands[0]
        .image_pixels
        .as_ref()
        .expect("checkbox tick should paint real SVG pixels");
    assert_eq!((icon.width, icon.height), (16, 16));
    assert_eq!(icon_commands[0].frame.width, 16.0);
    assert_eq!(icon_commands[0].frame.height, 16.0);
    assert!(
        !icon.resource_key.starts_with("missing-icon:"),
        "checkbox tick should resolve through the shell checkmark asset"
    );
}
