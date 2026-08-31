use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::geometry::{field_paint_rect, field_surface_radius};

#[test]
fn workbench_field_preserves_fractional_declared_bounds() {
    let declared = FrameRect {
        x: 12.3,
        y: 8.4,
        width: 67.2,
        height: 30.5,
    };
    let rect = field_paint_rect(&TemplatePaneNodeData::default(), &declared);

    assert_eq!(rect.x, 12.3);
    assert_eq!(rect.y, 8.4);
    assert_eq!(rect.width, 67.2);
    assert_eq!(rect.height, 30.5);
}

#[test]
fn workbench_field_surface_radius_stays_inside_a_narrow_control() {
    let rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 0.5,
        height: 32.0,
    };

    assert_eq!(field_surface_radius(&rect, 4.0), 0.25);
}
