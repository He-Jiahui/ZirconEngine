use std::path::{Path, PathBuf};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData, TemplatePaneNodeData,
};

const ICON_BUTTON_COMPONENT_SCREENSHOT: &str = "editor-components-icon-buttons-900x360.png";
const ICON_BUTTON_ATLAS_WIDTH: u32 = 900;
const ICON_BUTTON_ATLAS_HEIGHT: u32 = 360;
const ICON_BUTTON_ATLAS_BACKGROUND: [u8; 4] = [17, 20, 22, 255];

#[test]
fn icon_button_component_visual_paints_context_sizes_and_pressed_offset() {
    let bytes = icon_button_component_bytes();
    let toolbar_panel = pixel_at(&bytes, 30, 92);

    assert_eq!(
        pixel_at(&bytes, 56, 132),
        toolbar_panel,
        "a normal toolbar icon button must stay quiet instead of painting an input-like background"
    );

    assert!(
        changed_pixel_count(&bytes, 64, 138, 32, 32) > 0,
        "toolbar icon button should paint a visible Icon20 glyph"
    );
    assert!(
        changed_pixel_count(&bytes, 260, 140, 30, 30) > 0,
        "panel icon button should paint a visible Icon16 glyph"
    );
    assert!(
        changed_pixel_count(&bytes, 444, 132, 42, 42) > 0,
        "rail icon button should paint a visible Icon24 glyph"
    );
    assert!(
        changed_pixel_count(&bytes, 640, 144, 32, 32) > 0,
        "pressed toolbar icon button should paint its lowered glyph region"
    );
    assert_ne!(
        pixel_at(&bytes, 676, 136),
        pixel_at(&bytes, 746, 136),
        "pressed toolbar icon button should retain its rounded selection surface while disabled stays quiet"
    );
}

#[test]
#[ignore = "writes local icon-button component screenshot artifact for visual review"]
fn capture_icon_button_component_visual_artifact() {
    let bytes = icon_button_component_bytes();
    let output_path = visual_layout_output_path(ICON_BUTTON_COMPONENT_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        ICON_BUTTON_ATLAS_WIDTH,
        ICON_BUTTON_ATLAS_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("icon-button component screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn icon_button_component_bytes() -> Vec<u8> {
    paint_template_nodes_for_test_with_background(
        ICON_BUTTON_ATLAS_WIDTH,
        ICON_BUTTON_ATLAS_HEIGHT,
        ICON_BUTTON_ATLAS_BACKGROUND,
        model_rc(icon_button_component_nodes()),
    )
}

fn icon_button_component_nodes() -> Vec<TemplatePaneNodeData> {
    vec![
        surface("IconButtonRoot", "shell", 0.0, 0.0, 900.0, 360.0),
        label(
            "IconButtonTitle",
            "Icon Buttons",
            22.0,
            20.0,
            260.0,
            22.0,
            13.0,
            "",
        ),
        label(
            "IconButtonSubtitle",
            "Toolbar, panel, rail and pressed states use retained icon painters",
            22.0,
            42.0,
            560.0,
            18.0,
            10.0,
            "muted",
        ),
        surface("IconToolbarPanel", "panel", 18.0, 78.0, 188.0, 214.0),
        surface("IconPanelButtonsPanel", "panel", 230.0, 78.0, 188.0, 214.0),
        surface("IconRailPanel", "panel", 442.0, 78.0, 188.0, 214.0),
        surface("IconPressedPanel", "inset", 654.0, 78.0, 228.0, 214.0),
        label(
            "IconToolbarTitle",
            "Toolbar",
            36.0,
            96.0,
            120.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "IconPanelTitle",
            "Panel",
            248.0,
            96.0,
            120.0,
            18.0,
            11.0,
            "",
        ),
        label("IconRailTitle", "Rail", 460.0, 96.0, 120.0, 18.0, 11.0, ""),
        label(
            "IconPressedTitle",
            "Pressed / Disabled",
            672.0,
            96.0,
            150.0,
            18.0,
            11.0,
            "",
        ),
        icon_button(
            "WorkbenchToolbarMenu",
            "zircon_editor_shell/toolbar/menu.svg",
            54.0,
            130.0,
            48.0,
            40.0,
            IconState::Normal,
        ),
        label(
            "IconToolbarCopy",
            "Icon20",
            52.0,
            184.0,
            96.0,
            18.0,
            10.0,
            "muted",
        ),
        icon_button(
            "WorkbenchMiniAdd",
            "zircon_editor_shell/controls/add.svg",
            262.0,
            132.0,
            38.0,
            38.0,
            IconState::Normal,
        ),
        label(
            "IconPanelCopy",
            "Icon16",
            260.0,
            184.0,
            96.0,
            18.0,
            10.0,
            "muted",
        ),
        icon_button(
            "WorkbenchRailAssets",
            "zircon_editor_shell/activity/cube.svg",
            454.0,
            124.0,
            56.0,
            56.0,
            IconState::Selected,
        ),
        label(
            "IconRailCopy",
            "Icon24",
            468.0,
            194.0,
            96.0,
            18.0,
            10.0,
            "muted",
        ),
        icon_button(
            "WorkbenchToolbarPressed",
            "zircon_editor_shell/toolbar/menu.svg",
            672.0,
            130.0,
            48.0,
            40.0,
            IconState::Pressed,
        ),
        icon_button(
            "WorkbenchToolbarDisabled",
            "zircon_editor_shell/toolbar/menu.svg",
            746.0,
            130.0,
            48.0,
            40.0,
            IconState::Disabled,
        ),
        label(
            "IconPressedCopy",
            "Pressed drops 1px",
            672.0,
            190.0,
            176.0,
            18.0,
            10.0,
            "muted",
        ),
        label(
            "IconDisabledCopy",
            "Disabled stays muted",
            672.0,
            208.0,
            176.0,
            18.0,
            10.0,
            "muted",
        ),
    ]
}

#[derive(Clone, Copy)]
enum IconState {
    Normal,
    Selected,
    Pressed,
    Disabled,
}

fn icon_button(
    control_id: &str,
    icon_name: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: IconState,
) -> TemplatePaneNodeData {
    let selected = matches!(state, IconState::Selected);
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "IconButton".into(),
        icon_name: icon_name.into(),
        selected,
        checked: selected,
        pressed: matches!(state, IconState::Pressed),
        disabled: matches!(state, IconState::Disabled),
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

fn changed_pixel_count(bytes: &[u8], x: u32, y: u32, width: u32, height: u32) -> usize {
    let mut changed = 0;
    for py in y..(y + height) {
        for px in x..(x + width) {
            let index = ((py as usize * ICON_BUTTON_ATLAS_WIDTH as usize) + px as usize) * 4;
            if bytes[index..index + 4] != ICON_BUTTON_ATLAS_BACKGROUND {
                changed += 1;
            }
        }
    }
    changed
}

fn pixel_at(bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
    let index = ((y as usize * ICON_BUTTON_ATLAS_WIDTH as usize) + x as usize) * 4;
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
