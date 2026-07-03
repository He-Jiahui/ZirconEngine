use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_text::{
    measure_runtime_text_width, measure_runtime_text_width_with_style,
};
use super::super::super::super::paint_theme::{
    HostTextPreferences, HostTextSmoothing, HostUtilityTabTextRole, PALETTE,
};
use super::super::super::style_selector::{OUTLINED_BORDER, OUTLINED_SURFACE, PRIMARY_SURFACE};
use super::super::super::template_nodes::{
    paint_template_nodes_for_test, push_template_node_commands,
};
use super::super::content::button_label_paint_style_with_preferences;
use super::support::{
    changed_pixel_count, pixel_at, positioned_button_node, resolved_button_style,
};
use crate::ui::layouts::common::model_rc;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

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

    assert_eq!(pixel_at(&bytes, 152, 24, 24), PRIMARY_SURFACE);
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

    assert_eq!(pixel_at(&bytes, 152, 24, 24), OUTLINED_SURFACE);
    assert_eq!(pixel_at(&bytes, 152, 72, 8), OUTLINED_BORDER);
    assert!(changed_pixel_count(&bytes, 152, 42, 16, 70, 18) > 0);
}

#[test]
fn disabled_workbench_button_uses_disabled_surface_and_text() {
    let mut node = positioned_button_node(
        "WorkbenchDisabledButton",
        "Disabled",
        "outlined",
        12.0,
        8.0,
        120.0,
        34.0,
    );
    node.disabled = true;
    let bytes = paint_template_nodes_for_test(152, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 152, 24, 24), PALETTE.surface_disabled);
    assert_eq!(pixel_at(&bytes, 152, 72, 8), PALETTE.border_disabled);
    assert!(changed_pixel_count(&bytes, 152, 45, 16, 62, 18) > 0);
}

#[test]
fn dropdown_workbench_button_paints_trailing_chevron() {
    let bytes = paint_template_nodes_for_test(
        152,
        48,
        model_rc(vec![positioned_button_node(
            "WorkbenchDropdownButton",
            "Dropdown",
            "outlined",
            12.0,
            8.0,
            120.0,
            34.0,
        )]),
    );

    assert!(changed_pixel_count(&bytes, 152, 106, 18, 16, 12) > 0);
}

#[test]
fn workbench_button_with_svg_icon_paints_asset_pixels_before_label() {
    let mut node = positioned_button_node(
        "WorkbenchModuleCompile",
        "Compile",
        "filled",
        12.0,
        8.0,
        84.0,
        30.0,
    );
    node.icon_name = "zircon_editor_shell/toolbar/compile.svg".into();
    let origin = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 128.0,
        height: 48.0,
    };
    let clip = origin.clone();
    let mut commands = Vec::new();

    push_template_node_commands(&mut commands, &node, &origin, &clip, None, 0);

    let icon = commands
        .iter()
        .find(|command| command.image_pixels.is_some())
        .expect("button icon should paint loaded SVG pixels");
    let text = commands
        .iter()
        .find(|command| command.text.as_deref() == Some("Compile"))
        .expect("button label should still paint");
    assert!(
        icon.frame.x < text.frame.x,
        "button SVG icon should be laid out before the label"
    );
    assert_eq!(icon.image_key, None);
}

#[test]
fn semantic_button_glyphs_prefer_shell_asset_pixels() {
    let cases = [
        ("WorkbenchAddComponent", "Add Component", "outlined"),
        ("WorkbenchButtonDelete", "", "outlined"),
        ("WorkbenchDropdownButton", "Dropdown", "outlined"),
    ];

    for (control_id, label, variant) in cases {
        let mut node = positioned_button_node(control_id, label, variant, 12.0, 8.0, 128.0, 30.0);
        if control_id.contains("Delete") {
            node.validation_level = "danger".into();
        }
        let origin = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 160.0,
            height: 48.0,
        };
        let clip = origin.clone();
        let mut commands = Vec::new();

        push_template_node_commands(&mut commands, &node, &origin, &clip, None, 0);

        let icon = commands
            .iter()
            .find_map(|command| command.image_pixels.as_ref())
            .unwrap_or_else(|| {
                panic!("{control_id} should paint its semantic glyph as SVG pixels")
            });
        assert_eq!(icon.width, 14);
        assert_eq!(icon.height, 14);
        assert!(
            !icon.resource_key.starts_with("missing-icon:"),
            "{control_id} should resolve a real shell SVG asset, got {}",
            icon.resource_key
        );
    }
}

