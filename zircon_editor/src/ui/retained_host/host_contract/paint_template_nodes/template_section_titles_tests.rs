use super::super::super::data::{TemplateNodeFrameData, TemplatePaneNodeData};
use super::super::template_nodes::paint_template_nodes_for_test;
use super::super::template_section_title_glyphs::{
    section_icon_color, SectionTitleIcon, SECTION_TRANSFORM_GLYPH,
};
use super::*;
use crate::ui::layouts::common::model_rc;

#[test]
fn workbench_section_title_matches_titles_without_row_labels() {
    assert!(is_workbench_section_title(&title_node(
        "WorkbenchButtonsTitle",
        "Buttons"
    )));
    assert!(is_workbench_section_title(&title_node(
        "WorkbenchTransformLabel",
        "Transform"
    )));
    assert!(!is_workbench_section_title(&title_node(
        "WorkbenchTransformPositionLabel",
        "Position"
    )));
}

#[test]
fn component_drawer_section_title_paints_bold_label() {
    let bytes = paint_template_nodes_for_test(
        180,
        48,
        model_rc(vec![title_node("WorkbenchButtonsTitle", "Buttons")]),
    );

    assert!(changed_pixel_count(&bytes, 180, 18, 14, 72, 20) > 0);
    assert_eq!(pixel_at(&bytes, 180, 12, 8), [0, 0, 0, 255]);
}

#[test]
fn inspector_section_title_paints_leading_icon_and_label() {
    let bytes = paint_template_nodes_for_test(
        180,
        48,
        model_rc(vec![title_node("WorkbenchInspectorTitle", "Props")]),
    );

    assert!(changed_pixel_count(&bytes, 180, 18, 17, 18, 18) > 0);
    assert!(changed_pixel_count(&bytes, 180, 43, 14, 58, 20) > 0);
}

#[test]
fn mesh_renderer_section_title_uses_audited_title_tone() {
    assert_eq!(
        section_text_color(&title_node("WorkbenchMeshLabel", "Mesh Renderer")),
        SECTION_MESH_TEXT
    );
}

#[test]
fn section_title_uses_declared_title_tone() {
    let mut node = title_node("WorkbenchSelectionTitle", "Checkboxes & Radios");
    node.label_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(152, 163, 168);

    assert_eq!(section_text_color(&node), [152, 163, 168, 255]);
}

#[test]
fn transform_section_title_uses_audited_icon_opacity() {
    assert_eq!(
        section_icon_color(SectionTitleIcon::Transform),
        SECTION_TRANSFORM_GLYPH
    );
    assert_eq!(SECTION_TRANSFORM_GLYPH[3], 97);
}

fn title_node(control_id: &str, text: &str) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        font_weight: 700,
        frame: TemplateNodeFrameData {
            x: 10.0,
            y: 8.0,
            width: 150.0,
            height: 30.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn changed_pixel_count(
    bytes: &[u8],
    frame_width: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> usize {
    let mut changed = 0;
    for py in y..(y + height) {
        for px in x..(x + width) {
            let index = ((py as usize * frame_width as usize) + px as usize) * 4;
            if bytes[index..index + 4] != [0, 0, 0, 255] {
                changed += 1;
            }
        }
    }
    changed
}

fn pixel_at(bytes: &[u8], frame_width: u32, x: u32, y: u32) -> [u8; 4] {
    let index = ((y as usize * frame_width as usize) + x as usize) * 4;
    [
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ]
}
