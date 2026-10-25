use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::push_icon_button_commands;
use super::support::{changed_pixel_count, frame_rect, icon_node, pixel_at, positioned_icon_node};
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_theme::{
    enter_host_paint_theme_scope, host_paint_theme_snapshot_at_scale_for_test, PALETTE,
};

#[test]
fn selected_toolbar_icon_button_paints_active_surface_and_glyph() {
    let bytes = paint_template_nodes_for_test(
        64,
        56,
        model_rc(vec![positioned_icon_node(
            "WorkbenchToolSelect",
            "zircon_editor_shell/toolbar/select.svg",
            true,
            8.0,
            8.0,
            48.0,
            40.0,
        )]),
    );

    assert_ne!(pixel_at(&bytes, 64, 16, 16), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 64, 22, 16, 22, 24) > 0);
}

#[test]
fn normal_toolbar_icon_button_keeps_outer_background_clean_and_draws_glyph() {
    let bytes = paint_template_nodes_for_test(
        48,
        48,
        model_rc(vec![icon_node(
            "WorkbenchToolbarMenu",
            "zircon_editor_shell/toolbar/menu.svg",
            false,
            34.0,
            34.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 48, 4, 4), [0, 0, 0, 255]);
    assert_eq!(pixel_at(&bytes, 48, 8, 8), [0, 0, 0, 255]);
    assert_eq!(pixel_at(&bytes, 48, 24, 6), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 48, 12, 12, 18, 18) > 0);
}

#[test]
fn selected_rail_icon_button_paints_large_active_surface() {
    let bytes = paint_template_nodes_for_test(
        64,
        64,
        model_rc(vec![positioned_icon_node(
            "WorkbenchRailScene",
            "zircon_editor_shell/activity/play.svg",
            true,
            8.0,
            8.0,
            48.0,
            48.0,
        )]),
    );

    assert_ne!(pixel_at(&bytes, 64, 16, 16), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 64, 25, 20, 18, 24) > 0);
}

#[test]
fn panel_danger_icon_button_paints_surface_and_error_glyph() {
    let bytes = paint_template_nodes_for_test(
        48,
        48,
        model_rc(vec![icon_node(
            "WorkbenchMiniDelete",
            "zircon_editor_shell/controls/delete.svg",
            false,
            36.0,
            36.0,
        )]),
    );

    assert_ne!(pixel_at(&bytes, 48, 8, 8), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 48, 14, 12, 20, 24) > 0);
}

#[test]
fn asset_import_icon_button_paints_primary_accent_fill() {
    let mut node = positioned_icon_node(
        "ImportModel",
        "editor_pages/asset_browser/import_pipeline/import.svg",
        false,
        8.0,
        8.0,
        80.0,
        28.0,
    );
    node.action_id = "workbench.asset.import_model".into();
    node.component_variant = "workbench-icon-button".into();

    let bytes = paint_template_nodes_for_test(104, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 104, 16, 16), PALETTE.accent);
    assert_eq!(pixel_at(&bytes, 104, 48, 8), PALETTE.accent);
}

#[test]
fn authored_play_options_icon_paints_at_logical_size_across_dpi() {
    use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge;
    use crate::ui::retained_host::ui::to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale;
    use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};

    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
        .expect("the authored Unreal toolbar must project");
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
            .find(|node| node.control_id.as_str() == "WorkbenchRunMode")
            .expect("the actual authored Play Options node must be mounted");
        assert_eq!(mounted.layout_icon_size, 16.0 * scale);
        assert_eq!(mounted.frame.width, 24.0 * scale);
        assert_eq!(mounted.frame.height, 32.0 * scale);
        for pressed in [false, true] {
            let rect = frame_rect(&mounted.frame);
            let mut node = mounted.clone();
            node.value_number = 3.0;
            node.pressed = pressed;
            let mut commands = Vec::new();
            assert!(push_icon_button_commands(
                &mut commands,
                &node,
                &rect,
                &rect,
                10,
                1.0,
            ));
            let glyphs: Vec<_> = commands
                .iter()
                .filter(|command| command.image_pixels.is_some())
                .collect();
            assert_eq!(
                glyphs.len(),
                1,
                "the actual authored dropdown asset must paint"
            );
            let glyph = glyphs[0];
            assert_eq!(glyph.frame.width, 16.0 * scale);
            assert_eq!(glyph.frame.height, 16.0 * scale);
            assert_eq!(glyph.clip_frame.as_ref(), Some(&rect));
            assert!(glyph.frame.x >= rect.x && glyph.frame.y >= rect.y);
            assert!(glyph.frame.x + glyph.frame.width <= rect.x + rect.width);
            assert!(glyph.frame.y + glyph.frame.height <= rect.y + rect.height);
        }
    }
}

