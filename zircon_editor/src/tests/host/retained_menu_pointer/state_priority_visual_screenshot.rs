use std::path::{Path, PathBuf};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData, TemplatePaneNodeData,
    TemplatePaneOptionData,
};

const STATE_PRIORITY_SCREENSHOT: &str = "editor-components-state-priority-900x360.png";
const STATE_PRIORITY_WIDTH: u32 = 900;
const STATE_PRIORITY_HEIGHT: u32 = 360;
const STATE_PRIORITY_BACKGROUND: [u8; 4] = [17, 20, 22, 255];

#[test]
fn state_priority_visual_paints_focus_without_promoting_hot_or_selected_surfaces() {
    let nodes = state_priority_nodes();
    assert!(
        nodes.iter().any(|node| {
            node.control_id.as_str() == "StatePriorityFocusedListRow"
                && node.focused
                && !node.selected
                && !node.checked
                && !node.hovered
        }),
        "focused-only list row fixture must stay separate from selected and hovered states"
    );
    assert!(
        nodes.iter().any(|node| {
            node.control_id.as_str() == "StatePriorityCompileFocused"
                && node.focused
                && !node.selected
                && !node.checked
                && !node.hovered
                && !node.pressed
        }),
        "focused command fixture must not impersonate active command state"
    );

    let bytes = state_priority_bytes_from_nodes(nodes);
    let row_panel = pixel_at(&bytes, 34, 94);

    assert_ne!(
        pixel_at(&bytes, 100, 122),
        row_panel,
        "focused-only list row should keep a visible focus border"
    );
    assert_eq!(
        pixel_at(&bytes, 188, 134),
        row_panel,
        "focused-only list row should not fill itself like a hovered or selected row"
    );
    assert_ne!(
        pixel_at(&bytes, 188, 168),
        row_panel,
        "hovered list row should still paint a real hover surface"
    );
    assert_ne!(
        pixel_at(&bytes, 188, 202),
        row_panel,
        "selected list row should still paint a marked selected surface"
    );

    let focused_popup_fill = pixel_at(&bytes, 414, 176);
    let selected_popup_fill = pixel_at(&bytes, 414, 134);
    let hovered_popup_fill = pixel_at(&bytes, 414, 204);
    assert_ne!(
        pixel_at(&bytes, 258, 172),
        focused_popup_fill,
        "focused-only popup row should keep a visible row outline"
    );
    assert_ne!(
        focused_popup_fill, selected_popup_fill,
        "focused-only popup row should not use the selected row fill"
    );
    assert_ne!(
        focused_popup_fill, hovered_popup_fill,
        "focused-only popup row should not use the hovered row fill"
    );

    let chip_panel = pixel_at(&bytes, 486, 94);
    assert_ne!(
        pixel_at(&bytes, 560, 122),
        chip_panel,
        "focused-only chip should keep a visible focus border"
    );
    assert_ne!(
        pixel_at(&bytes, 600, 170),
        chip_panel,
        "pressed chip should paint an active pressed surface"
    );
    assert_ne!(
        pixel_at(&bytes, 600, 224),
        pixel_at(&bytes, 600, 260),
        "focused command should stay visually quieter than hovered command"
    );

    let normal_chrome = pixel_at(&bytes, 704, 134);
    assert_eq!(
        pixel_at(&bytes, 794, 134),
        normal_chrome,
        "focused chrome panel should preserve the normal panel fill"
    );
    assert_ne!(
        pixel_at(&bytes, 794, 194),
        normal_chrome,
        "selected chrome panel should still paint the selected/open fill"
    );
}

