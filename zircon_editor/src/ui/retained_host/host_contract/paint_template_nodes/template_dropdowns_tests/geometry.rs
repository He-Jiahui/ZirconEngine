use super::super::super::super::data::FrameRect;
use super::super::super::template_dropdown_metrics::workbench_dropdown_metrics;
use super::super::geometry::{
    dropdown_surface_radius, has_paintable_dropdown_extent, pixel_aligned_rect,
};

#[test]
fn workbench_dropdown_alignment_stays_inside_fractional_declared_bounds() {
    let declared = FrameRect {
        x: 12.3,
        y: 8.4,
        width: 95.2,
        height: 30.5,
    };
    let rect = pixel_aligned_rect(&declared);

    assert_eq!(rect.x, 13.0);
    assert_eq!(rect.y, 9.0);
    assert_eq!(rect.width, 94.0);
    assert_eq!(rect.height, 29.0);
    assert!(rect.x >= declared.x);
    assert!(rect.y >= declared.y);
    assert!(rect.right() <= declared.right());
    assert!(rect.bottom() <= declared.bottom());
}

#[test]
fn dropdown_surface_radius_stays_inside_a_narrow_control() {
    let rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 0.5,
        height: 32.0,
    };

    assert_eq!(
        dropdown_surface_radius(&rect, &workbench_dropdown_metrics()),
        0.25
    );
}

#[test]
fn dropdown_geometry_rejects_collapsed_non_finite_and_overflowed_extents() {
    let valid = FrameRect {
        x: 12.0,
        y: 8.0,
        width: 95.0,
        height: 30.5,
    };

    assert!(has_paintable_dropdown_extent(&valid));
    assert!(!has_paintable_dropdown_extent(&FrameRect {
        width: 0.0,
        ..valid.clone()
    }));
    assert!(!has_paintable_dropdown_extent(&FrameRect {
        x: f32::NAN,
        ..valid.clone()
    }));
    assert!(!has_paintable_dropdown_extent(&FrameRect {
        x: f32::MAX,
        ..valid
    }));
}
