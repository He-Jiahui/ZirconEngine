use super::super::super::style_selector::workbench_dropdown_palette;
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::push_dropdown_commands;
use super::support::{changed_pixel_count, dropdown_node, option, pixel_at, scaled_test_color};
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::data::FrameRect;

#[test]
fn closed_workbench_dropdown_paints_surface_border_text_and_chevron() {
    let palette = workbench_dropdown_palette();
    let mut node = dropdown_node(false);
    node.border_width = 1.0;
    let bytes = paint_template_nodes_for_test(140, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 140, 88, 24), palette.surface);
    assert_eq!(pixel_at(&bytes, 140, 54, 8), palette.border);
    assert!(changed_pixel_count(&bytes, 140, 22, 16, 50, 18) > 0);
    assert!(changed_pixel_count(&bytes, 140, 96, 15, 18, 18) > 0);
}

#[test]
fn closed_workbench_dropdown_chevron_prefers_shell_asset_pixels() {
    let node = dropdown_node(false);
    let rect = FrameRect {
        x: 12.0,
        y: 8.0,
        width: 104.0,
        height: 32.0,
    };
    let mut commands = Vec::new();

    assert!(push_dropdown_commands(
        &mut commands,
        &node,
        &rect,
        &rect,
        0,
        1.0,
    ));

    let icon_commands = commands
        .iter()
        .filter(|command| command.image_pixels.is_some())
        .collect::<Vec<_>>();
    assert_eq!(
        icon_commands.len(),
        1,
        "dropdown should render its chevron through the shared shell SVG asset"
    );
    let icon = icon_commands[0];
    assert_eq!(icon.frame.width, 16.0);
    assert_eq!(icon.frame.height, 16.0);
    assert!(icon
        .image_pixels
        .as_ref()
        .map(|image| !image.resource_key.starts_with("missing-icon:"))
        .unwrap_or(false));
}

#[test]
fn open_workbench_dropdown_uses_focus_border_and_keeps_popup_rows() {
    let palette = workbench_dropdown_palette();
    let mut node = dropdown_node(true);
    node.border_width = 1.0;
    node.popup_open = true;
    node.structured_options = model_rc(vec![
        option("dropdown", true, false, true, false),
        option("option_a", false, true, false, false),
        option("option_b", false, false, false, true),
    ]);
    let bytes = paint_template_nodes_for_test(160, 140, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 160, 54, 8), palette.focus_border);
    assert!(changed_pixel_count(&bytes, 160, 18, 44, 110, 78) > 0);
}

#[test]
fn workbench_dropdown_honors_declared_layout_offset() {
    let palette = workbench_dropdown_palette();
    let mut node = dropdown_node(false);
    node.layout_offset_x = 20.0;
    node.layout_offset_y = 12.0;
    let bytes = paint_template_nodes_for_test(160, 80, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 160, 88, 24), palette.surface);
    assert_eq!(pixel_at(&bytes, 160, 54, 8), [0, 0, 0, 255]);
}

#[test]
fn workbench_dropdown_applies_declared_visual_brightness() {
    let palette = workbench_dropdown_palette();
    let mut node = dropdown_node(false);
    node.border_width = 1.0;
    node.label_brightness = 1.2;
    let expected_surface = scaled_test_color(palette.surface, 1.2);
    let expected_border = scaled_test_color(palette.border, 1.2);
    let bytes = paint_template_nodes_for_test(140, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 140, 88, 24), expected_surface);
    assert_eq!(pixel_at(&bytes, 140, 54, 8), expected_border);
}

#[test]
fn workbench_dropdown_keeps_first_option_label_fallback() {
    let mut node = dropdown_node(false);
    node.value_text = "".into();
    node.options = model_rc(vec!["Fallback".into()]);
    let rect = FrameRect {
        x: 12.0,
        y: 8.0,
        width: 104.0,
        height: 32.0,
    };
    let mut commands = Vec::new();

    assert!(push_dropdown_commands(
        &mut commands,
        &node,
        &rect,
        &rect,
        0,
        1.0,
    ));
    assert!(commands
        .iter()
        .any(|command| command.text.as_deref() == Some("Fallback")));
}

#[test]
fn authored_viewport_dropdown_paints_chevron_and_preserves_border_width_across_dpi() {
    use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge;
    use crate::ui::retained_host::host_contract::paint_theme::{
        enter_host_paint_theme_scope, host_paint_theme_snapshot_at_scale_for_test,
    };
    use crate::ui::retained_host::ui::to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale;
    use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};

    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
        .expect("the authored Unreal viewport must project");
    for scale in [1.0, 1.25] {
        let _scope =
            enter_host_paint_theme_scope(host_paint_theme_snapshot_at_scale_for_test(scale));
        let nodes = to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale(
            Some(bridge.host_projection()),
            None,
            Some(UiFrame::new(100.0, 50.0, 1280.0 * scale, 800.0 * scale)),
            scale,
        );
        let mounted = nodes
            .iter()
            .find(|node| node.control_id.as_str() == "SetProjectionMode")
            .expect("the actual authored projection dropdown must be mounted");
        assert_eq!(mounted.component_role.as_str(), "dropdown");
        assert_eq!(mounted.border_width, 0.0);
        let rect = FrameRect {
            x: mounted.frame.x,
            y: mounted.frame.y,
            width: mounted.frame.width,
            height: mounted.frame.height,
        };
        for border_width in [0.0, scale] {
            let mut node = mounted.clone();
            node.border_width = border_width;
            let mut commands = Vec::new();
            assert!(push_dropdown_commands(
                &mut commands,
                &node,
                &rect,
                &rect,
                0,
                1.0,
            ));
            let surface = &commands[0];
            assert_eq!(surface.background_color, Some([56, 56, 56, 255]));
            assert_eq!(surface.border_width, border_width);
            assert_eq!(surface.corner_radius, 4.0 * scale);
            let glyphs: Vec<_> = commands
                .iter()
                .filter(|command| command.image_pixels.is_some())
                .collect();
            assert_eq!(glyphs.len(), 1);
            let glyph = glyphs[0];
            assert_eq!(glyph.frame.width, 16.0 * scale);
            assert_eq!(glyph.frame.height, 16.0 * scale);
            assert_eq!(glyph.clip_frame.as_ref(), Some(&rect));
            assert!(!glyph
                .image_pixels
                .as_ref()
                .unwrap()
                .resource_key
                .starts_with("missing-icon:"));
        }
    }
}

#[test]
fn dropdown_root_resolves_label_and_metrics_once() {
    let root = include_str!("../../dropdowns/commands.rs");
    let surface = include_str!("../../dropdowns/surface.rs");
    let text = include_str!("../../dropdowns/text.rs");
    let chevron = include_str!("../../dropdown_glyphs/chevron.rs");

    assert_eq!(root.matches("dropdown_label(node)").count(), 1);
    assert_eq!(root.matches("workbench_dropdown_metrics()").count(), 1);
    assert!(!surface.contains("workbench_dropdown_metrics()"));
    assert!(!text.contains("workbench_dropdown_metrics()"));
    assert!(!chevron.contains("workbench_dropdown_metrics()"));
}
