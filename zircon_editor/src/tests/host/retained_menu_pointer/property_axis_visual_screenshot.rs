use std::path::{Path, PathBuf};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData, TemplatePaneNodeData,
};

const PROPERTY_AXIS_COMPONENT_SCREENSHOT: &str = "editor-components-property-axis-900x360.png";
const PROPERTY_AXIS_ATLAS_WIDTH: u32 = 900;
const PROPERTY_AXIS_ATLAS_HEIGHT: u32 = 360;
const PROPERTY_AXIS_ATLAS_BACKGROUND: [u8; 4] = [17, 20, 22, 255];

#[test]
fn property_axis_component_visual_paints_property_rows_transform_axes_and_states() {
    let bytes = property_axis_component_bytes();

    let property_panel = pixel_at(&bytes, 24, 84);
    assert_ne!(
        property_panel, PROPERTY_AXIS_ATLAS_BACKGROUND,
        "property panel should paint a visible surface"
    );

    let scalar_field = pixel_at(&bytes, 200, 176);
    assert_ne!(
        scalar_field, property_panel,
        "scalar property should paint a recessed value field"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            44,
            166,
            266,
            20,
            &[PROPERTY_AXIS_ATLAS_BACKGROUND, property_panel, scalar_field],
        ) > 0,
        "scalar property row should paint label and value text"
    );

    let focused_field_border = pixel_at(&bytes, 130, 176);
    assert_ne!(
        focused_field_border, scalar_field,
        "focused scalar property should expose a distinct value-field border"
    );

    assert!(
        distinct_pixel_count(
            &bytes,
            122,
            204,
            188,
            28,
            &[PROPERTY_AXIS_ATLAS_BACKGROUND, property_panel],
        ) > 12,
        "vector property should paint XYZ labels and compact value fields"
    );

    let axis_panel = pixel_at(&bytes, 366, 84);
    let axis_field = pixel_at(&bytes, 406, 136);
    assert_ne!(
        axis_field, axis_panel,
        "transform axis value field should paint its own recessed surface"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            380,
            126,
            214,
            22,
            &[PROPERTY_AXIS_ATLAS_BACKGROUND, axis_panel, axis_field],
        ) > 0,
        "transform position row should paint axis labels and values"
    );

    assert!(
        distinct_pixel_count(
            &bytes,
            382,
            204,
            34,
            28,
            &[PROPERTY_AXIS_ATLAS_BACKGROUND, axis_panel],
        ) > 0,
        "scale link icon should paint lobe and connector geometry"
    );

    let state_panel = pixel_at(&bytes, 640, 84);
    assert!(
        distinct_pixel_count(
            &bytes,
            660,
            126,
            196,
            134,
            &[PROPERTY_AXIS_ATLAS_BACKGROUND, state_panel],
        ) > 0,
        "state panel should paint selected, focused, and disabled property/axis controls"
    );

    let focused_scalar_border = pixel_at(&bytes, 810, 159);
    let selected_scalar_border = pixel_at(&bytes, 810, 125);
    assert_ne!(
        selected_scalar_border, focused_scalar_border,
        "selected scalar value field should not borrow the focused scalar border"
    );
    assert!(
        color_count(&bytes, 746, 125, 107, 24, selected_scalar_border) > 0,
        "selected scalar value field should paint its own neutral border"
    );
    assert_eq!(
        color_count(&bytes, 746, 125, 107, 24, focused_scalar_border),
        0,
        "selected scalar value field should not borrow the focus-ring border"
    );

    let selected_axis_border = pixel_at(&bytes, 754, 203);
    let selected_axis_fill = pixel_at(&bytes, 784, 216);
    let focused_axis_border = pixel_at(&bytes, 754, 237);
    let focused_axis_fill = pixel_at(&bytes, 784, 250);
    assert!(
        color_count(&bytes, 718, 202, 72, 28, selected_axis_fill) > 0,
        "selected axis value field should use hover-state background"
    );
    assert!(
        color_count(&bytes, 718, 202, 72, 28, selected_axis_border) > 0,
        "selected axis value field should use hover-state border"
    );
    assert_ne!(
        selected_axis_border, focused_axis_border,
        "selected axis value field should not reuse the focused axis border"
    );
    assert_eq!(
        color_count(&bytes, 718, 202, 72, 28, focused_axis_border),
        0,
        "selected axis value field should not use the focused axis border"
    );

    let normal_axis_fill = pixel_at(&bytes, 456, 140);
    assert!(
        color_count(&bytes, 718, 236, 72, 28, normal_axis_fill) > 0,
        "focused-only axis value field should keep the normal recessed background"
    );
    assert!(
        color_count(&bytes, 718, 236, 72, 28, focused_axis_border) > 0,
        "focused-only axis value field should express focus through the border"
    );
    assert_eq!(
        focused_axis_fill, normal_axis_fill,
        "focused-only axis value field should not switch to the selected/hover fill"
    );
}

