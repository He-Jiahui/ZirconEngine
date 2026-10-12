use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::template_dropdown_metrics::workbench_dropdown_metrics;
use super::super::geometry::{
    dropdown_paint_rect, dropdown_surface_radius, has_paintable_dropdown_extent,
};

#[test]
fn workbench_dropdown_preserves_fractional_declared_bounds() {
    let declared = FrameRect {
        x: 12.3,
        y: 8.4,
        width: 95.2,
        height: 30.5,
    };
    let rect = dropdown_paint_rect(&TemplatePaneNodeData::default(), &declared);

    assert_eq!(rect.x, declared.x);
    assert_eq!(rect.y, declared.y);
    assert_eq!(rect.width, declared.width);
    assert_eq!(rect.height, declared.height);
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