#[test]
fn play_options_without_valid_authored_size_keeps_the_default_density_gate() {
    for scale in [1.0, 1.25] {
        let _scope =
            enter_host_paint_theme_scope(host_paint_theme_snapshot_at_scale_for_test(scale));
        for size in [0.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let rect = FrameRect {
                x: 8.0 * scale,
                y: 6.0 * scale,
                width: 24.0 * scale,
                height: 32.0 * scale,
            };
            let mut node = icon_node(
                "WorkbenchRunMode",
                "zircon_editor_shell/toolbar/dropdown.svg",
                false,
                24.0,
                32.0,
            );
            node.layout_icon_size = size;
            node.value_number = 16.0 * scale;
            let mut commands = Vec::new();
            assert!(push_icon_button_commands(
                &mut commands,
                &node,
                &rect,
                &rect,
                10,
                1.0,
            ));
            assert!(commands
                .iter()
                .all(|command| command.image_pixels.is_none()));
        }
    }
}

#[test]
fn authored_play_options_icon_rejects_invalid_or_insufficient_button_extent() {
    for scale in [1.0, 1.25] {
        let _scope =
            enter_host_paint_theme_scope(host_paint_theme_snapshot_at_scale_for_test(scale));
        for (width, height) in [
            (0.0, 32.0),
            (20.0, 32.0),
            (24.0, 20.0),
            (f32::NAN, 32.0),
            (24.0, f32::INFINITY),
        ] {
            let rect = FrameRect {
                x: 8.0 * scale,
                y: 6.0 * scale,
                width: width * scale,
                height: height * scale,
            };
            let mut node = icon_node(
                "WorkbenchRunMode",
                "zircon_editor_shell/toolbar/dropdown.svg",
                false,
                width,
                height,
            );
            node.layout_icon_size = 16.0 * scale;
            let mut commands = Vec::new();
            assert!(push_icon_button_commands(
                &mut commands,
                &node,
                &rect,
                &rect,
                10,
                1.0,
            ));
            assert!(commands
                .iter()
                .all(|command| command.image_pixels.is_none()));
        }
    }
}

#[test]
fn undeclared_toolbar_icon_keeps_its_default_density_size_across_dpi() {
    for scale in [1.0, 1.25] {
        let _scope =
            enter_host_paint_theme_scope(host_paint_theme_snapshot_at_scale_for_test(scale));
        let rect = FrameRect {
            x: 8.0 * scale,
            y: 6.0 * scale,
            width: 30.0 * scale,
            height: 32.0 * scale,
        };
        let node = icon_node(
            "WorkbenchToolbarMenu",
            "zircon_editor_shell/toolbar/menu.svg",
            false,
            30.0,
            32.0,
        );
        let mut commands = Vec::new();
        assert!(push_icon_button_commands(
            &mut commands,
            &node,
            &rect,
            &rect,
            10,
            1.0,
        ));
        let glyph = commands
            .iter()
            .find(|command| command.image_pixels.is_some())
            .expect("the undeclared toolbar glyph keeps its density default");
        assert_eq!(glyph.frame.width, 20.0 * scale);
        assert_eq!(glyph.frame.height, 20.0 * scale);
    }
}

#[test]
fn tab_close_ignores_authored_toolbar_icon_size() {
    let _scope = enter_host_paint_theme_scope(host_paint_theme_snapshot_at_scale_for_test(1.25));
    for size in [4.0, 40.0, f32::NAN] {
        let rect = FrameRect {
            x: 6.0,
            y: 6.0,
            width: 20.0,
            height: 20.0,
        };
        let mut node = icon_node(
            "DockTabClose0",
            "editor_pages/unreal/close-small.svg",
            false,
            20.0,
            20.0,
        );
        node.layout_icon_size = size;
        let mut commands = Vec::new();
        assert!(push_icon_button_commands(
            &mut commands,
            &node,
            &rect,
            &rect,
            10,
            1.0,
        ));
        assert_eq!(commands.len(), 1);
        assert!(commands[0].image_pixels.is_some());
        assert_eq!(commands[0].frame.width, 16.0);
        assert_eq!(commands[0].frame.height, 16.0);
    }
}
