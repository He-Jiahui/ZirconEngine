use std::path::{Path, PathBuf};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::primitives::SharedString;
use crate::ui::retained_host::{
    apply_host_appearance_from_tokens, paint_template_nodes_for_test_with_background,
    TemplateNodeFrameData, TemplatePaneNodeData,
};
use zircon_runtime_interface::ui::design_tokens::{
    EditorDesignTokens, EditorFontSmoothing, EditorUtilityTabTextRole,
};
use zircon_runtime_interface::ui::style::UiRgbaColor;

const GLOBAL_APPEARANCE_COMPONENTS_SCREENSHOT: &str =
    "editor-components-global-appearance-preferences-900x360.png";

#[test]
#[ignore = "writes local component screenshot artifact for global appearance preference review"]
fn capture_global_appearance_preferences_component_visual_artifact() {
    let tokens = appearance_review_tokens();
    let _appearance = ScopedHostAppearance::install(&tokens);
    let width = 900;
    let height = 360;
    let bytes = paint_template_nodes_for_test_with_background(
        width,
        height,
        tokens.palette.surface[0].to_u8(),
        model_rc(appearance_component_nodes()),
    );
    let output_path = visual_layout_output_path(GLOBAL_APPEARANCE_COMPONENTS_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        width,
        height,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("global appearance component screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

struct ScopedHostAppearance;

impl ScopedHostAppearance {
    fn install(tokens: &EditorDesignTokens) -> Self {
        apply_host_appearance_from_tokens(tokens);
        Self
    }
}

impl Drop for ScopedHostAppearance {
    fn drop(&mut self) {
        apply_host_appearance_from_tokens(&EditorDesignTokens::workbench_dark());
    }
}

fn appearance_review_tokens() -> EditorDesignTokens {
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.typography.ui_family = "monospace".to_string();
    tokens.typography.ui_strong_family = "monospace".to_string();
    tokens.typography.code_family = "monospace".to_string();
    tokens.typography.utility_tab_text_role = EditorUtilityTabTextRole::Code;
    tokens.typography.font_smoothing = EditorFontSmoothing::Subpixel;
    tokens.typography.body_size = 12.0;
    tokens.typography.caption_size = 10.0;
    tokens.typography.title_size = 16.0;
    tokens.typography.body_weight = 430;
    tokens.typography.strong_weight = 620;
    tokens.typography.code_weight = 450;
    tokens.typography.line_height = 1.25;

    tokens.palette.surface = [
        UiRgbaColor::from_u8(12, 15, 18, 255),
        UiRgbaColor::from_u8(18, 22, 26, 255),
        UiRgbaColor::from_u8(24, 29, 34, 255),
        UiRgbaColor::from_u8(36, 43, 49, 255),
    ];
    tokens.palette.surface_recessed = UiRgbaColor::from_u8(9, 12, 15, 255);
    tokens.palette.surface_hover = UiRgbaColor::from_u8(42, 50, 57, 255);
    tokens.palette.surface_selected = UiRgbaColor::from_u8(24, 65, 75, 255);
    tokens.palette.surface_disabled = UiRgbaColor::from_u8(31, 36, 40, 255);
    tokens.palette.accent = UiRgbaColor::from_u8(58, 191, 207, 255);
    tokens.palette.accent_soft = UiRgbaColor::from_u8(21, 70, 78, 255);
    tokens.palette.border = UiRgbaColor::from_u8(58, 67, 75, 255);
    tokens.palette.border_disabled = UiRgbaColor::from_u8(44, 50, 55, 255);
    tokens.palette.text_primary = UiRgbaColor::from_u8(235, 239, 241, 255);
    tokens.palette.text_secondary = UiRgbaColor::from_u8(170, 181, 188, 255);
    tokens.palette.text_disabled = UiRgbaColor::from_u8(99, 109, 116, 255);
    tokens.palette.popup = UiRgbaColor::from_u8(16, 19, 22, 255);
    tokens.palette.focus_ring = tokens.palette.accent;
    tokens.palette.track = UiRgbaColor::from_u8(45, 54, 61, 255);
    tokens.palette.shadow = UiRgbaColor::from_u8(0, 0, 0, 118);

    tokens.controls.default_height = 34.0;
    tokens.controls.compact_height = 30.0;
    tokens.controls.dense_height = 28.0;
    tokens.controls.small_radius = 3.0;
    tokens.controls.control_radius = 4.0;
    tokens.controls.large_radius = 6.0;
    tokens.controls.panel_radius = 6.0;
    tokens.controls.border_width = 1.0;
    tokens.density.gap_small = 4.0;
    tokens.density.gap_medium = 8.0;
    tokens.density.gap_large = 12.0;
    tokens.density.row_height = 30.0;
    tokens
}

fn appearance_component_nodes() -> Vec<TemplatePaneNodeData> {
    vec![
        surface("AppearanceRoot", "shell", 0.0, 0.0, 900.0, 360.0),
        label(
            "AppearanceTitle",
            "Appearance Sample",
            22.0,
            18.0,
            260.0,
            24.0,
            16.0,
            "primary",
        ),
        label(
            "AppearanceGlyphProbe",
            "iiii WWWW 1234",
            592.0,
            22.0,
            280.0,
            20.0,
            12.0,
            "muted",
        ),
        surface("UtilityTabsPanel", "panel", 18.0, 58.0, 420.0, 76.0),
        button(
            "AssetBrowserPreviewTabButton",
            "Preview",
            "",
            30.0,
            78.0,
            84.0,
            34.0,
            "selected",
        ),
        button(
            "AssetBrowserReferencesTabButton",
            "References",
            "",
            124.0,
            78.0,
            116.0,
            34.0,
            "",
        ),
        button(
            "AssetBrowserMetadataTabButton",
            "Metadata",
            "",
            250.0,
            78.0,
            100.0,
            34.0,
            "hover",
        ),
        button(
            "AssetBrowserPluginsTabButton",
            "Plugins",
            "",
            360.0,
            78.0,
            70.0,
            34.0,
            "",
        ),
        surface("ButtonsPanel", "panel", 454.0, 58.0, 428.0, 76.0),
        button(
            "CommandSave",
            "Save All",
            "primary",
            474.0,
            80.0,
            112.0,
            32.0,
            "",
        ),
        button(
            "CommandSimulate",
            "Simulate",
            "",
            596.0,
            80.0,
            116.0,
            32.0,
            "hover",
        ),
        button(
            "CommandDisabled",
            "Disabled",
            "",
            722.0,
            80.0,
            118.0,
            32.0,
            "disabled",
        ),
        surface("InputPanel", "panel", 18.0, 150.0, 420.0, 86.0),
        field(
            "AssetBrowserSearchInput",
            "Search components",
            34.0,
            176.0,
            258.0,
            34.0,
            "focus",
        ),
        dropdown(
            "AssetBrowserKindFilter",
            "All Assets",
            304.0,
            176.0,
            112.0,
            34.0,
            "open",
        ),
        surface("RowsPanel", "panel", 454.0, 150.0, 428.0, 86.0),
        list_row(
            "PreferenceRowSelected",
            "theme/editor_tokens.zui",
            470.0,
            166.0,
            394.0,
            30.0,
            "selected",
        ),
        list_row(
            "PreferenceRowHover",
            "ui/editor/workbench_page_chrome.zui",
            470.0,
            198.0,
            394.0,
            30.0,
            "hover",
        ),
        surface("CompositePanel", "panel", 18.0, 252.0, 864.0, 76.0),
        segmented(
            "AppearanceDensitySegmented",
            &["Compact", "Default", "Expanded"],
            "Default",
            34.0,
            274.0,
            270.0,
            34.0,
        ),
        selection(
            "AppearanceThemeToggle",
            "Global theme",
            330.0,
            275.0,
            150.0,
            32.0,
            "checkbox",
            true,
        ),
        selection(
            "AppearanceGridToggle",
            "Snap",
            494.0,
            275.0,
            136.0,
            32.0,
            "toggle",
            true,
        ),
        list_row(
            "PreferenceStatusRow",
            "appearance.zui",
            646.0,
            275.0,
            218.0,
            30.0,
            "hover",
        ),
    ]
}

fn label(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    font_size: f32,
    tone: &str,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        font_size,
        text_tone: tone.into(),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn surface(
    control_id: &str,
    variant: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Panel".into(),
        surface_variant: variant.into(),
        border_width: 1.0,
        corner_radius: 6.0,
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn button(
    control_id: &str,
    text: &str,
    variant: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: &str,
) -> TemplatePaneNodeData {
    let mut node = TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Button".into(),
        component_role: "button".into(),
        text: text.into(),
        button_variant: variant.into(),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    };
    apply_node_state(&mut node, state);
    node
}

fn field(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: &str,
) -> TemplatePaneNodeData {
    let mut node = TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Input".into(),
        component_role: "text-input".into(),
        text: text.into(),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    };
    apply_node_state(&mut node, state);
    node
}

fn dropdown(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: &str,
) -> TemplatePaneNodeData {
    let mut node = TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Dropdown".into(),
        component_role: "dropdown".into(),
        text: text.into(),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    };
    if state == "open" {
        node.popup_open = true;
    }
    apply_node_state(&mut node, state);
    node
}

fn segmented(
    control_id: &str,
    options: &[&str],
    selected: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "SegmentedControl".into(),
        component_role: "segmented-control".into(),
        value_text: selected.into(),
        options: model_rc(
            options
                .iter()
                .map(|option| SharedString::from(*option))
                .collect::<Vec<_>>(),
        ),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn selection(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    family: &str,
    checked: bool,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "SelectionControl".into(),
        component_role: family.into(),
        text: text.into(),
        checked,
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn list_row(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: &str,
) -> TemplatePaneNodeData {
    let mut node = TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "ListRow".into(),
        component_role: "list-row".into(),
        text: text.into(),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    };
    apply_node_state(&mut node, state);
    node
}

fn apply_node_state(node: &mut TemplatePaneNodeData, state: &str) {
    match state {
        "selected" => node.selected = true,
        "hover" => node.hovered = true,
        "pressed" => node.pressed = true,
        "focus" => node.focused = true,
        "disabled" => node.disabled = true,
        _ => {}
    }
}

fn node_id(control_id: &str) -> SharedString {
    format!("{control_id}.node").into()
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> TemplateNodeFrameData {
    TemplateNodeFrameData {
        x,
        y,
        width,
        height,
    }
}

fn visual_layout_output_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should live under the repository root")
        .join("docs")
        .join("tests")
        .join("editor")
}

fn visual_layout_output_path(filename: &str) -> PathBuf {
    let output_dir = visual_layout_output_dir();
    std::fs::create_dir_all(&output_dir).expect("visual-layout output directory should exist");
    output_dir.join(filename)
}
