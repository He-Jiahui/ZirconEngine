use super::{has_paintable_weight_heatmap_extent, WeightHeatmapGeometry};
use crate::ui::retained_host::host_contract::data::FrameRect;

#[test]
fn heatmap_geometry_reserves_measured_legend_label_width_inside_content_bounds() {
    let frame = FrameRect {
        x: 10.0,
        y: 4.0,
        width: 160.0,
        height: 90.0,
    };
    let label_width = 42.0;
    let geometry = WeightHeatmapGeometry::from_frame(&frame, label_width);
    let label = geometry.legend_label_frame(label_width, 8.0, 10.0);

    assert!(geometry.plot.width < 130.0);
    assert_eq!(label.x + label.width, frame.x + frame.width - 3.0);

    let narrow_frame = FrameRect {
        x: 10.0,
        y: 4.0,
        width: 20.0,
        height: 90.0,
    };
    let narrow_geometry = WeightHeatmapGeometry::from_frame(&narrow_frame, label_width);
    let narrow_label = narrow_geometry.legend_label_frame(label_width, 8.0, 10.0);
    assert!(narrow_label.x >= narrow_frame.x);
    assert!(narrow_label.x + narrow_label.width <= narrow_frame.x + narrow_frame.width - 3.0);
}

#[test]
fn collapsed_or_invalid_frames_do_not_materialize_heatmap_geometry() {
    let geometry = WeightHeatmapGeometry::from_frame(
        &FrameRect {
            x: f32::NAN,
            y: f32::INFINITY,
            width: 0.0,
            height: f32::NEG_INFINITY,
        },
        f32::NAN,
    );

    assert!(!geometry.is_drawable());
    assert_eq!(geometry.plot.width, 0.0);
    assert_eq!(geometry.plot.height, 0.0);
    assert_eq!(geometry.legend.width, 0.0);
    assert_eq!(geometry.legend_label_frame(20.0, 4.0, 10.0).width, 0.0);
}

#[test]
fn paintable_extent_requires_finite_positive_frame_dimensions() {
    assert!(has_paintable_weight_heatmap_extent(&FrameRect {
        x: 1.0,
        y: 2.0,
        width: 32.0,
        height: 24.0,
    }));
    for frame in [
        FrameRect {
            x: f32::NAN,
            y: 0.0,
            width: 32.0,
            height: 24.0,
        },
        FrameRect {
            x: 0.0,
            y: f32::INFINITY,
            width: 32.0,
            height: 24.0,
        },
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 24.0,
        },
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: 32.0,
            height: -1.0,
        },
    ] {
        assert!(!has_paintable_weight_heatmap_extent(&frame));
    }
}
