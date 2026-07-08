use std::path::{Path, PathBuf};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::primitives::Color;
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData, TemplatePaneNodeData,
};

const TRANSFORM_CONTROLS_COMPONENT_SCREENSHOT: &str =
    "editor-components-transform-controls-900x360.png";
const TRANSFORM_ATLAS_WIDTH: u32 = 900;
const TRANSFORM_ATLAS_HEIGHT: u32 = 360;
const TRANSFORM_ATLAS_BACKGROUND: [u8; 4] = [17, 20, 22, 255];

#[test]
fn transform_controls_component_visual_paints_axis_labels_fields_and_scale_link() {
    let bytes = transform_controls_component_bytes();

    assert!(
        changed_pixel_count(&bytes, 92, 126, 24, 28) > 0,
        "Position X axis label should render visible pixels"
    );
    assert!(
        changed_pixel_count(&bytes, 124, 126, 224, 28) > 0,
        "Position axis value fields should render compact field pixels"
    );
    assert!(
        changed_pixel_count(&bytes, 92, 246, 24, 28) > 0,
        "Scale-link axis label glyph should render visible pixels"
    );
}

#[test]
#[ignore = "writes local transform component screenshot artifact for visual review"]
fn capture_transform_controls_component_visual_artifact() {
    let bytes = transform_controls_component_bytes();
    let output_path = visual_layout_output_path(TRANSFORM_CONTROLS_COMPONENT_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        TRANSFORM_ATLAS_WIDTH,
        TRANSFORM_ATLAS_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("transform component screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn transform_controls_component_bytes() -> Vec<u8> {
    paint_template_nodes_for_test_with_background(
        TRANSFORM_ATLAS_WIDTH,
        TRANSFORM_ATLAS_HEIGHT,
        TRANSFORM_ATLAS_BACKGROUND,
        model_rc(transform_controls_component_nodes()),
    )
}

fn transform_controls_component_nodes() -> Vec<TemplatePaneNodeData> {
    let mut nodes = vec![
        surface("TransformRoot", "shell", 0.0, 0.0, 900.0, 360.0),
        label(
            "TransformTitle",
            "Transform Controls",
            22.0,
            20.0,
            260.0,
            22.0,
            13.0,
            "",
        ),
        label(
            "TransformSubtitle",
            "Axis labels, compact fields and Scale-link use retained painters",
            22.0,
            42.0,
            520.0,
            18.0,
            10.0,
            "muted",
        ),
        surface("TransformPanel", "panel", 18.0, 78.0, 410.0, 246.0),
        surface("TransformPreviewPanel", "inset", 454.0, 78.0, 428.0, 246.0),
        label(
            "TransformPanelTitle",
            "Details Panel",
            36.0,
            96.0,
            160.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "TransformPreviewTitle",
            "Interactive States",
            472.0,
            96.0,
            180.0,
            18.0,
            11.0,
            "",
        ),
    ];

    nodes.extend(transform_row(
        "Position",
        "Location",
        128.0,
        ["-12.50", "84.00", "192.25"],
        [false, false, false],
        [false, false, false],
    ));
    nodes.extend(transform_row(
        "Rotation",
        "Rotation",
        178.0,
        ["0.00", "90.00", "0.00"],
        [false, true, false],
        [false, false, false],
    ));
    nodes.extend(transform_row(
        "Scale",
        "Scale",
        228.0,
        ["1.00", "1.00", "1.00"],
        [false, false, false],
        [false, false, true],
    ));
    nodes.push(scale_link_label(92.0, 246.0));

    nodes.extend([
        label(
            "TransformPreviewCopy",
            "Focused Y uses cyan; disabled Z stays muted.",
            472.0,
            128.0,
            352.0,
            20.0,
            11.0,
            "muted",
        ),
        axis_value_field(
            "WorkbenchTransformRotationY",
            "90.00",
            472.0,
            164.0,
            106.0,
            true,
            false,
        ),
        axis_value_field(
            "WorkbenchTransformScaleZ",
            "1.00",
            592.0,
            164.0,
            106.0,
            false,
            true,
        ),
        axis_label(
            "WorkbenchTransformPositionAxisX",
            "X",
            472.0,
            218.0,
            Color::from_rgb_u8(226, 82, 82),
        ),
        axis_label(
            "WorkbenchTransformRotationAxisY",
            "Y",
            510.0,
            218.0,
            Color::from_rgb_u8(92, 198, 112),
        ),
        axis_label(
            "WorkbenchTransformScaleAxisZ",
            "Z",
            548.0,
            218.0,
            Color::from_rgb_u8(88, 152, 231),
        ),
        scale_link_label(596.0, 218.0),
    ]);

    nodes
}

fn transform_row(
    family: &str,
    title: &str,
    y: f32,
    values: [&str; 3],
    focused: [bool; 3],
    disabled: [bool; 3],
) -> Vec<TemplatePaneNodeData> {
    let label_y = y + 4.0;
    let field_y = y;
    vec![
        label(
            &format!("Transform{family}RowTitle"),
            title,
            36.0,
            label_y,
            78.0,
            18.0,
            11.0,
            "",
        ),
        axis_label(
            &format!("WorkbenchTransform{family}AxisX"),
            "X",
            92.0,
            field_y,
            Color::from_rgb_u8(226, 82, 82),
        ),
        axis_value_field(
            &format!("WorkbenchTransform{family}X"),
            values[0],
            124.0,
            field_y,
            72.0,
            focused[0],
            disabled[0],
        ),
        axis_label(
            &format!("WorkbenchTransform{family}AxisY"),
            "Y",
            208.0,
            field_y,
            Color::from_rgb_u8(92, 198, 112),
        ),
        axis_value_field(
            &format!("WorkbenchTransform{family}Y"),
            values[1],
            240.0,
            field_y,
            72.0,
            focused[1],
            disabled[1],
        ),
        axis_label(
            &format!("WorkbenchTransform{family}AxisZ"),
            "Z",
            324.0,
            field_y,
            Color::from_rgb_u8(88, 152, 231),
        ),
        axis_value_field(
            &format!("WorkbenchTransform{family}Z"),
            values[2],
            356.0,
            field_y,
            50.0,
            focused[2],
            disabled[2],
        ),
    ]
}

fn axis_label(control_id: &str, text: &str, x: f32, y: f32, color: Color) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        label_color: color,
        frame: frame(x, y, 18.0, 28.0),
        ..TemplatePaneNodeData::default()
    }
}

fn scale_link_label(x: f32, y: f32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: "WorkbenchTransformScaleLink".into(),
        role: "Icon".into(),
        frame: frame(x, y, 22.0, 28.0),
        ..TemplatePaneNodeData::default()
    }
}

fn axis_value_field(
    control_id: &str,
    value: &str,
    x: f32,
    y: f32,
    width: f32,
    focused: bool,
    disabled: bool,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "InputField".into(),
        component_role: "input-field".into(),
        value_text: value.into(),
        focused,
        disabled,
        frame: frame(x, y, width, 28.0),
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
            let index = ((py as usize * TRANSFORM_ATLAS_WIDTH as usize) + px as usize) * 4;
            if bytes[index..index + 4] != TRANSFORM_ATLAS_BACKGROUND {
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
