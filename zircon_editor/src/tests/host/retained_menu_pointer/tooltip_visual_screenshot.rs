use std::path::{Path, PathBuf};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::primitives::Color;
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData, TemplatePaneNodeData,
};

const TOOLTIP_COMPONENT_SCREENSHOT: &str = "editor-components-tooltips-900x360.png";
const TOOLTIP_ATLAS_WIDTH: u32 = 900;
const TOOLTIP_ATLAS_HEIGHT: u32 = 360;
const TOOLTIP_ATLAS_BACKGROUND: [u8; 4] = [17, 20, 22, 255];
const DECLARED_ARROW: [u8; 4] = [26, 38, 42, 255];
const DECLARED_BODY: [u8; 4] = [155, 180, 184, 255];
const DECLARED_ICON: [u8; 4] = [37, 156, 167, 255];

#[test]
fn tooltip_component_visual_paints_compact_defaults_and_declared_icon() {
    let bytes = tooltip_component_bytes();

    let default_panel = pixel_at(&bytes, 30, 92);
    let default_bubble = pixel_at(&bytes, 92, 144);
    assert!(
        distinct_pixel_count(
            &bytes,
            86,
            136,
            72,
            34,
            &[TOOLTIP_ATLAS_BACKGROUND, default_panel, default_bubble],
        ) > 0,
        "ordinary tooltip should paint retained title/body text above the bubble surface"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            118,
            170,
            20,
            16,
            &[TOOLTIP_ATLAS_BACKGROUND, default_panel],
        ) > 0,
        "ordinary tooltip should paint the arrow diamond below the bubble"
    );
    assert_eq!(
        distinct_pixel_count(
            &bytes,
            118,
            190,
            20,
            20,
            &[TOOLTIP_ATLAS_BACKGROUND, default_panel],
        ),
        0,
        "ordinary tooltip should keep its lower panel area clear unless an icon is declared"
    );

    let declared_panel = pixel_at(&bytes, 242, 92);
    assert!(
        exact_pixel_count(&bytes, 328, 170, 22, 18, DECLARED_ARROW) > 0,
        "declared tooltip should paint the custom arrow color"
    );
    assert!(
        exact_pixel_count(&bytes, 286, 154, 80, 16, DECLARED_BODY) > 0,
        "declared tooltip should paint the custom body text color"
    );
    assert!(
        exact_pixel_count(&bytes, 330, 190, 20, 20, DECLARED_ICON) > 0,
        "declared tooltip should paint the custom info icon color"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            288,
            136,
            70,
            18,
            &[TOOLTIP_ATLAS_BACKGROUND, declared_panel],
        ) > 0,
        "declared tooltip should still paint title text through the shared text path"
    );

    let state_panel = pixel_at(&bytes, 454, 92);
    let pressed_bubble = pixel_at(&bytes, 516, 144);
    assert_ne!(
        pixel_at(&bytes, 510, 126),
        TOOLTIP_ATLAS_BACKGROUND,
        "pressed tooltip should paint a visible active border"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            526,
            136,
            74,
            32,
            &[TOOLTIP_ATLAS_BACKGROUND, state_panel, pressed_bubble],
        ) > 0,
        "pressed tooltip should keep title/body text above the active bubble"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            528,
            222,
            76,
            30,
            &[TOOLTIP_ATLAS_BACKGROUND, state_panel],
        ) > 0,
        "focused tooltip should keep text pixels without switching to hover fill"
    );

    let disabled_panel = pixel_at(&bytes, 666, 92);
    let disabled_bubble = pixel_at(&bytes, 730, 144);
    assert!(
        distinct_pixel_count(
            &bytes,
            738,
            136,
            72,
            34,
            &[TOOLTIP_ATLAS_BACKGROUND, disabled_panel, disabled_bubble],
        ) > 0,
        "disabled tooltip should still paint muted title/body text"
    );
    assert_eq!(
        distinct_pixel_count(
            &bytes,
            738,
            190,
            20,
            20,
            &[TOOLTIP_ATLAS_BACKGROUND, disabled_panel],
        ),
        0,
        "disabled tooltip should retain the compact no-icon default"
    );
}