#[test]
#[ignore = "writes local property/axis component screenshot artifact for visual review"]
fn capture_property_axis_component_visual_artifact() {
    let bytes = property_axis_component_bytes();
    let output_path = visual_layout_output_path(PROPERTY_AXIS_COMPONENT_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        PROPERTY_AXIS_ATLAS_WIDTH,
        PROPERTY_AXIS_ATLAS_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("property/axis component screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn property_axis_component_bytes() -> Vec<u8> {
    paint_template_nodes_for_test_with_background(
        PROPERTY_AXIS_ATLAS_WIDTH,
        PROPERTY_AXIS_ATLAS_HEIGHT,
        PROPERTY_AXIS_ATLAS_BACKGROUND,
        model_rc(property_axis_component_nodes()),
    )
}

fn property_axis_component_nodes() -> Vec<TemplatePaneNodeData> {
    let mut nodes = vec![
        surface("PropertyAxisRoot", "shell", 0.0, 0.0, 900.0, 360.0),
        label(
            "PropertyAxisTitle",
            "Inspector Properties",
            22.0,
            20.0,
            260.0,
            22.0,
            13.0,
            "",
        ),
        label(
            "PropertyAxisSubtitle",
            "Property rows, XYZ value groups and transform axis controls use retained painters",
            22.0,
            42.0,
            690.0,
            18.0,
            10.0,
            "muted",
        ),
        surface("PropertyRowsPanel", "panel", 18.0, 78.0, 318.0, 224.0),
        surface("TransformAxisPanel", "panel", 360.0, 78.0, 250.0, 224.0),
        surface("PropertyStatePanel", "inset", 634.0, 78.0, 248.0, 224.0),
        label(
            "PropertyRowsTitle",
            "Rows",
            38.0,
            96.0,
            120.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "TransformAxisTitle",
            "Transform",
            380.0,
            96.0,
            120.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "PropertyStateTitle",
            "States",
            654.0,
            96.0,
            120.0,
            18.0,
            11.0,
            "",
        ),
        property_row(
            "WorkbenchMeshRow",
            "Material",
            "MI_RustedPanel",
            38.0,
            122.0,
            278.0,
            30.0,
            PropertyState::Normal,
        ),
        property_row(
            "WorkbenchComponentPropertyVirtualRowRoughness",
            "Roughness",
            "0.42",
            38.0,
            162.0,
            278.0,
            30.0,
            PropertyState::Focused,
        ),
        property_row(
            "WorkbenchComponentPropertyVirtualRowLocation",
            "Location",
            "X 128.0 Y -32.0 Z 4.0",
            38.0,
            202.0,
            278.0,
            34.0,
            PropertyState::Normal,
        ),
        label(
            "PropertyRowsCopy",
            "Name slot and value slot stay aligned like Slate detail rows",
            38.0,
            252.0,
            276.0,
            28.0,
            10.0,
            "muted",
        ),
        label(
            "PropertyStateCopy",
            "Focused borders stay separate from disabled muted fields",
            654.0,
            224.0,
            210.0,
            34.0,
            10.0,
            "muted",
        ),
    ];

    push_transform_axis_row(&mut nodes, "Position", 126.0, ["128.0", "-32.0", "4.0"]);
    push_transform_axis_row(&mut nodes, "Rotation", 164.0, ["0.0", "45.0", "0.0"]);
    push_scale_axis_row(&mut nodes, 202.0, ["1.00", "1.00", "1.00"]);

    nodes.push(property_row(
        "WorkbenchComponentPropertyVirtualRowSelected",
        "Selected",
        "1.0",
        654.0,
        122.0,
        204.0,
        30.0,
        PropertyState::Selected,
    ));
    nodes.push(property_row(
        "WorkbenchComponentPropertyVirtualRowFocused",
        "Focused",
        "3.2",
        654.0,
        156.0,
        204.0,
        30.0,
        PropertyState::Focused,
    ));
    nodes.push(axis_label(
        "AxisSelectedLabel",
        "Selected",
        654.0,
        204.0,
        58.0,
        16.0,
        AxisState::Normal,
    ));
    nodes.push(axis_value_field(
        "WorkbenchAxisValueFieldSelected",
        "2.50",
        718.0,
        202.0,
        72.0,
        28.0,
        AxisState::Selected,
    ));
    nodes.push(axis_label(
        "AxisFocusedLabel",
        "Focused",
        654.0,
        238.0,
        58.0,
        16.0,
        AxisState::Normal,
    ));
    nodes.push(axis_value_field(
        "WorkbenchAxisValueFieldFocused",
        "2.718",
        718.0,
        236.0,
        72.0,
        28.0,
        AxisState::Focused,
    ));
    nodes.push(axis_label(
        "AxisDisabledLabel",
        "Disabled",
        654.0,
        272.0,
        58.0,
        16.0,
        AxisState::Disabled,
    ));
    nodes.push(axis_value_field(
        "WorkbenchAxisValueFieldDisabled",
        "locked",
        718.0,
        270.0,
        72.0,
        28.0,
        AxisState::Disabled,
    ));
    nodes
}

#[derive(Clone, Copy)]
enum PropertyState {
    Normal,
    Focused,
    Selected,
}

#[derive(Clone, Copy)]
enum AxisState {
    Normal,
    Focused,
    Disabled,
    Selected,
}

fn push_transform_axis_row(
    nodes: &mut Vec<TemplatePaneNodeData>,
    kind: &str,
    y: f32,
    values: [&str; 3],
) {
    nodes.push(label(
        &format!("Transform{kind}Label"),
        kind,
        380.0,
        y - 18.0,
        86.0,
        16.0,
        10.0,
        "muted",
    ));
    push_axis_group(nodes, kind, y, values, AxisState::Normal);
}

fn push_scale_axis_row(nodes: &mut Vec<TemplatePaneNodeData>, y: f32, values: [&str; 3]) {
    nodes.push(label(
        "TransformScaleLabel",
        "Scale",
        424.0,
        y - 18.0,
        86.0,
        16.0,
        10.0,
        "muted",
    ));
    nodes.push(scale_link(
        "WorkbenchTransformScaleLink",
        380.0,
        y,
        34.0,
        28.0,
    ));
    push_axis_group(nodes, "Scale", y, values, AxisState::Normal);
}

fn push_axis_group(
    nodes: &mut Vec<TemplatePaneNodeData>,
    kind: &str,
    y: f32,
    values: [&str; 3],
    state: AxisState,
) {
    let axis_specs = [
        ("X", 380.0, 404.0, values[0]),
        ("Y", 472.0, 496.0, values[1]),
        ("Z", 564.0, 588.0, values[2]),
    ];
    for (axis, label_x, field_x, value) in axis_specs {
        nodes.push(axis_label(
            &format!("WorkbenchTransform{kind}Axis{axis}"),
            axis,
            label_x,
            y,
            18.0,
            28.0,
            state,
        ));
        nodes.push(axis_value_field(
            &format!("WorkbenchTransform{kind}{axis}"),
            value,
            field_x,
            y,
            62.0,
            28.0,
            state,
        ));
    }
}

fn property_row(
    control_id: &str,
    label_text: &str,
    value: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: PropertyState,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "PropertyRow".into(),
        component_role: "property-row".into(),
        text: label_text.into(),
        value_text: value.into(),
        focused: matches!(state, PropertyState::Focused),
        pressed: false,
        selected: matches!(state, PropertyState::Selected),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn axis_label(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: AxisState,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        disabled: matches!(state, AxisState::Disabled),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn axis_value_field(
    control_id: &str,
    value: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: AxisState,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "InputField".into(),
        component_role: "axis-value-field".into(),
        value_text: value.into(),
        focused: matches!(state, AxisState::Focused),
        disabled: matches!(state, AxisState::Disabled),
        selected: matches!(state, AxisState::Selected),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn scale_link(control_id: &str, x: f32, y: f32, width: f32, height: f32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Icon".into(),
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
    let index = ((y as usize * PROPERTY_AXIS_ATLAS_WIDTH as usize) + x as usize) * 4;
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
            let index = ((py as usize * PROPERTY_AXIS_ATLAS_WIDTH as usize) + px as usize) * 4;
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

fn color_count(bytes: &[u8], x: u32, y: u32, width: u32, height: u32, target: [u8; 4]) -> usize {
    let mut count = 0;
    for py in y..(y + height) {
        for px in x..(x + width) {
            let index = ((py as usize * PROPERTY_AXIS_ATLAS_WIDTH as usize) + px as usize) * 4;
            let color = [
                bytes[index],
                bytes[index + 1],
                bytes[index + 2],
                bytes[index + 3],
            ];
            if color == target {
                count += 1;
            }
        }
    }
    count
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
