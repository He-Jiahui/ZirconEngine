use super::super::super::data::{TemplateNodeFrameData, TemplatePaneNodeData};
use super::super::template_nodes::paint_template_nodes_for_test;
use super::*;
use crate::ui::layouts::common::model_rc;

#[test]
fn workbench_chip_matches_viewport_chips_but_not_status_chips() {
    assert!(is_workbench_chip(&chip_node(
        "WorkbenchViewportMode",
        "Perspective",
    )));
    assert!(is_workbench_chip(&chip_node("WorkbenchChipRoot", "Chip")));
    assert!(!is_workbench_chip(&chip_node(
        "WorkbenchStatusGrid",
        "Grid: 10 cm",
    )));
}

#[test]
fn viewport_chip_paints_surface_border_text_and_chevron() {
    let bytes = paint_template_nodes_for_test(
        150,
        48,
        model_rc(vec![chip_node("WorkbenchViewportMode", "Perspective")]),
    );

    assert_eq!(pixel_at(&bytes, 150, 110, 24), CHIP_SURFACE);
    assert_eq!(pixel_at(&bytes, 150, 54, 8), CHIP_BORDER);
    assert!(changed_pixel_count(&bytes, 150, 22, 16, 62, 18) > 0);
    assert!(changed_pixel_count(&bytes, 150, 102, 15, 18, 18) > 0);
}

#[test]
fn focused_chip_uses_focus_border() {
    let mut node = chip_node("WorkbenchViewportAngle", "10 deg");
    node.focused = true;
    let bytes = paint_template_nodes_for_test(120, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 120, 54, 8), PALETTE.focus_ring);
}

fn chip_node(control_id: &str, text: &str) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        frame: TemplateNodeFrameData {
            x: 12.0,
            y: 8.0,
            width: 104.0,
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
