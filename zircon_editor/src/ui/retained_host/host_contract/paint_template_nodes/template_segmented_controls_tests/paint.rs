use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::style::{segmented_background, SEGMENT_IDLE_BACKGROUND};
use super::support::{changed_pixel_count, labeled_segmented_node, pixel_at, segmented_node};
use crate::ui::layouts::common::model_rc;

#[test]
fn segmented_control_paints_selected_middle_segment() {
    let bytes = paint_template_nodes_for_test(180, 48, model_rc(vec![segmented_node()]));

    assert_eq!(
        segmented_background(&segmented_node()),
        SEGMENT_IDLE_BACKGROUND
    );
    assert_eq!(pixel_at(&bytes, 180, 17, 15), SEGMENT_IDLE_BACKGROUND);
    assert!(changed_pixel_count(&bytes, 180, 62, 8, 48, 22) > 0);
    assert!(changed_pixel_count(&bytes, 180, 14, 8, 40, 22) > 0);
}

#[test]
fn segmented_control_paints_group_label_and_body() {
    let bytes = paint_template_nodes_for_test(190, 60, model_rc(vec![labeled_segmented_node()]));

    assert!(changed_pixel_count(&bytes, 190, 12, 4, 132, 14) > 0);
    assert!(changed_pixel_count(&bytes, 190, 18, 22, 144, 30) > 0);
    assert_eq!(pixel_at(&bytes, 190, 12, 22), [0, 0, 0, 255]);
}
