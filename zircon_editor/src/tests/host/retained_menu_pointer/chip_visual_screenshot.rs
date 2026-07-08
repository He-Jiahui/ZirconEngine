use std::path::{Path, PathBuf};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData, TemplatePaneNodeData,
};

const CHIP_COMPONENT_SCREENSHOT: &str = "editor-components-chips-900x360.png";
const CHIP_ATLAS_WIDTH: u32 = 900;
const CHIP_ATLAS_HEIGHT: u32 = 360;
const CHIP_ATLAS_BACKGROUND: [u8; 4] = [17, 20, 22, 255];

#[test]
fn chip_component_visual_paints_pill_chevron_focus_pressed_and_disabled() {
    let bytes = chip_component_bytes();

    let filter_surface = pixel_at(&bytes, 52, 146);
    assert!(
        distinct_pixel_count(
            &bytes,
            58,
            138,
            82,
            20,
            &[CHIP_ATLAS_BACKGROUND, filter_surface],
        ) > 0,
        "ordinary chip should paint readable retained label text"
    );

    let tag_surface = pixel_at(&bytes, 262, 146);
    assert!(
        distinct_pixel_count(
            &bytes,
            272,
            138,
            84,
            20,
            &[CHIP_ATLAS_BACKGROUND, tag_surface],
        ) > 0,
        "pill/tag chip should paint its compact label"
    );

    let viewport_surface = pixel_at(&bytes, 474, 146);
    assert!(
        distinct_pixel_count(
            &bytes,
            580,
            138,
            18,
            20,
            &[CHIP_ATLAS_BACKGROUND, viewport_surface],
        ) > 0,
        "viewport chip should paint trailing chevron segments"
    );

    let open_border = pixel_at(&bytes, 462, 184);
    assert_ne!(
        open_border, CHIP_ATLAS_BACKGROUND,
        "open chip should paint an active focus-style border"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            580,
            194,
            18,
            20,
            &[CHIP_ATLAS_BACKGROUND, pixel_at(&bytes, 474, 202)],
        ) > 0,
        "open chip should paint active chevron glyph pixels"
    );

    let focused_border = pixel_at(&bytes, 678, 128);
    assert_ne!(
        focused_border, CHIP_ATLAS_BACKGROUND,
        "focused chip should paint a visible focus border"
    );

    let pressed_surface = pixel_at(&bytes, 682, 202);
    assert!(
        distinct_pixel_count(
            &bytes,
            698,
            194,
            70,
            20,
            &[CHIP_ATLAS_BACKGROUND, pressed_surface],
        ) > 0,
        "pressed chip should paint label pixels over pressed surface"
    );

    let disabled_surface = pixel_at(&bytes, 682, 258);
    assert!(
        distinct_pixel_count(
            &bytes,
            698,
            250,
            76,
            20,
            &[CHIP_ATLAS_BACKGROUND, disabled_surface],
        ) > 0,
        "disabled chip should still paint muted label pixels"
    );
}