#[test]
fn editor_variant_button_uses_centered_button_text_path() {
    let node = positioned_button_node(
        "OpenAssetBrowser",
        "Asset Browser",
        "secondary",
        12.0,
        8.0,
        128.0,
        20.0,
    );
    let origin = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 180.0,
        height: 48.0,
    };
    let clip = origin.clone();
    let mut commands = Vec::new();

    push_template_node_commands(&mut commands, &node, &origin, &clip, None, 0);

    let text_commands: Vec<_> = commands
        .iter()
        .filter(|command| command.text.as_deref() == Some("Asset Browser"))
        .collect();
    assert_eq!(text_commands.len(), 1);
    let text_x = text_commands[0].frame.x;
    assert!(
        text_commands[0].frame.width >= 58.0,
        "button text frame should not force Asset Browser onto a wrapped second line, got width={}",
        text_commands[0].frame.width
    );
    assert_eq!(text_commands[0].font_size, 10.0);
    assert_eq!(text_commands[0].line_height, 12.0);
    let expected_centered_x = 12.0 + (128.0 - text_commands[0].frame.width) * 0.5;
    assert!(
        (text_x - expected_centered_x).abs() <= 0.5,
        "expected centered button text, got x={text_x}, expected {expected_centered_x}"
    );
}

#[test]
fn dock_tab_button_measures_label_with_declared_font_size() {
    let mut node = positioned_button_node("DockTab0", "Asset Browser", "", 12.0, 8.0, 162.0, 30.0);
    node.font_size = 12.0;
    let origin = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 48.0,
    };
    let clip = origin.clone();
    let mut commands = Vec::new();

    push_template_node_commands(&mut commands, &node, &origin, &clip, None, 0);

    let text = commands
        .iter()
        .find(|command| command.text.as_deref() == Some("Asset Browser"))
        .expect("dock tab text command");
    let runtime_width = measure_runtime_text_width("Asset Browser", 12.0);
    assert_eq!(text.font_size, 12.0);
    assert_eq!(text.line_height, 14.0);
    assert_eq!(
        text.frame.y.fract(),
        0.0,
        "declared 12px dock-tab labels should land on an integer text slot"
    );
    assert!(
        text.frame.width >= runtime_width + 5.5,
        "dock tab text frame should measure the declared 12px font, got frame={}, runtime={runtime_width}",
        text.frame.width
    );
}

#[test]
fn button_label_uses_strong_text_style_when_node_font_weight_is_strong() {
    let mut node =
        positioned_button_node("WorkbenchStrongButton", "Strong", "", 12.0, 8.0, 84.0, 24.0);
    node.font_size = 12.0;
    node.font_weight = 600;
    let origin = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 120.0,
        height: 40.0,
    };
    let clip = origin.clone();
    let mut commands = Vec::new();

    push_template_node_commands(&mut commands, &node, &origin, &clip, None, 0);

    let text = commands
        .iter()
        .find(|command| command.text.as_deref() == Some("Strong"))
        .expect("button text command");
    assert!(text.text_style.strong);
    assert!(!text.text_style.emphasis);
    assert!(!text.text_style.code);
}

#[test]
fn asset_browser_utility_tab_label_uses_ui_text_style() {
    let mut node = positioned_button_node(
        "AssetBrowserPreviewTabButton",
        "Preview",
        "",
        12.0,
        8.0,
        68.0,
        22.0,
    );
    node.action_id = "workbench.asset.utility_tab.set".into();
    node.font_size = 12.0;
    let origin = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 120.0,
        height: 40.0,
    };
    let clip = origin.clone();
    let mut commands = Vec::new();

    push_template_node_commands(&mut commands, &node, &origin, &clip, None, 0);

    let text = commands
        .iter()
        .find(|command| command.text.as_deref() == Some("Preview"))
        .expect("utility tab text command");
    let ui_width = measure_runtime_text_width("Preview", 12.0);
    let mono_width = measure_runtime_text_width_with_style(
        "Preview",
        12.0,
        UiTextRunPaintStyle {
            code: true,
            ..UiTextRunPaintStyle::default()
        },
    );
    assert!(!text.text_style.code);
    assert!(!text.text_style.emphasis);
    assert!(
        text.frame.width >= ui_width + 5.5,
        "utility tab text frame should measure with the UI text preference, got frame={}, ui={ui_width}",
        text.frame.width
    );
    assert_ne!(
        ui_width.round(),
        mono_width.round(),
        "test fixture must distinguish UI text preference measurement from code/mono measurement"
    );
}

