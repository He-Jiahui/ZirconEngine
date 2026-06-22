use super::super::super::super::data::FrameRect;
use super::super::super::template_inspector_row_geometry::{
    nested_label_rect, nested_select_field_rect,
};

#[test]
fn nested_lighting_select_preserves_right_edge_with_select_indent() {
    let rect = FrameRect {
        x: 8.0,
        y: 8.0,
        width: 304.0,
        height: 28.0,
    };

    let label = nested_label_rect(&rect);
    let field = nested_select_field_rect(&rect);

    assert_eq!(label.x, 22.0);
    assert_eq!(field.x, 162.0);
    assert_eq!(field.width, 150.0);
    assert_eq!(field.x + field.width, rect.x + rect.width);
}
