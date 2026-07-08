use std::path::{Path, PathBuf};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData, TemplatePaneNodeData,
    TemplatePaneOptionData,
};

const DROPDOWN_COMPONENT_SCREENSHOT: &str = "editor-components-dropdowns-900x360.png";
const DROPDOWN_ATLAS_WIDTH: u32 = 900;
const DROPDOWN_ATLAS_HEIGHT: u32 = 360;
const DROPDOWN_ATLAS_BACKGROUND: [u8; 4] = [17, 20, 22, 255];

#[test]
fn dropdown_component_visual_paints_value_placeholder_popup_focus_and_disabled() {
    let bytes = dropdown_component_bytes();

    let value_surface = pixel_at(&bytes, 52, 148);
    assert!(
        distinct_pixel_count(
            &bytes,
            58,
            140,
            82,
            20,
            &[DROPDOWN_ATLAS_BACKGROUND, value_surface],
        ) > 0,
        "ordinary dropdown should paint retained value text above recessed surface"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            154,
            139,
            18,
            22,
            &[DROPDOWN_ATLAS_BACKGROUND, value_surface],
        ) > 0,
        "ordinary dropdown should paint shell chevron asset"
    );

    let placeholder_surface = pixel_at(&bytes, 262, 148);
    assert!(
        distinct_pixel_count(
            &bytes,
            272,
            140,
            92,
            20,
            &[DROPDOWN_ATLAS_BACKGROUND, placeholder_surface],
        ) > 0,
        "placeholder dropdown should paint first option through muted placeholder text"
    );

    let open_surface = pixel_at(&bytes, 474, 146);
    let focused_border = pixel_at(&bytes, 462, 126);
    assert_ne!(
        focused_border, DROPDOWN_ATLAS_BACKGROUND,
        "open focused dropdown should paint a visible focus border"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            582,
            137,
            18,
            22,
            &[DROPDOWN_ATLAS_BACKGROUND, open_surface],
        ) > 0,
        "open focused dropdown should keep chevron above the trigger surface"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            466,
            166,
            130,
            84,
            &[DROPDOWN_ATLAS_BACKGROUND, open_surface],
        ) > 0,
        "open dropdown should paint its option popup rows"
    );

    let state_surface = pixel_at(&bytes, 682, 148);
    assert!(
        distinct_pixel_count(
            &bytes,
            698,
            140,
            94,
            20,
            &[DROPDOWN_ATLAS_BACKGROUND, state_surface],
        ) > 0,
        "focused dropdown should paint value text through retained text path"
    );

    let disabled_surface = pixel_at(&bytes, 682, 204);
    assert!(
        distinct_pixel_count(
            &bytes,
            698,
            196,
            100,
            20,
            &[DROPDOWN_ATLAS_BACKGROUND, disabled_surface],
        ) > 0,
        "disabled dropdown should paint muted label and chevron"
    );
}

