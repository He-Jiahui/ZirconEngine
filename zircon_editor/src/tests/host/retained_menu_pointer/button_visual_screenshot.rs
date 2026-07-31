use std::path::{Path, PathBuf};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData, TemplatePaneNodeData,
};
use zircon_runtime_interface::ui::{
    design_tokens::EditorPaletteTokens,
    style::{UiRgbaColor, UiStyleColor},
};

const BUTTON_COMPONENT_SCREENSHOT: &str = "editor-components-buttons-900x360.png";
const BUTTON_ATLAS_WIDTH: u32 = 900;
const BUTTON_ATLAS_HEIGHT: u32 = 360;
const BUTTON_ATLAS_BACKGROUND: [u8; 4] = [17, 20, 22, 255];

#[test]
fn button_component_visual_paints_text_icon_pressed_disabled_and_tabs() {
    let bytes = button_component_bytes();

    let filled_surface = pixel_at(&bytes, 48, 148);
    assert_eq!(
        filled_surface,
        EditorPaletteTokens::WORKBENCH_ACCENT,
        "filled primary button should use the centralized primary accent surface"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            82,
            142,
            76,
            18,
            &[BUTTON_ATLAS_BACKGROUND, filled_surface]
        ) > 0,
        "filled button should paint readable centered text above the surface"
    );

    let add_surface = pixel_at(&bytes, 256, 148);
    assert!(
        distinct_pixel_count(
            &bytes,
            274,
            140,
            20,
            20,
            &[BUTTON_ATLAS_BACKGROUND, add_surface]
        ) > 0,
        "Add Component button should paint its leading Icon16 glyph"
    );

    let dropdown_surface = pixel_at(&bytes, 256, 204);
    assert!(
        distinct_pixel_count(
            &bytes,
            368,
            198,
            18,
            18,
            &[BUTTON_ATLAS_BACKGROUND, dropdown_surface],
        ) > 0,
        "dropdown button should paint its trailing chevron"
    );

    let pressed_surface = pixel_at(&bytes, 468, 148);
    assert_eq!(
        pressed_surface,
        EditorPaletteTokens::WORKBENCH_SURFACE[2],
        "pressed secondary button should keep its state surface when authored normal chrome exists"
    );
    assert_ne!(
        pressed_surface,
        [81, 88, 94, 255],
        "pressed secondary button should not repaint the declared normal surface"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            496,
            144,
            74,
            22,
            &[BUTTON_ATLAS_BACKGROUND, pressed_surface],
        ) > 0,
        "pressed button should paint its lowered content region"
    );

    let disabled_surface = pixel_at(&bytes, 468, 204);
    assert!(
        distinct_pixel_count(
            &bytes,
            496,
            198,
            74,
            22,
            &[BUTTON_ATLAS_BACKGROUND, disabled_surface],
        ) > 0,
        "disabled button should still paint muted label pixels"
    );

    let tab_surface = pixel_at(&bytes, 676, 146);
    assert!(
        distinct_pixel_count(
            &bytes,
            690,
            138,
            62,
            20,
            &[BUTTON_ATLAS_BACKGROUND, tab_surface]
        ) > 0,
        "selected module tab should paint active label/indicator pixels"
    );
}

