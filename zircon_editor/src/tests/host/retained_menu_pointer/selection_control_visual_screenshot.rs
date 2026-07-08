use std::path::{Path, PathBuf};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData, TemplatePaneNodeData,
};

const SELECTION_COMPONENT_SCREENSHOT: &str = "editor-components-selection-controls-900x360.png";
const SELECTION_ATLAS_WIDTH: u32 = 900;
const SELECTION_ATLAS_HEIGHT: u32 = 360;
const SELECTION_ATLAS_BACKGROUND: [u8; 4] = [17, 20, 22, 255];

#[test]
fn selection_control_component_visual_paints_checkbox_radio_toggle_focus_and_disabled() {
    let bytes = selection_component_bytes();

    let checkbox_panel = pixel_at(&bytes, 30, 92);
    assert!(
        distinct_pixel_count(
            &bytes,
            50,
            138,
            24,
            24,
            &[SELECTION_ATLAS_BACKGROUND, checkbox_panel],
        ) > 0,
        "checked checkbox should paint visible mark and tick pixels"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            80,
            140,
            98,
            20,
            &[SELECTION_ATLAS_BACKGROUND, checkbox_panel],
        ) > 0,
        "checkbox should paint retained label text"
    );

    let radio_panel = pixel_at(&bytes, 242, 92);
    assert!(
        distinct_pixel_count(
            &bytes,
            262,
            194,
            24,
            24,
            &[SELECTION_ATLAS_BACKGROUND, radio_panel],
        ) > 0,
        "checked radio should paint ring and center dot"
    );

    let toggle_panel = pixel_at(&bytes, 454, 92);
    assert!(
        distinct_pixel_count(
            &bytes,
            542,
            137,
            52,
            26,
            &[SELECTION_ATLAS_BACKGROUND, toggle_panel],
        ) > 0,
        "enabled toggle should paint track and thumb"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            542,
            193,
            52,
            26,
            &[SELECTION_ATLAS_BACKGROUND, toggle_panel],
        ) > 0,
        "checked toggle should paint active track and shifted thumb"
    );

    let focused_panel = pixel_at(&bytes, 666, 92);
    assert_ne!(
        pixel_at(&bytes, 690, 144),
        focused_panel,
        "focused checkbox should paint a visible focus border"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            696,
            195,
            124,
            20,
            &[SELECTION_ATLAS_BACKGROUND, focused_panel],
        ) > 0,
        "disabled selection control should still paint muted label pixels"
    );
}