#[test]
fn asset_browser_utility_tab_label_role_can_switch_to_code_text_preference() {
    let mut utility_tab = positioned_button_node(
        "AssetBrowserPreviewTabButton",
        "Preview",
        "",
        12.0,
        8.0,
        68.0,
        22.0,
    );
    utility_tab.action_id = "workbench.asset.utility_tab.set".into();
    let ordinary_button = positioned_button_node(
        "WorkbenchSecondaryButton",
        "Preview",
        "",
        12.0,
        8.0,
        68.0,
        22.0,
    );
    let preferences = HostTextPreferences {
        ui_family: "ui-family".to_string(),
        ui_strong_family: "ui-strong-family".to_string(),
        code_family: "code-family".to_string(),
        utility_tab_text_role: HostUtilityTabTextRole::Code,
        smoothing: HostTextSmoothing::Grayscale,
        ui_weight: 400,
        strong_weight: 600,
        code_weight: 400,
    };

    let utility_style = button_label_paint_style_with_preferences(&utility_tab, &preferences);
    let ordinary_style = button_label_paint_style_with_preferences(&ordinary_button, &preferences);

    assert!(utility_style.code);
    assert!(!utility_style.emphasis);
    assert!(!ordinary_style.code);
}

#[test]
fn button_label_text_slot_snaps_utility_tab_line_height_to_pixels() {
    let mut node = positioned_button_node(
        "AssetBrowserPreviewTabButton",
        "Preview",
        "",
        12.0,
        8.0,
        68.0,
        22.0,
    );
    node.font_size = 12.0;
    let origin = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 120.0,
        height: 40.0,
    };
    let clip = origin.clone();
    let mut commands = Vec::new();

    push_template_node_commands(&mut commands, &node, &origin, &clip, None, 0);

    let text = commands
        .iter()
        .find(|command| command.text.as_deref() == Some("Preview"))
        .expect("utility tab text command");
    assert_eq!(text.font_size, 12.0);
    assert_eq!(text.line_height, 14.0);
    assert_eq!(text.frame.height, 14.0);
    assert_eq!(text.frame.y, 12.0);
}

#[test]
fn button_label_frame_keeps_raster_guard_for_short_actions() {
    let node = positioned_button_node(
        "AssetBrowserImportButton",
        "Import",
        "secondary",
        12.0,
        8.0,
        96.0,
        24.0,
    );
    let origin = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 128.0,
        height: 40.0,
    };
    let clip = origin.clone();
    let mut commands = Vec::new();

    push_template_node_commands(&mut commands, &node, &origin, &clip, None, 0);

    let text = commands
        .iter()
        .find(|command| command.text.as_deref() == Some("Import"))
        .expect("Import button text command");
    let runtime_width = measure_runtime_text_width("Import", text.font_size);
    assert!(
        text.frame.width >= runtime_width + 5.5,
        "button text frame should retain raster guard beyond runtime width, got frame={}, runtime={runtime_width}",
        text.frame.width
    );
}

#[test]
fn pressed_editor_variant_button_shifts_content_like_slate_pressed_padding() {
    let normal = positioned_button_node(
        "OpenAssetBrowser",
        "Asset Browser",
        "secondary",
        12.0,
        8.0,
        128.0,
        20.0,
    );
    let mut pressed = normal.clone();
    pressed.pressed = true;
    let origin = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 180.0,
        height: 48.0,
    };
    let clip = origin.clone();
    let mut normal_commands = Vec::new();
    let mut pressed_commands = Vec::new();

    push_template_node_commands(&mut normal_commands, &normal, &origin, &clip, None, 0);
    push_template_node_commands(&mut pressed_commands, &pressed, &origin, &clip, None, 0);

    let normal_text = normal_commands
        .iter()
        .find(|command| command.text.as_deref() == Some("Asset Browser"))
        .expect("normal button text command");
    let pressed_text = pressed_commands
        .iter()
        .find(|command| command.text.as_deref() == Some("Asset Browser"))
        .expect("pressed button text command");

    assert_eq!(pressed_text.frame.y, normal_text.frame.y + 1.0);
}

#[test]
fn selected_asset_browser_toolbar_chip_paints_segment_without_tab_underline() {
    let mut node =
        positioned_button_node("AssetBrowserKindAllChip", "All", "", 12.0, 8.0, 72.0, 24.0);
    node.selected = true;
    node.focused = true;
    node.action_id = "workbench.asset.kind_filter.set".into();

    let bytes = paint_template_nodes_for_test(112, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 112, 48, 10), PALETTE.surface);
    assert_ne!(pixel_at(&bytes, 112, 16, 30), PALETTE.accent);
    assert_ne!(pixel_at(&bytes, 112, 48, 30), PALETTE.accent);
    assert_ne!(pixel_at(&bytes, 112, 48, 8), PALETTE.focus_ring);
}