#[test]
#[ignore = "writes local button component screenshot artifact for visual review"]
fn capture_button_component_visual_artifact() {
    let bytes = button_component_bytes();
    let output_path = visual_layout_output_path(BUTTON_COMPONENT_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        BUTTON_ATLAS_WIDTH,
        BUTTON_ATLAS_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("button component screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn button_component_bytes() -> Vec<u8> {
    paint_template_nodes_for_test_with_background(
        BUTTON_ATLAS_WIDTH,
        BUTTON_ATLAS_HEIGHT,
        BUTTON_ATLAS_BACKGROUND,
        model_rc(button_component_nodes()),
    )
}

fn button_component_nodes() -> Vec<TemplatePaneNodeData> {
    vec![
        surface("ButtonRoot", "shell", 0.0, 0.0, 900.0, 360.0),
        label("ButtonTitle", "Buttons", 22.0, 20.0, 220.0, 22.0, 13.0, ""),
        label(
            "ButtonSubtitle",
            "Text, icon, menu, pressed and tab states use retained button painters",
            22.0,
            42.0,
            560.0,
            18.0,
            10.0,
            "muted",
        ),
        surface("ButtonTextPanel", "panel", 18.0, 78.0, 188.0, 214.0),
        surface("ButtonIconPanel", "panel", 230.0, 78.0, 188.0, 214.0),
        surface("ButtonStatePanel", "panel", 442.0, 78.0, 188.0, 214.0),
        surface("ButtonTabPanel", "inset", 654.0, 78.0, 228.0, 214.0),
        label(
            "ButtonTextTitle",
            "Text Buttons",
            36.0,
            96.0,
            140.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "ButtonIconTitle",
            "Icon / Menu",
            248.0,
            96.0,
            140.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "ButtonStateTitle",
            "Pressed / Disabled",
            460.0,
            96.0,
            150.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "ButtonTabTitle",
            "Tab-like",
            672.0,
            96.0,
            140.0,
            18.0,
            11.0,
            "",
        ),
        button(
            "WorkbenchPrimaryButton",
            "Compile",
            "filled",
            "",
            42.0,
            130.0,
            132.0,
            34.0,
            ButtonState::Normal,
        ),
        button(
            "WorkbenchSecondaryButton",
            "Browse",
            "outlined",
            "",
            42.0,
            186.0,
            132.0,
            34.0,
            ButtonState::Normal,
        ),
        button(
            "WorkbenchAddComponent",
            "Add Component",
            "outlined",
            "",
            250.0,
            130.0,
            146.0,
            34.0,
            ButtonState::Normal,
        ),
        button(
            "WorkbenchDropdownButton",
            "More",
            "outlined",
            "",
            250.0,
            186.0,
            146.0,
            34.0,
            ButtonState::Normal,
        ),
        button(
            "WorkbenchPressedButton",
            "Pressed",
            "outlined",
            "",
            462.0,
            130.0,
            136.0,
            34.0,
            ButtonState::Pressed,
        ),
        button(
            "WorkbenchDisabledButton",
            "Disabled",
            "outlined",
            "",
            462.0,
            186.0,
            136.0,
            34.0,
            ButtonState::Disabled,
        ),
        button(
            "WorkbenchModuleScene",
            "Scene",
            "text",
            "",
            672.0,
            130.0,
            72.0,
            30.0,
            ButtonState::Selected,
        ),
        button(
            "WorkbenchModuleAssets",
            "Assets",
            "text",
            "",
            750.0,
            130.0,
            78.0,
            30.0,
            ButtonState::Normal,
        ),
        button(
            "AssetBrowserPreviewTabButton",
            "Preview",
            "text",
            "",
            672.0,
            186.0,
            88.0,
            28.0,
            ButtonState::Selected,
        ),
        button(
            "AssetBrowserMetadataTabButton",
            "Meta",
            "text",
            "",
            766.0,
            186.0,
            72.0,
            28.0,
            ButtonState::Hovered,
        ),
        label(
            "ButtonTabCopy",
            "Selected tab stays quiet",
            672.0,
            232.0,
            180.0,
            18.0,
            10.0,
            "muted",
        ),
    ]
}

#[derive(Clone, Copy)]
enum ButtonState {
    Normal,
    Hovered,
    Selected,
    Pressed,
    Disabled,
}

fn button(
    control_id: &str,
    text: &str,
    variant: &str,
    icon_name: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: ButtonState,
) -> TemplatePaneNodeData {
    let selected = matches!(state, ButtonState::Selected);
    let mut node = TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Button".into(),
        component_role: "button".into(),
        text: text.into(),
        button_variant: variant.into(),
        icon_name: icon_name.into(),
        selected,
        checked: selected,
        hovered: matches!(state, ButtonState::Hovered),
        pressed: matches!(state, ButtonState::Pressed),
        disabled: matches!(state, ButtonState::Disabled),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    };
    if control_id == "WorkbenchPressedButton" {
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
    let index = ((y as usize * BUTTON_ATLAS_WIDTH as usize) + x as usize) * 4;
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
            let index = ((py as usize * BUTTON_ATLAS_WIDTH as usize) + px as usize) * 4;
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
