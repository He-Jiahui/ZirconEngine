use std::path::{Path, PathBuf};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData, TemplatePaneNodeData,
};
use zircon_runtime_interface::ui::{
    design_tokens::EditorPaletteTokens,
    style::{UiRgbaColor, UiStyleColor},
};

const FIELD_COMPONENT_SCREENSHOT: &str = "editor-components-fields-900x360.png";
const FIELD_ATLAS_WIDTH: u32 = 900;
const FIELD_ATLAS_HEIGHT: u32 = 360;
const FIELD_ATLAS_BACKGROUND: [u8; 4] = [17, 20, 22, 255];

#[test]
fn field_component_visual_paints_input_search_stepper_focus_and_disabled() {
    let bytes = field_component_bytes();

    let input_surface = pixel_at(&bytes, 52, 148);
    assert!(
        distinct_pixel_count(
            &bytes,
            74,
            140,
            104,
            20,
            &[FIELD_ATLAS_BACKGROUND, input_surface],
        ) > 0,
        "ordinary input should paint readable value text above recessed surface"
    );

    let search_surface = pixel_at(&bytes, 262, 148);
    assert!(
        distinct_pixel_count(
            &bytes,
            276,
            140,
            20,
            20,
            &[FIELD_ATLAS_BACKGROUND, search_surface],
        ) > 0,
        "search field should paint its leading shell search icon"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            314,
            140,
            72,
            20,
            &[FIELD_ATLAS_BACKGROUND, search_surface],
        ) > 0,
        "search field should paint placeholder text after icon inset"
    );
    assert_eq!(
        pixel_at(&bytes, 300, 186),
        EditorPaletteTokens::WORKBENCH_FOCUS_RING,
        "focused search should share the editable-text primary focus outline"
    );

    let stepper_surface = pixel_at(&bytes, 474, 148);
    assert!(
        distinct_pixel_count(
            &bytes,
            582,
            138,
            18,
            24,
            &[FIELD_ATLAS_BACKGROUND, stepper_surface],
        ) > 0,
        "stepper field should paint divider and up/down arrows"
    );

    let focused_border = pixel_at(&bytes, 728, 130);
    assert_eq!(
        focused_border,
        EditorPaletteTokens::WORKBENCH_FOCUS_RING,
        "focused field should paint the shared Starship primary focus outline"
    );
    assert_eq!(
        pixel_at(&bytes, 682, 148),
        EditorPaletteTokens::WORKBENCH_SURFACE_RECESSED,
        "focused field should keep its recessed state surface when authored normal chrome exists"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            698,
            140,
            96,
            20,
            &[FIELD_ATLAS_BACKGROUND, pixel_at(&bytes, 682, 148)],
        ) > 0,
        "focused field should paint value text through retained text path"
    );

    let disabled_surface = pixel_at(&bytes, 682, 204);
    assert!(
        distinct_pixel_count(
            &bytes,
            700,
            196,
            100,
            20,
            &[FIELD_ATLAS_BACKGROUND, disabled_surface],
        ) > 0,
        "disabled field should paint muted placeholder label"
    );
}

#[test]
#[ignore = "writes local field/search component screenshot artifact for visual review"]
fn capture_field_component_visual_artifact() {
    let bytes = field_component_bytes();
    let output_path = visual_layout_output_path(FIELD_COMPONENT_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        FIELD_ATLAS_WIDTH,
        FIELD_ATLAS_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("field component screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn field_component_bytes() -> Vec<u8> {
    paint_template_nodes_for_test_with_background(
        FIELD_ATLAS_WIDTH,
        FIELD_ATLAS_HEIGHT,
        FIELD_ATLAS_BACKGROUND,
        model_rc(field_component_nodes()),
    )
}

fn field_component_nodes() -> Vec<TemplatePaneNodeData> {
    vec![
        surface("FieldRoot", "shell", 0.0, 0.0, 900.0, 360.0),
        label("FieldTitle", "Fields", 22.0, 20.0, 220.0, 22.0, 13.0, ""),
        label(
            "FieldSubtitle",
            "Input, search, stepper, focused and disabled states use retained text-field painters",
            22.0,
            42.0,
            650.0,
            18.0,
            10.0,
            "muted",
        ),
        surface("FieldInputPanel", "panel", 18.0, 78.0, 188.0, 214.0),
        surface("FieldSearchPanel", "panel", 230.0, 78.0, 188.0, 214.0),
        surface("FieldStepperPanel", "panel", 442.0, 78.0, 188.0, 214.0),
        surface("FieldStatePanel", "inset", 654.0, 78.0, 228.0, 214.0),
        label(
            "FieldInputTitle",
            "Text Input",
            36.0,
            96.0,
            130.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "FieldSearchTitle",
            "Search",
            248.0,
            96.0,
            120.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "FieldStepperTitle",
            "Stepper",
            460.0,
            96.0,
            120.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "FieldStateTitle",
            "Focused / Disabled",
            672.0,
            96.0,
            160.0,
            18.0,
            11.0,
            "",
        ),
        field(
            "WorkbenchInputText",
            "Directional Light",
            "",
            "input-field",
            42.0,
            130.0,
            138.0,
            32.0,
            FieldState::Normal,
        ),
        field(
            "AssetBrowserImportPathField",
            "",
            "Drop or paste asset source path",
            "input-field",
            42.0,
            186.0,
            138.0,
            32.0,
            FieldState::Normal,
        ),
        field(
            "SearchEdited",
            "",
            "Search Assets",
            "search-field",
            250.0,
            130.0,
            148.0,
            30.0,
            FieldState::Normal,
        ),
        field(
            "SearchEditedFocused",
            "material",
            "Search Assets",
            "search-field",
            250.0,
            186.0,
            148.0,
            30.0,
            FieldState::Focused,
        ),
        field(
            "WorkbenchInputStepper",
            "42",
            "",
            "input-field",
            462.0,
            130.0,
            138.0,
            32.0,
            FieldState::Normal,
        ),
        field(
            "WorkbenchInputStepper",
            "128",
            "",
            "input-field",
            462.0,
            186.0,
            138.0,
            32.0,
            FieldState::Hovered,
        ),
        field(
            "WorkbenchInputFocused",
            "Focus border",
            "",
            "input-field",
            678.0,
            130.0,
            150.0,
            32.0,
            FieldState::Focused,
        ),
        field(
            "WorkbenchInputDisabled",
            "",
            "",
            "input-field",
            678.0,
            186.0,
            150.0,
            32.0,
            FieldState::Disabled,
        ),
        label(
            "FieldStateCopy",
            "Teal focus; muted disabled label",
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
enum FieldState {
    Normal,
    Hovered,
    Focused,
    Disabled,
}

fn field(
    control_id: &str,
    value: &str,
    text: &str,
    component_role: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: FieldState,
) -> TemplatePaneNodeData {
    let mut node = TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "InputField".into(),
        component_role: component_role.into(),
        value_text: value.into(),
        text: text.into(),
        component_variant: "workbench-field".into(),
        hovered: matches!(state, FieldState::Hovered),
        focused: matches!(state, FieldState::Focused),
        disabled: matches!(state, FieldState::Disabled),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    };
    if control_id == "WorkbenchInputFocused" {
        node.button_style.element.background_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(81, 88, 94, 255)));
        node.button_style.element.border_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(109, 116, 122, 255)));
    }
    node
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
    let index = ((y as usize * FIELD_ATLAS_WIDTH as usize) + x as usize) * 4;
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
            let index = ((py as usize * FIELD_ATLAS_WIDTH as usize) + px as usize) * 4;
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
