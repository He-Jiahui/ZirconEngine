use std::path::{Path, PathBuf};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData, TemplatePaneNodeData,
};
use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

const VIEWPORT_SCENE_COMPONENT_SCREENSHOT: &str = "editor-components-viewport-scene-900x360.png";
const VIEWPORT_SCENE_ATLAS_WIDTH: u32 = 900;
const VIEWPORT_SCENE_ATLAS_HEIGHT: u32 = 360;
const VIEWPORT_SCENE_ATLAS_BACKGROUND: [u8; 4] = [17, 20, 22, 255];
const VIEWPORT_STAGE: [u8; 4] = [28, 34, 38, 255];

#[test]
fn viewport_scene_visual_keeps_handrail_posts_visible_at_wide_and_narrow_widths() {
    let bytes = viewport_scene_component_bytes();

    let wide_stage = pixel_at(&bytes, 96, 166);
    let narrow_stage = pixel_at(&bytes, 506, 166);

    assert_eq!(wide_stage, VIEWPORT_STAGE);
    assert_eq!(narrow_stage, VIEWPORT_STAGE);
    assert_ne!(pixel_at(&bytes, 175, 166), wide_stage);
    assert_ne!(pixel_at(&bytes, 246, 166), wide_stage);
    assert_ne!(pixel_at(&bytes, 523, 166), narrow_stage);
    assert_ne!(pixel_at(&bytes, 543, 166), narrow_stage);
}

#[test]
#[ignore = "writes a local viewport scene component screenshot artifact for visual review"]
fn capture_viewport_scene_component_visual_artifact() {
    let bytes = viewport_scene_component_bytes();
    let output_path = visual_layout_output_path(VIEWPORT_SCENE_COMPONENT_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        VIEWPORT_SCENE_ATLAS_WIDTH,
        VIEWPORT_SCENE_ATLAS_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("viewport scene screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn viewport_scene_component_bytes() -> Vec<u8> {
    paint_template_nodes_for_test_with_background(
        VIEWPORT_SCENE_ATLAS_WIDTH,
        VIEWPORT_SCENE_ATLAS_HEIGHT,
        VIEWPORT_SCENE_ATLAS_BACKGROUND,
        model_rc(viewport_scene_component_nodes()),
    )
}

fn viewport_scene_component_nodes() -> Vec<TemplatePaneNodeData> {
    vec![
        surface(
            "ViewportSceneRoot",
            0.0,
            0.0,
            900.0,
            360.0,
            [20, 24, 27, 255],
        ),
        label(
            "ViewportSceneTitle",
            "Viewport Scene Geometry",
            22.0,
            20.0,
            320.0,
            22.0,
            13.0,
            "",
        ),
        label(
            "ViewportSceneSubtitle",
            "Rail posts preserve their Slate-like composition across responsive scene widths",
            22.0,
            42.0,
            680.0,
            18.0,
            10.0,
            "muted",
        ),
        surface(
            "ViewportSceneWideStage",
            18.0,
            78.0,
            410.0,
            226.0,
            VIEWPORT_STAGE,
        ),
        surface(
            "ViewportSceneNarrowStage",
            450.0,
            78.0,
            432.0,
            226.0,
            VIEWPORT_STAGE,
        ),
        label(
            "ViewportSceneWideLabel",
            "Wide viewport",
            38.0,
            96.0,
            180.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "ViewportSceneNarrowLabel",
            "Compact viewport",
            470.0,
            96.0,
            180.0,
            18.0,
            11.0,
            "",
        ),
        handrail("WorkbenchViewportHandrailWide", 58.0, 142.0, 320.0),
        handrail("WorkbenchViewportHandrailNarrow", 490.0, 142.0, 88.0),
    ]
}

fn handrail(control_id: &str, x: f32, y: f32, width: f32) -> TemplatePaneNodeData {
    surface(control_id, x, y, width, 4.0, [179, 113, 48, 122])
}

fn surface(
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    background: [u8; 4],
) -> TemplatePaneNodeData {
    let mut node = TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Pane".into(),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    };
    node.button_style.element.background_color = Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(
        background[0],
        background[1],
        background[2],
        background[3],
    )));
    node
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
    let index = ((y as usize * VIEWPORT_SCENE_ATLAS_WIDTH as usize) + x as usize) * 4;
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