#[test]
fn inactive_asset_browser_toolbar_chip_paints_without_tile_surface() {
    let mut node = positioned_button_node(
        "AssetBrowserKindTextureButton",
        "Texture",
        "",
        12.0,
        8.0,
        78.0,
        24.0,
    );
    node.action_id = "workbench.asset.kind_filter.set".into();

    let bytes = paint_template_nodes_for_test(118, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 118, 48, 10), [0, 0, 0, 255]);
    assert_ne!(pixel_at(&bytes, 118, 48, 30), PALETTE.accent);
    assert_ne!(pixel_at(&bytes, 118, 48, 10), PALETTE.surface_pressed);
}

#[test]
fn selected_asset_browser_utility_tab_still_paints_slate_indicator() {
    let mut node = positioned_button_node(
        "AssetBrowserPreviewTabButton",
        "Preview",
        "",
        12.0,
        8.0,
        72.0,
        24.0,
    );
    node.selected = true;
    node.focused = true;
    node.action_id = "workbench.asset.utility_tab.set".into();

    let bytes = paint_template_nodes_for_test(112, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 112, 48, 10), PALETTE.surface_pressed);
    assert_ne!(pixel_at(&bytes, 112, 16, 30), PALETTE.accent);
    assert_eq!(pixel_at(&bytes, 112, 48, 30), PALETTE.accent);
    assert_ne!(pixel_at(&bytes, 112, 48, 8), PALETTE.focus_ring);
}

#[test]
fn selected_page_tab_button_paints_slate_indicator_without_focus_frame() {
    let mut node = positioned_button_node("PageTab0", "Effect", "ghost", 12.0, 8.0, 72.0, 28.0);
    node.selected = true;
    node.focused = true;

    let bytes = paint_template_nodes_for_test(112, 52, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 112, 48, 12), PALETTE.surface_hover);
    assert_eq!(pixel_at(&bytes, 112, 48, 34), PALETTE.accent);
    assert_ne!(pixel_at(&bytes, 112, 48, 8), PALETTE.focus_ring);
}

#[test]
fn selected_dock_tab_button_paints_slate_indicator_without_focus_frame() {
    let mut node = positioned_button_node("DockTab1", "Effect", "", 12.0, 8.0, 72.0, 28.0);
    node.selected = true;
    node.focused = true;

    let bytes = paint_template_nodes_for_test(112, 52, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 112, 48, 12), PALETTE.surface_hover);
    assert_eq!(pixel_at(&bytes, 112, 48, 34), PALETTE.accent);
    assert_ne!(pixel_at(&bytes, 112, 48, 8), PALETTE.focus_ring);
}

#[test]
fn selected_workbench_module_tab_paints_slate_indicator_without_focus_frame() {
    let mut node = positioned_button_node(
        "WorkbenchModuleEffect",
        "Effect",
        "tab",
        12.0,
        8.0,
        68.0,
        34.0,
    );
    node.selected = true;
    node.checked = true;
    node.focused = true;
    node.action_id = "workbench.module.effect".into();

    let bytes = paint_template_nodes_for_test(112, 56, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 112, 48, 12), PALETTE.surface_hover);
    assert_eq!(pixel_at(&bytes, 112, 48, 40), PALETTE.accent);
    assert_ne!(pixel_at(&bytes, 112, 48, 8), PALETTE.focus_ring);
}

#[test]
fn prominent_workbench_command_button_paints_muted_surface_instead_of_accent_fill() {
    let mut node = positioned_button_node(
        "WorkbenchModuleCompile",
        "Compile",
        "filled",
        12.0,
        8.0,
        84.0,
        34.0,
    );
    node.selected = true;
    node.checked = true;
    node.action_id = "workbench.module.compile".into();
    node.button_style = resolved_button_style(
        PALETTE.accent,
        PALETTE.focus_ring,
        PALETTE.shell_background,
        1.0,
    );

    let bytes = paint_template_nodes_for_test(124, 56, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 124, 20, 24), PALETTE.surface_hover);
    assert_eq!(pixel_at(&bytes, 124, 48, 8), PALETTE.border);
    assert_ne!(pixel_at(&bytes, 124, 20, 24), PALETTE.accent);
}

#[test]
fn asset_import_command_button_paints_primary_accent_fill() {
    let mut node =
        positioned_button_node("ImportModel", "Import", "primary", 12.0, 8.0, 96.0, 26.0);
    node.action_id = "workbench.asset.import_model".into();
    node.button_style = resolved_button_style(
        PALETTE.accent,
        PALETTE.focus_ring,
        PALETTE.shell_background,
        1.0,
    );

    let bytes = paint_template_nodes_for_test(132, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 132, 20, 20), PALETTE.accent);
    assert_eq!(pixel_at(&bytes, 132, 48, 8), PALETTE.accent);
}
