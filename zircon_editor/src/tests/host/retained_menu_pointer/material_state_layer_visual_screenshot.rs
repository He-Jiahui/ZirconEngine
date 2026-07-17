use std::path::{Path, PathBuf};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData, TemplatePaneNodeData,
};

const MATERIAL_STATE_LAYER_SCREENSHOT: &str = "editor-components-material-state-layer-900x360.png";
const MATERIAL_STATE_LAYER_WIDTH: u32 = 900;
const MATERIAL_STATE_LAYER_HEIGHT: u32 = 360;
const MATERIAL_STATE_LAYER_BACKGROUND: [u8; 4] = [17, 20, 22, 255];
const OUTER_INSET: f32 = 18.0;
const PANEL_GAP: f32 = 12.0;
const PANEL_TOP: f32 = 78.0;
const PANEL_HEIGHT: f32 = 230.0;

#[test]
fn material_state_layer_visual_separates_hover_focus_press_and_drag_priority() {
    let enabled_bytes = material_state_layer_bytes(true);
    let disabled_bytes = material_state_layer_bytes(false);
    let mut enabled_samples = [[0; 4]; MaterialVisualState::ALL.len()];

    for (index, state) in MaterialVisualState::ALL.into_iter().enumerate() {
        let enabled_sample = sample_center(&enabled_bytes, index);
        let disabled_sample = sample_center(&disabled_bytes, index);
        assert_ne!(
            enabled_sample,
            disabled_sample,
            "{} should differ from an otherwise identical state-layer-disabled baseline",
            state.label()
        );
        enabled_samples[index] = enabled_sample;
    }
    let [hover, focus, press, drag] = enabled_samples;

    assert_ne!(
        hover, focus,
        "hover and keyboard focus should paint distinct Material state layers"
    );
    assert_ne!(
        press, focus,
        "pressed should stay visually above keyboard focus"
    );
    assert_ne!(
        drag, focus,
        "dragging should stay visually above keyboard focus"
    );
    assert_ne!(
        drag, hover,
        "dragging should paint more strongly than hover"
    );
}

#[test]
#[ignore = "writes local Material state-layer screenshot artifact for visual review"]
fn capture_material_state_layer_visual_artifact() {
    let bytes = material_state_layer_bytes(true);
    let output_path = visual_layout_output_path(MATERIAL_STATE_LAYER_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        MATERIAL_STATE_LAYER_WIDTH,
        MATERIAL_STATE_LAYER_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("Material state-layer screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn material_state_layer_bytes(state_layer_enabled: bool) -> Vec<u8> {
    paint_template_nodes_for_test_with_background(
        MATERIAL_STATE_LAYER_WIDTH,
        MATERIAL_STATE_LAYER_HEIGHT,
        MATERIAL_STATE_LAYER_BACKGROUND,
        model_rc(material_state_layer_nodes(state_layer_enabled)),
    )
}

fn material_state_layer_nodes(state_layer_enabled: bool) -> Vec<TemplatePaneNodeData> {
    let mut nodes = vec![
        surface(
            "MaterialStateLayerRoot",
            "shell",
            frame(
                0.0,
                0.0,
                MATERIAL_STATE_LAYER_WIDTH as f32,
                MATERIAL_STATE_LAYER_HEIGHT as f32,
            ),
        ),
        label(
            "MaterialStateLayerTitle",
            "Material State Priority",
            frame(OUTER_INSET + 4.0, 20.0, 320.0, 22.0),
            13.0,
            "",
        ),
        label(
            "MaterialStateLayerSubtitle",
            "Pressed and dragging stay above keyboard focus",
            frame(OUTER_INSET + 4.0, 42.0, 520.0, 18.0),
            10.0,
            "muted",
        ),
    ];

    for (index, sample) in MaterialVisualState::ALL.into_iter().enumerate() {
        let id_stem = sample.id_stem();
        nodes.extend([
            surface(&format!("{id_stem}Panel"), "panel", sample_frame(index)),
            label(
                &format!("{id_stem}Label"),
                sample.label(),
                sample_label_frame(index),
                11.0,
                "",
            ),
            state_button(
                &format!("{id_stem}Button"),
                "Action",
                sample,
                state_layer_enabled,
                state_button_frame(index),
            ),
            label(
                &format!("{id_stem}Description"),
                sample.description(),
                sample_description_frame(index),
                10.0,
                "muted",
            ),
        ]);
    }

    nodes
}

#[derive(Clone, Copy)]
enum MaterialVisualState {
    Hovered,
    Focused,
    PressedFocused,
    DraggingFocused,
}

impl MaterialVisualState {
    const ALL: [Self; 4] = [
        Self::Hovered,
        Self::Focused,
        Self::PressedFocused,
        Self::DraggingFocused,
    ];

    const fn id_stem(self) -> &'static str {
        match self {
            Self::Hovered => "MaterialStateHovered",
            Self::Focused => "MaterialStateFocused",
            Self::PressedFocused => "MaterialStatePressedFocused",
            Self::DraggingFocused => "MaterialStateDraggingFocused",
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Hovered => "Hovered",
            Self::Focused => "Focused",
            Self::PressedFocused => "Pressed + Focused",
            Self::DraggingFocused => "Dragging + Focused",
        }
    }

    const fn description(self) -> &'static str {
        match self {
            Self::Hovered => "Hover layer 0.08",
            Self::Focused => "Focus layer 0.10",
            Self::PressedFocused => "Pressed wins, 0.10",
            Self::DraggingFocused => "Drag wins, 0.16",
        }
    }
}

