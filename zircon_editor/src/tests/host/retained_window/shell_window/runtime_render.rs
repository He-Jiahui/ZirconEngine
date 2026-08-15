use super::support::*;

#[test]
fn rust_owned_host_painter_draws_runtime_render_commands() {
    let commands = vec![
        runtime_quad_command(1, 10.0, 10.0, 70.0, 36.0, 0, "#112233", "#89abcd"),
        runtime_quad_command(2, 20.0, 20.0, 38.0, 24.0, 4, "#44aa66", "#44aa66"),
        runtime_text_command(3, 12.0, 58.0, 130.0, 20.0, "Runtime Text"),
        runtime_image_command(4, 92.0, 12.0, 34.0, 34.0),
    ];

    let bytes = paint_runtime_render_commands_for_test(150, 90, &commands);

    assert_eq!(pixel(150, &bytes, 16, 16), [17, 34, 51, 255]);
    assert_eq!(
        pixel(150, &bytes, 28, 28),
        [68, 170, 102, 255],
        "higher z-index runtime commands should paint over lower commands"
    );
    assert!(
        lit_row_count(150, &bytes, 12, 58, 130, 20) > 0,
        "runtime text command should draw glyph pixels"
    );
    assert!(
        lit_row_count(150, &bytes, 0, 50, 145, 36) > 6,
        "runtime text should occupy glyph-height rows instead of a 3px placeholder bar"
    );
    assert_ne!(
        pixel(150, &bytes, 108, 28),
        [0, 0, 0, 255],
        "runtime image command should draw resolved icon pixels"
    );
}

#[test]
fn rust_owned_host_painter_resolves_runtime_svg_image_assets() {
    let image = paint_runtime_render_commands_for_test(
        80,
        56,
        &[runtime_image_command_with_asset(
            41,
            10.0,
            10.0,
            36.0,
            36.0,
            UiVisualAssetRef::Image("ui/editor/showcase_checker.svg".to_string()),
        )],
    );
    let fallback = paint_runtime_render_commands_for_test(
        80,
        56,
        &[runtime_image_command_with_asset(
            42,
            10.0,
            10.0,
            36.0,
            36.0,
            UiVisualAssetRef::Image("missing/not-found.svg".to_string()),
        )],
    );
    let res_icon_as_image = paint_runtime_render_commands_for_test(
        80,
        56,
        &[runtime_image_command_with_asset(
            43,
            10.0,
            10.0,
            36.0,
            36.0,
            UiVisualAssetRef::Image("res://icons/ionicons/options-outline.svg".to_string()),
        )],
    );
    let ionicons_icon_alias = paint_runtime_render_commands_for_test(
        80,
        56,
        &[runtime_image_command_with_asset(
            44,
            10.0,
            10.0,
            36.0,
            36.0,
            UiVisualAssetRef::Icon("ionicons/options-outline.svg".to_string()),
        )],
    );

    assert_ne!(
        pixel(80, &image, 28, 28),
        pixel(80, &fallback, 28, 28),
        "runtime SVG image assets should draw decoded pixels instead of the deterministic placeholder"
    );
    assert_eq!(
        pixel(80, &image, 16, 16),
        [77, 137, 255, 255],
        "showcase checker SVG should preserve decoded RGBA image color"
    );
    assert_ne!(
        pixel(80, &res_icon_as_image, 28, 28),
        pixel(80, &fallback, 28, 28),
        "res:// icon SVG image aliases should resolve through the editor assets tree"
    );
    assert_ne!(
        pixel(80, &ionicons_icon_alias, 28, 28),
        pixel(80, &fallback, 28, 28),
        "ionicons/name.svg icon aliases should resolve through the icon asset tree"
    );
}