#[test]
#[ignore = "writes local selection-control component screenshot artifact for visual review"]
fn capture_selection_control_component_visual_artifact() {
    let bytes = selection_component_bytes();
    let output_path = visual_layout_output_path(SELECTION_COMPONENT_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        SELECTION_ATLAS_WIDTH,
        SELECTION_ATLAS_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("selection-control component screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn selection_component_bytes() -> Vec<u8> {
    paint_template_nodes_for_test_with_background(
        SELECTION_ATLAS_WIDTH,
        SELECTION_ATLAS_HEIGHT,
        SELECTION_ATLAS_BACKGROUND,
        model_rc(selection_component_nodes()),
    )
}

fn selection_component_nodes() -> Vec<TemplatePaneNodeData> {
    vec![
        surface("SelectionRoot", "shell", 0.0, 0.0, 900.0, 360.0),
        label(
            "SelectionTitle",
            "Selection Controls",
            22.0,
            20.0,
            260.0,
            22.0,
            13.0,
            "",
        ),
        label(
            "SelectionSubtitle",
            "Checkbox, radio and toggle states use retained selection-control painters",
            22.0,
            42.0,
            610.0,
            18.0,
            10.0,
            "muted",
        ),
        surface("SelectionCheckboxPanel", "panel", 18.0, 78.0, 188.0, 214.0),
        surface("SelectionRadioPanel", "panel", 230.0, 78.0, 188.0, 214.0),
        surface("SelectionTogglePanel", "panel", 442.0, 78.0, 188.0, 214.0),
        surface("SelectionStatePanel", "inset", 654.0, 78.0, 228.0, 214.0),
        label(
            "SelectionCheckboxTitle",
            "Checkbox",
            36.0,
            96.0,
            140.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "SelectionRadioTitle",
            "Radio",
            248.0,
            96.0,
            140.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "SelectionToggleTitle",
            "Toggle",
            460.0,
            96.0,
            140.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "SelectionStateTitle",
            "Focused / Disabled",
            672.0,
            96.0,
            160.0,
            18.0,
            11.0,
            "",
        ),
        selection_control(
            SelectionKind::Checkbox,
            "WorkbenchCheckboxOn",
            "Use bounds",
            42.0,
            130.0,
            148.0,
            32.0,
            SelectionState::Checked,
        ),
        selection_control(
            SelectionKind::Checkbox,
            "WorkbenchCheckboxOff",
            "Occlusion",
            42.0,
            186.0,
            148.0,
            32.0,
            SelectionState::Normal,
        ),
        selection_control(
            SelectionKind::Radio,
            "WorkbenchRadioOff",
            "Local",
            254.0,
            130.0,
            148.0,
            32.0,
            SelectionState::Normal,
        ),
        selection_control(
            SelectionKind::Radio,
            "WorkbenchRadioOn",
            "World",
            254.0,
            186.0,
            148.0,
            32.0,
            SelectionState::Checked,
        ),
        selection_control(
            SelectionKind::Toggle,
            "WorkbenchToggleOff",
            "Realtime",
            462.0,
            130.0,
            138.0,
            32.0,
            SelectionState::Normal,
        ),
        selection_control(
            SelectionKind::Toggle,
            "WorkbenchToggleOn",
            "Snap",
            462.0,
            186.0,
            138.0,
            32.0,
            SelectionState::CheckedHovered,
        ),
        selection_control(
            SelectionKind::Checkbox,
            "WorkbenchCheckboxFocused",
            "Focus ring",
            678.0,
            130.0,
            160.0,
            32.0,
            SelectionState::Focused,
        ),
        selection_control(
            SelectionKind::Toggle,
            "WorkbenchToggleDisabled",
            "Disabled",
            678.0,
            186.0,
            160.0,
            32.0,
            SelectionState::Disabled,
        ),
        label(
            "SelectionStateCopy",
            "Focus is border only; disabled is muted",
            672.0,
            232.0,
            196.0,
            18.0,
            10.0,
            "muted",
        ),
    ]
}

#[derive(Clone, Copy)]
enum SelectionKind {
    Checkbox,
    Radio,
    Toggle,
}

#[derive(Clone, Copy)]
enum SelectionState {
    Normal,
    Checked,
    CheckedHovered,
    Focused,
    Disabled,
}

fn selection_control(
    kind: SelectionKind,
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: SelectionState,
) -> TemplatePaneNodeData {
    let (role, component_role) = match kind {
        SelectionKind::Checkbox => ("Checkbox", "checkbox"),
        SelectionKind::Radio => ("Radio", "radio"),
        SelectionKind::Toggle => ("Toggle", "toggle"),
    };
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: role.into(),
        component_role: component_role.into(),
        text: text.into(),
        component_variant: "workbench-selection-control".into(),
        checked: matches!(
            state,
            SelectionState::Checked | SelectionState::CheckedHovered
        ),
        selected: matches!(
            state,
            SelectionState::Checked | SelectionState::CheckedHovered
        ),
        hovered: matches!(state, SelectionState::CheckedHovered),
        focused: matches!(state, SelectionState::Focused),
        disabled: matches!(state, SelectionState::Disabled),
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
    let index = ((y as usize * SELECTION_ATLAS_WIDTH as usize) + x as usize) * 4;
    [
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ]
}

fn distinct_pixel_count(
    bytes: &[u8],
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    excluded_colors: &[[u8; 4]],
) -> usize {
    let mut changed = 0;
    for py in y..(y + height) {
        for px in x..(x + width) {
            let index = ((py as usize * SELECTION_ATLAS_WIDTH as usize) + px as usize) * 4;
            let color = [
                bytes[index],
                bytes[index + 1],
                bytes[index + 2],
                bytes[index + 3],
            ];
            if !excluded_colors.contains(&color) {
                changed += 1;
            }
        }
    }
    changed
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
