use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::style_selector::{
    WORKBENCH_TEXT_FIELD_BORDER as FIELD_BORDER,
    WORKBENCH_TEXT_FIELD_DISABLED_SURFACE as FIELD_DISABLED_SURFACE,
    WORKBENCH_TEXT_FIELD_DISABLED_TEXT as FIELD_DISABLED_TEXT,
    WORKBENCH_TEXT_FIELD_FOCUSED_BORDER as FIELD_FOCUSED_BORDER,
    WORKBENCH_TEXT_FIELD_FOCUSED_SURFACE as FIELD_FOCUSED_SURFACE,
    WORKBENCH_TEXT_FIELD_PLACEHOLDER as FIELD_PLACEHOLDER,
    WORKBENCH_TEXT_FIELD_SURFACE as FIELD_SURFACE,
};
use super::super::geometry::field_paint_rect;
use super::super::search::{search_field_text_left, SEARCH_FIELD_MAX_HEIGHT};
use super::super::style::{field_opacity, field_style};
use super::support::positioned_field_node;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_theme::{METRICS, PALETTE};
use zircon_runtime_interface::ui::style::{UiPainterResolvedState, UiRgbaColor, UiStyleColor};

#[test]
fn focused_workbench_field_uses_declared_focus_border() {
    let mut node = positioned_field_node(
        "WorkbenchInputFocused",
        "Focused input",
        12.0,
        8.0,
        170.0,
        32.0,
    );
    node.focused = true;
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(27, 152, 160, 255)));

    assert_eq!(field_border(&node), [27, 152, 160, 255]);
}

#[test]
fn disabled_workbench_field_uses_declared_opacity() {
    let mut node = positioned_field_node("WorkbenchInputDisabled", "", 12.0, 8.0, 170.0, 32.0);
    node.disabled = true;
    node.button_style.element.opacity = 0.94;

    assert!((field_opacity(&node, 1.0) - 0.94).abs() < 0.001);
    assert!((field_opacity(&node, 0.5) - 0.47).abs() < 0.001);
}

#[test]
fn workbench_field_selector_uses_shared_text_field_state_priority() {
    let mut node =
        positioned_field_node("WorkbenchInputText", "Text field", 12.0, 8.0, 170.0, 32.0);
    node.hovered = true;
    node.focused = true;
    node.pressed = true;

    assert_eq!(field_visual_state(&node), UiPainterResolvedState::Pressed);
    assert_eq!(field_surface(&node), FIELD_FOCUSED_SURFACE);

    node.pressed = false;
    assert_eq!(field_visual_state(&node), UiPainterResolvedState::Focused);
    assert_eq!(field_border(&node), FIELD_FOCUSED_BORDER);
    assert_eq!(FIELD_FOCUSED_BORDER, PALETTE.border);
    assert_ne!(FIELD_FOCUSED_BORDER, PALETTE.accent);
    assert_ne!(FIELD_FOCUSED_BORDER, PALETTE.focus_ring);

    node.disabled = true;
    assert_eq!(field_visual_state(&node), UiPainterResolvedState::Disabled);
    assert_eq!(field_surface(&node), FIELD_DISABLED_SURFACE);
    assert_eq!(field_text_color(&node), FIELD_DISABLED_TEXT);

    let placeholder = positioned_field_node("WorkbenchInputDisabled", "", 12.0, 8.0, 170.0, 32.0);
    assert_eq!(field_text_color(&placeholder), FIELD_PLACEHOLDER);
}

#[test]
fn workbench_field_uses_slate_recessed_surface_neutral_focus_and_muted_placeholder() {
    let node = positioned_field_node("WorkbenchInputText", "Text field", 12.0, 8.0, 170.0, 32.0);
    assert_eq!(field_surface(&node), FIELD_SURFACE);
    assert_eq!(field_surface(&node), PALETTE.surface_inset);
    assert_ne!(field_surface(&node), PALETTE.surface_pressed);
    assert_eq!(field_border(&node), FIELD_BORDER);

    let mut focused = positioned_field_node(
        "WorkbenchInputFocused",
        "Focused input",
        12.0,
        8.0,
        170.0,
        32.0,
    );
    focused.focused = true;
    assert_eq!(field_surface(&focused), PALETTE.surface);
    assert_eq!(field_border(&focused), FIELD_FOCUSED_BORDER);
    assert_eq!(field_border(&focused), PALETTE.border);
    assert_ne!(field_border(&focused), PALETTE.accent);

    let mut placeholder = positioned_field_node("SearchEdited", "", 12.0, 8.0, 170.0, 32.0);
    placeholder.text = "Search".into();
    assert_eq!(field_text_color(&placeholder), FIELD_PLACEHOLDER);
    assert_eq!(field_text_color(&placeholder), PALETTE.text_muted);
}

#[test]
fn asset_import_path_field_uses_placeholder_tone_when_empty() {
    let mut placeholder =
        positioned_field_node("AssetBrowserImportPathField", "", 12.0, 8.0, 240.0, 28.0);
    placeholder.text = "Drop or paste asset source path".into();

    assert_eq!(field_text_color(&placeholder), FIELD_PLACEHOLDER);
    assert_eq!(field_text_color(&placeholder), PALETTE.text_muted);

    placeholder.value_text = "E:/Project/assets/mesh.fbx".into();
    assert_ne!(field_text_color(&placeholder), FIELD_PLACEHOLDER);
}

#[test]
fn search_field_uses_placeholder_tone_and_icon_text_inset() {
    let mut node = positioned_field_node("SearchEdited", "", 12.0, 8.0, 170.0, 28.0);
    node.text = "Search".into();

    assert_eq!(field_text_color(&node), FIELD_PLACEHOLDER);
    assert_eq!(
        search_field_text_left(&node),
        METRICS.input_pad[0] + 16.0 + METRICS.gap_s
    );

    let normal = positioned_field_node("WorkbenchInputText", "Search", 12.0, 8.0, 170.0, 28.0);
    assert_eq!(search_field_text_left(&normal), METRICS.input_pad[0]);
}

#[test]
fn search_field_paint_rect_clamps_tall_authored_frames_to_compact_control_height() {
    let search = positioned_field_node("SearchEdited", "workbench", 12.3, 8.2, 170.4, 44.0);

    let rect = field_paint_rect(
        &search,
        &FrameRect {
            x: search.frame.x,
            y: search.frame.y,
            width: search.frame.width,
            height: search.frame.height,
        },
    );

    assert_eq!(rect.height, SEARCH_FIELD_MAX_HEIGHT);
    assert_eq!(rect.y, 16.0);
    assert_eq!(rect.x, 12.0);
    assert_eq!(rect.width, 170.0);

    let normal = positioned_field_node("WorkbenchInputText", "workbench", 12.3, 8.2, 170.4, 44.0);
    let normal_rect = field_paint_rect(
        &normal,
        &FrameRect {
            x: normal.frame.x,
            y: normal.frame.y,
            width: normal.frame.width,
            height: normal.frame.height,
        },
    );
    assert_eq!(normal_rect.height, 44.0);
    assert_eq!(normal_rect.y, 8.0);
}

fn field_surface(node: &TemplatePaneNodeData) -> [u8; 4] {
    field_style(node).surface
}

fn field_border(node: &TemplatePaneNodeData) -> [u8; 4] {
    field_style(node).border
}

fn field_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    field_style(node).text
}

fn field_visual_state(node: &TemplatePaneNodeData) -> UiPainterResolvedState {
    field_style(node).state
}
