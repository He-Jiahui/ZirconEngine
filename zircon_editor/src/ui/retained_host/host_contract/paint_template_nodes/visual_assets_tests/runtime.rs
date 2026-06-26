use super::super::{load_existing_icon_asset_pixels_for_size, load_visual_asset_pixels_for_size};
use super::support::has_visible_pixel;
use zircon_runtime_interface::ui::surface::UiVisualAssetRef;

#[test]
fn runtime_svg_icon_pixels_follow_requested_target_size() {
    let icon = UiVisualAssetRef::Icon("folder-open-outline".to_string());

    let small = load_visual_asset_pixels_for_size(&icon, 16, 16)
        .expect("runtime SVG icon should render at a requested small size");
    let large = load_visual_asset_pixels_for_size(&icon, 48, 48)
        .expect("runtime SVG icon should render at a requested large size");

    assert_eq!((small.width, small.height), (16, 16));
    assert_eq!((large.width, large.height), (48, 48));
    assert_ne!(small.rgba.len(), large.rgba.len());
    assert!(has_visible_pixel(&large));
}

#[test]
fn toolbar_shell_svg_icons_load_as_real_pixels() {
    for icon_name in [
        "zircon_editor_shell/toolbar/file-new.svg",
        "zircon_editor_shell/toolbar/folder-open.svg",
        "zircon_editor_shell/toolbar/save.svg",
        "zircon_editor_shell/toolbar/compile.svg",
        "zircon_editor_shell/toolbar/snap.svg",
        "zircon_editor_shell/toolbar/sun.svg",
        "zircon_editor_shell/toolbar/dropdown.svg",
        "zircon_editor_shell/toolbar/chevron-right.svg",
        "zircon_editor_shell/toolbar/more-vertical.svg",
        "zircon_editor_shell/toolbar/more-horizontal.svg",
        "zircon_editor_shell/controls/add.svg",
        "zircon_editor_shell/controls/check.svg",
        "zircon_editor_shell/controls/delete.svg",
        "zircon_editor_shell/activity/settings.svg",
        "zircon_editor_shell/inspector/mesh-renderer.svg",
        "zircon_editor_shell/inspector/material.svg",
        "zircon_editor_shell/scene/eye.svg",
        "zircon_editor_shell/scene/eye-off.svg",
        "zircon_editor_shell/scene/lock.svg",
        "zircon_editor_shell/scene/root.svg",
        "zircon_editor_shell/scene/sky.svg",
        "zircon_editor_shell/scene/geometry.svg",
        "zircon_editor_shell/scene/props.svg",
        "zircon_editor_shell/scene/player-start.svg",
        "zircon_editor_shell/scene/audio-zone.svg",
        "zircon_editor_shell/viewport/magnet.svg",
        "zircon_editor_shell/viewport/globe.svg",
        "zircon_editor_shell/viewport/crosshair.svg",
        "zircon_editor_shell/status/disabled.svg",
    ] {
        let pixels =
            load_existing_icon_asset_pixels_for_size(icon_name, 20, 20, Some([203, 210, 220, 255]))
                .unwrap_or_else(|| {
                    panic!("{icon_name} should load as an existing toolbar SVG icon")
                });

        assert_eq!((pixels.width, pixels.height), (20, 20));
        assert!(
            has_visible_pixel(&pixels),
            "{icon_name} should produce visible non-placeholder pixels"
        );
        assert!(
            !pixels.resource_key.starts_with("missing-icon:"),
            "{icon_name} should not resolve through the missing-icon fallback"
        );
    }
}

#[test]
fn semantic_shell_icon_aliases_load_as_real_pixels() {
    for icon_name in [
        "plus",
        "add",
        "new",
        "file-new",
        "folder",
        "save",
        "compile",
        "checkmark",
        "tick",
        "settings",
        "gear",
        "filter",
        "trash",
        "delete",
        "eye",
        "eye-off",
        "lock",
        "more",
        "more-horizontal",
        "ellipsis",
        "dropdown",
        "chevron-down",
        "chevron-right",
        "snap",
        "globe",
        "target",
        "crosshair",
        "select",
        "move",
        "rotate",
        "scale",
        "grid",
        "checkbox",
        "radio",
        "info",
        "warning",
        "error",
        "success",
        "disabled",
        "mesh",
        "material",
        "root",
        "environment",
        "level",
        "props",
        "player-start",
        "audio-zone",
    ] {
        let pixels =
            load_existing_icon_asset_pixels_for_size(icon_name, 18, 18, Some([203, 210, 220, 255]))
                .unwrap_or_else(|| {
                    panic!("{icon_name} semantic alias should resolve to a shell SVG icon")
                });

        assert_eq!((pixels.width, pixels.height), (18, 18));
        assert!(
            has_visible_pixel(&pixels),
            "{icon_name} should produce visible semantic icon pixels"
        );
        assert!(
            !pixels.resource_key.starts_with("missing-icon:"),
            "{icon_name} should not resolve through the missing-icon fallback"
        );
    }
}