#[test]
#[ignore = "writes local tooltip component screenshot artifact for visual review"]
fn capture_tooltip_component_visual_artifact() {
    let bytes = tooltip_component_bytes();
    let output_path = visual_layout_output_path(TOOLTIP_COMPONENT_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        TOOLTIP_ATLAS_WIDTH,
        TOOLTIP_ATLAS_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("tooltip component screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn tooltip_component_bytes() -> Vec<u8> {
    paint_template_nodes_for_test_with_background(
        TOOLTIP_ATLAS_WIDTH,
        TOOLTIP_ATLAS_HEIGHT,
        TOOLTIP_ATLAS_BACKGROUND,
        model_rc(tooltip_component_nodes()),
    )
}

fn tooltip_component_nodes() -> Vec<TemplatePaneNodeData> {
    vec![
        surface("TooltipRoot", "shell", 0.0, 0.0, 900.0, 360.0),
        label(
            "TooltipTitle",
            "Tooltips",
            22.0,
            20.0,
            220.0,
            22.0,
            13.0,
            "",
        ),
        label(
            "TooltipSubtitle",
            "Bubble, arrow, declared icon, colors, focused, pressed and disabled states",
            22.0,
            42.0,
            660.0,
            18.0,
            10.0,
            "muted",
        ),
        surface("TooltipDefaultPanel", "panel", 18.0, 78.0, 188.0, 214.0),
        surface("TooltipDeclaredPanel", "panel", 230.0, 78.0, 188.0, 214.0),
        surface("TooltipStatePanel", "panel", 442.0, 78.0, 188.0, 214.0),
        surface("TooltipDisabledPanel", "inset", 654.0, 78.0, 228.0, 214.0),
        label(
            "TooltipDefaultTitle",
            "Default",
            36.0,
            96.0,
            140.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "TooltipDeclaredTitle",
            "Declared",
            248.0,
            96.0,
            140.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "TooltipStateTitle",
            "States",
            460.0,
            96.0,
            140.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "TooltipDisabledTitle",
            "Disabled",
            672.0,
            96.0,
            140.0,
            18.0,
            11.0,
            "",
        ),
        tooltip(
            "WorkbenchTooltipViewport",
            "Viewport",
            "Hold Shift",
            42.0,
            126.0,
            142.0,
            88.0,
            TooltipState::Normal,
            TooltipColors::Default,
        ),
        tooltip(
            "WorkbenchTooltipHover",
            "Hover",
            "Snap grid",
            42.0,
            210.0,
            142.0,
            72.0,
            TooltipState::Hovered,
            TooltipColors::Default,
        ),
        tooltip(
            "WorkbenchTooltipDeclared",
            "Asset Tip",
            "Async load",
            254.0,
            126.0,
            142.0,
            88.0,
            TooltipState::Normal,
            TooltipColors::Declared,
        ),
        tooltip(
            "WorkbenchTooltipPressed",
            "Pressed",
            "Confirm now",
            466.0,
            126.0,
            142.0,
            88.0,
            TooltipState::Pressed,
            TooltipColors::Default,
        ),
        tooltip(
            "WorkbenchTooltipFocused",
            "Focused",
            "Keyboard",
            466.0,
            210.0,
            142.0,
            72.0,
            TooltipState::Focused,
            TooltipColors::Default,
        ),
        tooltip(
            "WorkbenchTooltipDisabled",
            "Unavailable",
            "No target",
            678.0,
            126.0,
            142.0,
            88.0,
            TooltipState::Disabled,
            TooltipColors::Default,
        ),
        label(
            "TooltipDisabledCopy",
            "Muted bubble, text and shadow",
            672.0,
            232.0,
            190.0,
            32.0,
            10.0,
            "muted",
        ),
    ]
}

#[derive(Clone, Copy)]
enum TooltipState {
    Normal,
    Hovered,
    Focused,
    Pressed,
    Disabled,
}

#[derive(Clone, Copy)]
enum TooltipColors {
    Default,
    Declared,
}

fn tooltip(
    control_id: &str,
    title: &str,
    body: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: TooltipState,
    colors: TooltipColors,
) -> TemplatePaneNodeData {
    let mut node = TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Tooltip".into(),
        component_role: "tooltip".into(),
        surface_variant: "workbench-tooltip".into(),
        text: title.into(),
        label_text: body.into(),
        value_number: 8.0,
        layout_icon_size: 18.0,
        layout_content_offset_y: icon_offset_for_height(height),
        hovered: matches!(state, TooltipState::Hovered),
        focused: matches!(state, TooltipState::Focused),
        pressed: matches!(state, TooltipState::Pressed),
        disabled: matches!(state, TooltipState::Disabled),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    };

    if matches!(colors, TooltipColors::Declared) {
        node.icon_name = "info".into();
        node.value_number = 10.0;
        node.value_color =
            Color::from_rgb_u8(DECLARED_ARROW[0], DECLARED_ARROW[1], DECLARED_ARROW[2]);
        node.label_color = Color::from_rgb_u8(DECLARED_BODY[0], DECLARED_BODY[1], DECLARED_BODY[2]);
        node.icon_color = Color::from_rgb_u8(DECLARED_ICON[0], DECLARED_ICON[1], DECLARED_ICON[2]);
    }

    node
}

fn icon_offset_for_height(height: f32) -> f32 {
    (height - 24.0).max(46.0)
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
    let index = ((y as usize * TOOLTIP_ATLAS_WIDTH as usize) + x as usize) * 4;
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
            let index = ((py as usize * TOOLTIP_ATLAS_WIDTH as usize) + px as usize) * 4;
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

fn exact_pixel_count(
    bytes: &[u8],
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    expected: [u8; 4],
) -> usize {
    let mut matches = 0;
    for py in y..(y + height) {
        for px in x..(x + width) {
            let index = ((py as usize * TOOLTIP_ATLAS_WIDTH as usize) + px as usize) * 4;
            let color = [
                bytes[index],
                bytes[index + 1],
                bytes[index + 2],
                bytes[index + 3],
            ];
            if color == expected {
                matches += 1;
            }
        }
    }
    matches
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