fn state_button(
    control_id: &str,
    text: &str,
    sample: MaterialVisualState,
    state_layer_enabled: bool,
    frame: TemplateNodeFrameData,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Button".into(),
        component_role: "button".into(),
        text: text.into(),
        button_variant: "outlined".into(),
        state_layer_enabled,
        hovered: matches!(sample, MaterialVisualState::Hovered),
        focused: matches!(
            sample,
            MaterialVisualState::Focused
                | MaterialVisualState::PressedFocused
                | MaterialVisualState::DraggingFocused
        ),
        pressed: matches!(sample, MaterialVisualState::PressedFocused),
        dragging: matches!(sample, MaterialVisualState::DraggingFocused),
        frame,
        ..TemplatePaneNodeData::default()
    }
}

fn surface(control_id: &str, variant: &str, frame: TemplateNodeFrameData) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Panel".into(),
        surface_variant: variant.into(),
        border_width: 1.0,
        corner_radius: 6.0,
        frame,
        ..TemplatePaneNodeData::default()
    }
}

fn label(
    control_id: &str,
    text: &str,
    frame: TemplateNodeFrameData,
    font_size: f32,
    tone: &str,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        font_size,
        text_tone: tone.into(),
        frame,
        ..TemplatePaneNodeData::default()
    }
}

fn sample_frame(index: usize) -> TemplateNodeFrameData {
    let available = MATERIAL_STATE_LAYER_WIDTH as f32 - OUTER_INSET * 2.0;
    let width = (available - PANEL_GAP * 3.0) / 4.0;
    let x = OUTER_INSET + index as f32 * (width + PANEL_GAP);
    frame(x, PANEL_TOP, width, PANEL_HEIGHT)
}

fn sample_label_frame(index: usize) -> TemplateNodeFrameData {
    let panel = sample_frame(index);
    frame(panel.x + 18.0, panel.y + 18.0, panel.width - 36.0, 20.0)
}

fn state_button_frame(index: usize) -> TemplateNodeFrameData {
    let panel = sample_frame(index);
    frame(panel.x + 18.0, panel.y + 66.0, panel.width - 36.0, 56.0)
}

fn sample_description_frame(index: usize) -> TemplateNodeFrameData {
    let panel = sample_frame(index);
    frame(panel.x + 18.0, panel.y + 142.0, panel.width - 36.0, 36.0)
}

fn state_button_sample_point(index: usize) -> (u32, u32) {
    let button = state_button_frame(index);
    (
        (button.x + button.width * 0.25) as u32,
        (button.y + button.height * 0.5) as u32,
    )
}

fn sample_center(bytes: &[u8], index: usize) -> [u8; 4] {
    let (x, y) = state_button_sample_point(index);
    pixel_at(bytes, x, y)
}

fn node_id(control_id: &str) -> String {
    format!("{control_id}.node")
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
    let index = ((y as usize * MATERIAL_STATE_LAYER_WIDTH as usize) + x as usize) * 4;
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
