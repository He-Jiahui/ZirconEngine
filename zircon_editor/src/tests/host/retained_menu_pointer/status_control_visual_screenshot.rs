use std::path::{Path, PathBuf};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData, TemplatePaneNodeData,
};

const STATUS_CONTROL_COMPONENT_SCREENSHOT: &str = "editor-components-status-controls-900x360.png";
const STATUS_CONTROL_ATLAS_WIDTH: u32 = 900;
const STATUS_CONTROL_ATLAS_HEIGHT: u32 = 360;
const STATUS_CONTROL_ATLAS_BACKGROUND: [u8; 4] = [17, 20, 22, 255];

#[test]
fn status_control_component_visual_paints_signals_chips_icons_and_states() {
    let bytes = status_control_component_bytes();

    let signal_panel = pixel_at(&bytes, 34, 278);
    assert_ne!(
        pixel_at(&bytes, 66, 137),
        signal_panel,
        "Ready status item should paint its compact inline marker"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            80,
            128,
            90,
            20,
            &[STATUS_CONTROL_ATLAS_BACKGROUND, signal_panel],
        ) > 0,
        "status signals should paint readable retained text"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            80,
            242,
            112,
            20,
            &[STATUS_CONTROL_ATLAS_BACKGROUND, signal_panel],
        ) > 0,
        "disabled status signal should still paint muted text"
    );

    let chip_panel = pixel_at(&bytes, 246, 278);
    assert!(
        distinct_pixel_count(
            &bytes,
            268,
            128,
            116,
            22,
            &[STATUS_CONTROL_ATLAS_BACKGROUND, chip_panel],
        ) > 0,
        "flat status chip should paint label/value text without a visible button fill"
    );
    assert_ne!(
        pixel_at(&bytes, 250, 179),
        chip_panel,
        "focused status chip should paint a slim focus border"
    );
    assert_ne!(
        pixel_at(&bytes, 254, 238),
        chip_panel,
        "disabled status chip should paint its muted surface"
    );

    let icon_panel = pixel_at(&bytes, 458, 278);
    assert!(
        distinct_pixel_count(
            &bytes,
            476,
            132,
            26,
            24,
            &[STATUS_CONTROL_ATLAS_BACKGROUND, icon_panel],
        ) > 0,
        "status snap toggle should paint its glyph"
    );
    assert_ne!(
        pixel_at(&bytes, 474, 178),
        icon_panel,
        "checked status icon should paint the selected status surface"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            682,
            130,
            150,
            112,
            &[STATUS_CONTROL_ATLAS_BACKGROUND],
        ) > 0,
        "composite status strip should paint the same retained controls together"
    );
}