#[test]
#[ignore = "writes local chip/tag component screenshot artifact for visual review"]
fn capture_chip_component_visual_artifact() {
    let bytes = chip_component_bytes();
    let output_path = visual_layout_output_path(CHIP_COMPONENT_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        CHIP_ATLAS_WIDTH,
        CHIP_ATLAS_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("chip component screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn chip_component_bytes() -> Vec<u8> {
    paint_template_nodes_for_test_with_background(
        CHIP_ATLAS_WIDTH,
        CHIP_ATLAS_HEIGHT,
        CHIP_ATLAS_BACKGROUND,
        model_rc(chip_component_nodes()),
    )
}

fn chip_component_nodes() -> Vec<TemplatePaneNodeData> {
    vec![
        surface("ChipRoot", "shell", 0.0, 0.0, 900.0, 360.0),
        label("ChipTitle", "Chips", 22.0, 20.0, 220.0, 22.0, 13.0, ""),
        label(
            "ChipSubtitle",
            "Filter chips, tag pills, viewport chips, focused, pressed and disabled states",
            22.0,
            42.0,
            650.0,
            18.0,
            10.0,
            "muted",
        ),
        surface("ChipFilterPanel", "panel", 18.0, 78.0, 188.0, 214.0),
        surface("ChipTagPanel", "panel", 230.0, 78.0, 188.0, 214.0),
        surface("ChipViewportPanel", "panel", 442.0, 78.0, 188.0, 214.0),
        surface("ChipStatePanel", "inset", 654.0, 78.0, 228.0, 214.0),
        label(
            "ChipFilterTitle",
            "Filters",
            36.0,
            96.0,
            130.0,
            18.0,
            11.0,
            "",
        ),
        label("ChipTagTitle", "Tags", 248.0, 96.0, 130.0, 18.0, 11.0, ""),
        label(
            "ChipViewportTitle",
            "Viewport",
            460.0,
            96.0,
            130.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "ChipStateTitle",
            "States",
            672.0,
            96.0,
            130.0,
            18.0,
            11.0,
            "",
        ),
        chip(
            "WorkbenchFilterChipAll",
            "All",
            "chip",
            "",
            42.0,
            130.0,
            92.0,
            30.0,
            ChipState::Normal,
            false,
        ),
        chip(
            "WorkbenchFilterChipMeshes",
            "Meshes",
            "chip",
            "",
            42.0,
            186.0,
            112.0,
            30.0,
            ChipState::Hovered,
            false,
        ),
        chip(
            "WorkbenchTagPillPhysics",
            "Physics",
            "pill",
            "",
            250.0,
            130.0,
            116.0,
            30.0,
            ChipState::Normal,
            false,
        ),
        chip(
            "WorkbenchTagPillReview",
            "Needs Review",
            "pill",
            "muted",
            250.0,
            186.0,
            136.0,
            30.0,
            ChipState::Normal,
            false,
        ),
        chip(
            "WorkbenchViewportMode",
            "Perspective",
            "chip",
            "",
            462.0,
            130.0,
            148.0,
            30.0,
            ChipState::Normal,
            true,
        ),
        chip(
            "WorkbenchViewportLit",
            "Lit",
            "chip",
            "",
            462.0,
            186.0,
            148.0,
            30.0,
            ChipState::Open,
            true,
        ),
        chip(
            "WorkbenchStateFocusedChip",
            "Focused",
            "chip",
            "",
            678.0,
            130.0,
            132.0,
            30.0,
            ChipState::Focused,
            false,
        ),
        chip(
            "WorkbenchStatePressedChip",
            "Pressed",
            "chip",
            "",
            678.0,
            186.0,
            132.0,
            30.0,
            ChipState::Pressed,
            false,
        ),
        chip(
            "WorkbenchStateDisabledChip",
            "Disabled",
            "chip",
            "",
            678.0,
            242.0,
            132.0,
            30.0,
            ChipState::Disabled,
            false,
        ),
    ]
}

#[derive(Clone, Copy)]
enum ChipState {
    Normal,
    Hovered,
    Focused,
    Pressed,
    Open,
    Disabled,
}

fn chip(
    control_id: &str,
    text: &str,
    component_role: &str,
    text_tone: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: ChipState,
    has_options: bool,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Label".into(),
        component_role: component_role.into(),
        text: text.into(),
        text_tone: text_tone.into(),
        hovered: matches!(state, ChipState::Hovered),
        focused: matches!(state, ChipState::Focused),
        pressed: matches!(state, ChipState::Pressed),
        popup_open: matches!(state, ChipState::Open),
        disabled: matches!(state, ChipState::Disabled),
        options: if has_options {
            model_rc(vec!["menu".into()])
        } else {
            model_rc(Vec::new())
        },
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
    let index = ((y as usize * CHIP_ATLAS_WIDTH as usize) + x as usize) * 4;
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
            let index = ((py as usize * CHIP_ATLAS_WIDTH as usize) + px as usize) * 4;
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