#[test]
#[ignore = "writes local dropdown/select component screenshot artifact for visual review"]
fn capture_dropdown_component_visual_artifact() {
    let bytes = dropdown_component_bytes();
    let output_path = visual_layout_output_path(DROPDOWN_COMPONENT_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        DROPDOWN_ATLAS_WIDTH,
        DROPDOWN_ATLAS_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("dropdown component screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn dropdown_component_bytes() -> Vec<u8> {
    paint_template_nodes_for_test_with_background(
        DROPDOWN_ATLAS_WIDTH,
        DROPDOWN_ATLAS_HEIGHT,
        DROPDOWN_ATLAS_BACKGROUND,
        model_rc(dropdown_component_nodes()),
    )
}

fn dropdown_component_nodes() -> Vec<TemplatePaneNodeData> {
    vec![
        surface("DropdownRoot", "shell", 0.0, 0.0, 900.0, 360.0),
        label(
            "DropdownTitle",
            "Dropdowns",
            22.0,
            20.0,
            220.0,
            22.0,
            13.0,
            "",
        ),
        label(
            "DropdownSubtitle",
            "Select triggers, placeholder text, popup rows, focused and disabled states",
            22.0,
            42.0,
            650.0,
            18.0,
            10.0,
            "muted",
        ),
        surface("DropdownValuePanel", "panel", 18.0, 78.0, 188.0, 214.0),
        surface(
            "DropdownPlaceholderPanel",
            "panel",
            230.0,
            78.0,
            188.0,
            214.0,
        ),
        surface("DropdownPopupPanel", "panel", 442.0, 78.0, 188.0, 214.0),
        surface("DropdownStatePanel", "inset", 654.0, 78.0, 228.0, 214.0),
        label(
            "DropdownValueTitle",
            "Value",
            36.0,
            96.0,
            130.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "DropdownPlaceholderTitle",
            "Placeholder",
            248.0,
            96.0,
            130.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "DropdownPopupTitle",
            "Open Popup",
            460.0,
            96.0,
            130.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "DropdownStateTitle",
            "Focused / Disabled",
            672.0,
            96.0,
            160.0,
            18.0,
            11.0,
            "",
        ),
        dropdown(
            "WorkbenchInputDropdownQuality",
            "Lit",
            "",
            &[],
            false,
            false,
            false,
            false,
            42.0,
            130.0,
            138.0,
            32.0,
            &[],
        ),
        dropdown(
            "WorkbenchInputDropdownAsset",
            "SM_Chair",
            "",
            &[],
            true,
            false,
            false,
            false,
            42.0,
            186.0,
            138.0,
            32.0,
            &[],
        ),
        dropdown(
            "WorkbenchInputDropdownActor",
            "",
            "",
            &["Select Actor"],
            false,
            false,
            false,
            false,
            250.0,
            130.0,
            148.0,
            32.0,
            &[],
        ),
        dropdown(
            "WorkbenchInputDropdownViewport",
            "Perspective",
            "",
            &[],
            false,
            true,
            false,
            false,
            250.0,
            186.0,
            148.0,
            30.0,
            &[],
        ),
        dropdown(
            "WorkbenchInputDropdownOpen",
            "Lighting",
            "",
            &[],
            false,
            true,
            true,
            false,
            462.0,
            126.0,
            148.0,
            30.0,
            &[
                option("Lighting", true, false, false, false),
                option("Environment", false, true, false, false),
                option("Disabled", false, false, false, true),
            ],
        ),
        dropdown(
            "WorkbenchInputDropdownFocused",
            "Camera",
            "",
            &[],
            false,
            true,
            false,
            false,
            678.0,
            130.0,
            150.0,
            32.0,
            &[],
        ),
        dropdown(
            "WorkbenchInputDropdownDisabled",
            "No Asset",
            "",
            &[],
            false,
            false,
            false,
            true,
            678.0,
            186.0,
            150.0,
            32.0,
            &[],
        ),
        label(
            "DropdownStateCopy",
            "Neutral focus; muted disabled label",
            672.0,
            232.0,
            196.0,
            18.0,
            10.0,
            "muted",
        ),
    ]
}

fn dropdown(
    control_id: &str,
    value: &str,
    text: &str,
    placeholder_options: &[&str],
    hovered: bool,
    focused: bool,
    popup_open: bool,
    disabled: bool,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    structured_options: &[TemplatePaneOptionData],
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Dropdown".into(),
        component_role: "dropdown".into(),
        value_text: value.into(),
        text: text.into(),
        component_variant: "workbench-dropdown".into(),
        hovered,
        focused,
        popup_open,
        disabled,
        options: model_rc(
            placeholder_options
                .iter()
                .map(|option| (*option).into())
                .collect(),
        ),
        structured_options: model_rc(structured_options.to_vec()),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn option(
    label: &str,
    selected: bool,
    hovered: bool,
    special: bool,
    disabled: bool,
) -> TemplatePaneOptionData {
    TemplatePaneOptionData {
        id: label.into(),
        label: label.into(),
        selected,
        hovered,
        special,
        disabled,
        ..TemplatePaneOptionData::default()
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
    let index = ((y as usize * DROPDOWN_ATLAS_WIDTH as usize) + x as usize) * 4;
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
            let index = ((py as usize * DROPDOWN_ATLAS_WIDTH as usize) + px as usize) * 4;
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
