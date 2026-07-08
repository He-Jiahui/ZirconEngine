use std::path::{Path, PathBuf};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData, TemplatePaneNodeData,
};

const SLIDER_COMPONENT_SCREENSHOT: &str = "editor-components-sliders-900x360.png";
const SLIDER_ATLAS_WIDTH: u32 = 900;
const SLIDER_ATLAS_HEIGHT: u32 = 360;
const SLIDER_ATLAS_BACKGROUND: [u8; 4] = [17, 20, 22, 255];

#[test]
fn slider_component_visual_paints_slider_steps_range_focus_pressed_and_disabled() {
    let bytes = slider_component_bytes();

    let basic_panel = pixel_at(&bytes, 30, 92);
    assert!(
        distinct_pixel_count(
            &bytes,
            98,
            143,
            150,
            10,
            &[SLIDER_ATLAS_BACKGROUND, basic_panel],
        ) > 0,
        "ordinary slider should paint track and fill pixels"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            248,
            132,
            54,
            30,
            &[SLIDER_ATLAS_BACKGROUND, basic_panel],
        ) > 0,
        "ordinary slider should paint retained value chip text"
    );

    let stepped_panel = pixel_at(&bytes, 330, 92);
    assert!(
        distinct_pixel_count(
            &bytes,
            120,
            200,
            160,
            18,
            &[SLIDER_ATLAS_BACKGROUND, basic_panel, stepped_panel],
        ) > 0,
        "steps slider should paint tick marks below the track"
    );

    let range_panel = pixel_at(&bytes, 330, 92);
    assert!(
        distinct_pixel_count(
            &bytes,
            398,
            151,
            138,
            12,
            &[SLIDER_ATLAS_BACKGROUND, range_panel],
        ) > 0,
        "range slider should paint a span fill between two thumbs"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            382,
            173,
            64,
            30,
            &[SLIDER_ATLAS_BACKGROUND, range_panel],
        ) > 0,
        "range slider should paint the min value chip"
    );

    let state_panel = pixel_at(&bytes, 666, 92);
    assert!(
        distinct_pixel_count(
            &bytes,
            724,
            136,
            30,
            18,
            &[SLIDER_ATLAS_BACKGROUND, state_panel],
        ) > 0,
        "focused slider should paint a thumb halo without using a hover surface"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            748,
            196,
            30,
            18,
            &[SLIDER_ATLAS_BACKGROUND, state_panel],
        ) > 0,
        "pressed slider should paint a hot thumb halo"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            740,
            248,
            54,
            24,
            &[SLIDER_ATLAS_BACKGROUND, state_panel],
        ) > 0,
        "disabled slider should still paint muted value text"
    );
}

#[test]
#[ignore = "writes local slider/range component screenshot artifact for visual review"]
fn capture_slider_component_visual_artifact() {
    let bytes = slider_component_bytes();
    let output_path = visual_layout_output_path(SLIDER_COMPONENT_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        SLIDER_ATLAS_WIDTH,
        SLIDER_ATLAS_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("slider component screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn slider_component_bytes() -> Vec<u8> {
    paint_template_nodes_for_test_with_background(
        SLIDER_ATLAS_WIDTH,
        SLIDER_ATLAS_HEIGHT,
        SLIDER_ATLAS_BACKGROUND,
        model_rc(slider_component_nodes()),
    )
}

fn slider_component_nodes() -> Vec<TemplatePaneNodeData> {
    vec![
        surface("SliderRoot", "shell", 0.0, 0.0, 900.0, 360.0),
        label("SliderTitle", "Sliders", 22.0, 20.0, 220.0, 22.0, 13.0, ""),
        label(
            "SliderSubtitle",
            "Slider, stepped slider, range slider, focused, pressed and disabled states",
            22.0,
            42.0,
            660.0,
            18.0,
            10.0,
            "muted",
        ),
        surface("SliderBasicPanel", "panel", 18.0, 78.0, 288.0, 214.0),
        surface("SliderRangePanel", "panel", 330.0, 78.0, 300.0, 214.0),
        surface("SliderStatePanel", "inset", 654.0, 78.0, 228.0, 214.0),
        label(
            "SliderBasicTitle",
            "Slider / Steps",
            36.0,
            96.0,
            180.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "SliderRangeTitle",
            "Range",
            348.0,
            96.0,
            160.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "SliderStateTitle",
            "States",
            672.0,
            96.0,
            130.0,
            18.0,
            11.0,
            "",
        ),
        slider(
            "WorkbenchInputSliderExposure",
            "Exposure",
            "+0.75",
            0.75,
            40.0,
            124.0,
            248.0,
            44.0,
            SliderState::Normal,
            SliderVariant::Plain,
        ),
        slider(
            "WorkbenchStepsSliderQuality",
            "Quality",
            "4",
            0.66,
            40.0,
            184.0,
            248.0,
            44.0,
            SliderState::Hovered,
            SliderVariant::Steps(6),
        ),
        slider(
            "WorkbenchRangeSliderLOD",
            "LOD",
            "0.82",
            0.82,
            350.0,
            124.0,
            260.0,
            78.0,
            SliderState::Normal,
            SliderVariant::Range(0.25),
        ),
        slider(
            "WorkbenchRangeSliderCull",
            "Cull",
            "0.60",
            0.6,
            350.0,
            212.0,
            260.0,
            52.0,
            SliderState::Hovered,
            SliderVariant::Range(0.15),
        ),
        slider(
            "WorkbenchSliderFocused",
            "",
            "Focus",
            0.45,
            678.0,
            126.0,
            178.0,
            38.0,
            SliderState::Focused,
            SliderVariant::Plain,
        ),
        slider(
            "WorkbenchSliderPressed",
            "",
            "Drag",
            0.62,
            678.0,
            182.0,
            178.0,
            38.0,
            SliderState::Pressed,
            SliderVariant::Plain,
        ),
        slider(
            "WorkbenchSliderDisabled",
            "",
            "Mute",
            0.35,
            678.0,
            238.0,
            178.0,
            38.0,
            SliderState::Disabled,
            SliderVariant::Plain,
        ),
    ]
}

#[derive(Clone, Copy)]
enum SliderState {
    Normal,
    Hovered,
    Focused,
    Pressed,
    Disabled,
}

#[derive(Clone, Copy)]
enum SliderVariant {
    Plain,
    Steps(usize),
    Range(f32),
}

fn slider(
    control_id: &str,
    label_text: &str,
    value_text: &str,
    value_percent: f32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: SliderState,
    variant: SliderVariant,
) -> TemplatePaneNodeData {
    let (range_min, ticks) = match variant {
        SliderVariant::Plain => (0.0, 0.0),
        SliderVariant::Steps(tick_count) => (0.0, tick_count as f32),
        SliderVariant::Range(min_percent) => (min_percent, 0.0),
    };

    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "RangeField".into(),
        component_role: "range-field".into(),
        label_text: label_text.into(),
        value_text: value_text.into(),
        value_percent,
        layout_second_cell_offset_x: range_min,
        layout_third_cell_offset_x: ticks,
        hovered: matches!(state, SliderState::Hovered),
        focused: matches!(state, SliderState::Focused),
        pressed: matches!(state, SliderState::Pressed),
        disabled: matches!(state, SliderState::Disabled),
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
    let index = ((y as usize * SLIDER_ATLAS_WIDTH as usize) + x as usize) * 4;
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
            let index = ((py as usize * SLIDER_ATLAS_WIDTH as usize) + px as usize) * 4;
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