#[test]
#[ignore = "writes local state-priority component screenshot artifact for visual review"]
fn capture_state_priority_component_visual_artifact() {
    let bytes = state_priority_bytes();
    let output_path = visual_layout_output_path(STATE_PRIORITY_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        STATE_PRIORITY_WIDTH,
        STATE_PRIORITY_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("state-priority component screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn state_priority_bytes() -> Vec<u8> {
    state_priority_bytes_from_nodes(state_priority_nodes())
}

fn state_priority_bytes_from_nodes(nodes: Vec<TemplatePaneNodeData>) -> Vec<u8> {
    paint_template_nodes_for_test_with_background(
        STATE_PRIORITY_WIDTH,
        STATE_PRIORITY_HEIGHT,
        STATE_PRIORITY_BACKGROUND,
        model_rc(nodes),
    )
}

fn state_priority_nodes() -> Vec<TemplatePaneNodeData> {
    vec![
        surface("StatePriorityRoot", "shell", 0.0, 0.0, 900.0, 360.0),
        label(
            "StatePriorityTitle",
            "State Priority",
            22.0,
            20.0,
            220.0,
            22.0,
            13.0,
            "",
        ),
        label(
            "StatePrioritySubtitle",
            "Focused-only rows, popups, chips, commands and chrome stay separate from hover/selected",
            22.0,
            42.0,
            760.0,
            18.0,
            10.0,
            "muted",
        ),
        surface("StatePriorityRowsPanel", "panel", 18.0, 78.0, 208.0, 214.0),
        surface("StatePriorityPopupPanel", "panel", 244.0, 78.0, 208.0, 214.0),
        surface("StatePriorityChipPanel", "panel", 470.0, 78.0, 194.0, 214.0),
        surface("StatePriorityChromePanel", "panel", 682.0, 78.0, 200.0, 214.0),
        label("StateRowsTitle", "Rows", 36.0, 96.0, 120.0, 18.0, 11.0, ""),
        label(
            "StatePopupTitle",
            "Popup Rows",
            262.0,
            96.0,
            120.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "StateChipTitle",
            "Chips / Commands",
            488.0,
            96.0,
            150.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "StateChromeTitle",
            "Chrome",
            700.0,
            96.0,
            120.0,
            18.0,
            11.0,
            "",
        ),
        list_row(
            "StatePriorityFocusedListRow",
            "Focused only",
            VisualState::Focused,
            38.0,
            122.0,
            168.0,
            28.0,
        ),
        list_row(
            "StatePriorityHoveredListRow",
            "Hovered",
            VisualState::Hovered,
            38.0,
            156.0,
            168.0,
            28.0,
        ),
        list_row(
            "StatePrioritySelectedListRow",
            "Selected",
            VisualState::Selected,
            38.0,
            190.0,
            168.0,
            28.0,
        ),
        popup_options(
            "StatePriorityPopupOptions",
            258.0,
            122.0,
            176.0,
            98.0,
            &[
                option("Selected Row", VisualState::Selected),
                option("Focused Row", VisualState::Focused),
                option("Hovered Row", VisualState::Hovered),
            ],
        ),
        chip(
            "WorkbenchStatePriorityFocusedChip",
            "Focused chip",
            VisualState::Focused,
            490.0,
            122.0,
            142.0,
            28.0,
        ),
        chip(
            "WorkbenchStatePriorityPressedChip",
            "Pressed chip",
            VisualState::Pressed,
            490.0,
            162.0,
            142.0,
            28.0,
        ),
        button(
            "StatePriorityCompileFocused",
            "Compile",
            VisualState::Focused,
            490.0,
            216.0,
            142.0,
            30.0,
        ),
        button(
            "StatePriorityCompileHovered",
            "Compile",
            VisualState::Hovered,
            490.0,
            252.0,
            142.0,
            30.0,
        ),
        shell_panel(
            "WorkbenchInspectorPanel",
            VisualState::Normal,
            700.0,
            122.0,
            72.0,
            42.0,
        ),
        shell_panel(
            "WorkbenchMainBandInspectorPanel",
            VisualState::Focused,
            790.0,
            122.0,
            72.0,
            42.0,
        ),
        shell_panel(
            "WorkbenchSceneTreePanel",
            VisualState::Selected,
            790.0,
            182.0,
            72.0,
            42.0,
        ),
        label(
            "StatePriorityRowsCopy",
            "focus border only; hot/selected fill remain distinct",
            36.0,
            242.0,
            170.0,
            30.0,
            10.0,
            "muted",
        ),
        label(
            "StatePriorityChromeCopy",
            "focused chrome keeps normal fill",
            700.0,
            242.0,
            150.0,
            30.0,
            10.0,
            "muted",
        ),
    ]
}

#[derive(Clone, Copy)]
enum VisualState {
    Normal,
    Focused,
    Hovered,
    Selected,
    Pressed,
}

fn list_row(
    control_id: &str,
    text: &str,
    state: VisualState,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    let selected = matches!(state, VisualState::Selected);
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "ListRow".into(),
        component_role: "list-row".into(),
        text: text.into(),
        selected,
        checked: selected,
        focused: matches!(state, VisualState::Focused),
        hovered: matches!(state, VisualState::Hovered),
        pressed: matches!(state, VisualState::Pressed),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn popup_options(
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    options: &[TemplatePaneOptionData],
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "DropdownPopup".into(),
        component_role: "dropdown-popup".into(),
        popup_open: true,
        structured_options: model_rc(options.to_vec()),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn option(label: &str, state: VisualState) -> TemplatePaneOptionData {
    TemplatePaneOptionData {
        id: label.into(),
        label: label.into(),
        selected: matches!(state, VisualState::Selected),
        focused: matches!(state, VisualState::Focused),
        hovered: matches!(state, VisualState::Hovered),
        ..TemplatePaneOptionData::default()
    }
}

fn chip(
    control_id: &str,
    text: &str,
    state: VisualState,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    let selected = matches!(state, VisualState::Selected);
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Label".into(),
        component_role: "chip".into(),
        text: text.into(),
        selected,
        checked: selected,
        focused: matches!(state, VisualState::Focused),
        hovered: matches!(state, VisualState::Hovered),
        pressed: matches!(state, VisualState::Pressed),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn button(
    control_id: &str,
    text: &str,
    state: VisualState,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    let selected = matches!(state, VisualState::Selected);
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Button".into(),
        component_role: "button".into(),
        text: text.into(),
        button_variant: "outlined".into(),
        action_id: "workbench.toolbar.compile".into(),
        selected,
        checked: selected,
        focused: matches!(state, VisualState::Focused),
        hovered: matches!(state, VisualState::Hovered),
        pressed: matches!(state, VisualState::Pressed),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn shell_panel(
    control_id: &str,
    state: VisualState,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    let selected = matches!(state, VisualState::Selected);
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Panel".into(),
        selected,
        checked: selected,
        focused: matches!(state, VisualState::Focused),
        hovered: matches!(state, VisualState::Hovered),
        pressed: matches!(state, VisualState::Pressed),
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
        control_id: control_id.into(),
        role: "Panel".into(),
        surface_variant: variant.into(),
        border_width: 1.0,
        corner_radius: 6.0,
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
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
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        font_size,
        text_tone: tone.into(),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> TemplateNodeFrameData {
    TemplateNodeFrameData {
        x,
        y,
        width,
        height,
    }
}

fn pixel_at(bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
    let index = ((y as usize * STATE_PRIORITY_WIDTH as usize) + x as usize) * 4;
    [
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ]
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