#[test]
#[ignore = "writes local status-control component screenshot artifact for visual review"]
fn capture_status_control_component_visual_artifact() {
    let bytes = status_control_component_bytes();
    let output_path = visual_layout_output_path(STATUS_CONTROL_COMPONENT_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        STATUS_CONTROL_ATLAS_WIDTH,
        STATUS_CONTROL_ATLAS_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("status-control component screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn status_control_component_bytes() -> Vec<u8> {
    paint_template_nodes_for_test_with_background(
        STATUS_CONTROL_ATLAS_WIDTH,
        STATUS_CONTROL_ATLAS_HEIGHT,
        STATUS_CONTROL_ATLAS_BACKGROUND,
        model_rc(status_control_component_nodes()),
    )
}

fn status_control_component_nodes() -> Vec<TemplatePaneNodeData> {
    vec![
        surface("StatusControlRoot", "shell", 0.0, 0.0, 900.0, 360.0),
        label(
            "StatusControlTitle",
            "Status Controls",
            22.0,
            20.0,
            260.0,
            22.0,
            13.0,
            "",
        ),
        label(
            "StatusControlSubtitle",
            "Signals, value chips and status icon toggles use retained status painters",
            22.0,
            42.0,
            640.0,
            18.0,
            10.0,
            "muted",
        ),
        surface("StatusSignalsPanel", "panel", 18.0, 78.0, 188.0, 214.0),
        surface("StatusChipsPanel", "panel", 230.0, 78.0, 188.0, 214.0),
        surface("StatusIconsPanel", "panel", 442.0, 78.0, 188.0, 214.0),
        surface("StatusStripPanel", "inset", 654.0, 78.0, 228.0, 214.0),
        label(
            "StatusSignalsTitle",
            "Signals",
            36.0,
            96.0,
            120.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "StatusChipsTitle",
            "Value Chips",
            248.0,
            96.0,
            120.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "StatusIconsTitle",
            "Icon Toggles",
            460.0,
            96.0,
            130.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "StatusStripTitle",
            "Status Strip",
            672.0,
            96.0,
            130.0,
            18.0,
            11.0,
            "",
        ),
        signal(
            "WorkbenchStatusReady",
            "Ready",
            38.0,
            122.0,
            148.0,
            30.0,
            StatusState::Normal,
        ),
        signal(
            "WorkbenchStatusErrors",
            "No Errors",
            38.0,
            160.0,
            148.0,
            30.0,
            StatusState::Normal,
        ),
        signal(
            "WorkbenchStatusWarnings",
            "2 Warnings",
            38.0,
            198.0,
            148.0,
            30.0,
            StatusState::Normal,
        ),
        signal(
            "WorkbenchStatusMessages",
            "Muted",
            38.0,
            236.0,
            148.0,
            30.0,
            StatusState::Disabled,
        ),
        chip(
            "WorkbenchStatusGrid",
            "Grid: 10 cm",
            250.0,
            122.0,
            136.0,
            30.0,
            StatusState::Normal,
        ),
        chip(
            "WorkbenchStatusSnap",
            "Snap: On",
            250.0,
            160.0,
            136.0,
            30.0,
            StatusState::Focused,
        ),
        chip(
            "WorkbenchStatusZoom",
            "100%",
            250.0,
            198.0,
            136.0,
            30.0,
            StatusState::Hovered,
        ),
        chip(
            "WorkbenchStatusGrid",
            "Grid: Off",
            250.0,
            236.0,
            136.0,
            30.0,
            StatusState::Disabled,
        ),
        icon(
            "WorkbenchStatusSnapToggle",
            462.0,
            122.0,
            40.0,
            30.0,
            StatusState::Normal,
        ),
        icon(
            "WorkbenchStatusWorld",
            508.0,
            122.0,
            40.0,
            30.0,
            StatusState::Focused,
        ),
        icon(
            "WorkbenchStatusTarget",
            554.0,
            122.0,
            40.0,
            30.0,
            StatusState::Disabled,
        ),
        icon(
            "WorkbenchStatusSnapToggle",
            462.0,
            170.0,
            40.0,
            30.0,
            StatusState::Checked,
        ),
        icon(
            "WorkbenchStatusWorld",
            508.0,
            170.0,
            40.0,
            30.0,
            StatusState::Hovered,
        ),
        icon(
            "WorkbenchStatusTarget",
            554.0,
            170.0,
            40.0,
            30.0,
            StatusState::Pressed,
        ),
        label(
            "StatusIconsCopy",
            "Snap, world and target keep a flat toolbar footprint",
            460.0,
            224.0,
            144.0,
            36.0,
            10.0,
            "muted",
        ),
        surface("StatusStripSurface", "panel", 672.0, 128.0, 190.0, 96.0),
        signal(
            "WorkbenchStatusReady",
            "Ready",
            672.0,
            138.0,
            72.0,
            28.0,
            StatusState::Normal,
        ),
        signal(
            "WorkbenchStatusWarnings",
            "2 Warnings",
            746.0,
            138.0,
            96.0,
            28.0,
            StatusState::Normal,
        ),
        chip(
            "WorkbenchStatusGrid",
            "Grid: 10 cm",
            682.0,
            178.0,
            112.0,
            28.0,
            StatusState::Normal,
        ),
        icon(
            "WorkbenchStatusSnapToggle",
            812.0,
            177.0,
            34.0,
            30.0,
            StatusState::Checked,
        ),
    ]
}

#[derive(Clone, Copy)]
enum StatusState {
    Normal,
    Hovered,
    Focused,
    Pressed,
    Checked,
    Disabled,
}

fn signal(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: StatusState,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        hovered: matches!(state, StatusState::Hovered),
        focused: matches!(state, StatusState::Focused),
        pressed: matches!(state, StatusState::Pressed),
        checked: matches!(state, StatusState::Checked),
        selected: matches!(state, StatusState::Checked),
        disabled: matches!(state, StatusState::Disabled),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn chip(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: StatusState,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        hovered: matches!(state, StatusState::Hovered),
        focused: matches!(state, StatusState::Focused),
        pressed: matches!(state, StatusState::Pressed),
        checked: matches!(state, StatusState::Checked),
        selected: matches!(state, StatusState::Checked),
        disabled: matches!(state, StatusState::Disabled),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn icon(
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: StatusState,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "IconButton".into(),
        hovered: matches!(state, StatusState::Hovered),
        focused: matches!(state, StatusState::Focused),
        pressed: matches!(state, StatusState::Pressed),
        checked: matches!(state, StatusState::Checked),
        selected: matches!(state, StatusState::Checked),
        disabled: matches!(state, StatusState::Disabled),
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
    let index = ((y as usize * STATUS_CONTROL_ATLAS_WIDTH as usize) + x as usize) * 4;
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
            let index = ((py as usize * STATUS_CONTROL_ATLAS_WIDTH as usize) + px as usize) * 4;
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
