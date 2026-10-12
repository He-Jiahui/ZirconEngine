use super::*;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

fn metrics() -> TemplateNodeImageGeometryMetrics {
    template_node_image_geometry_metrics_from_host(HostControlMetrics {
        gap_s: 4.0,
        border_width: 1.0,
        ..METRICS
    })
}

#[test]
fn leading_icon_geometry_uses_shared_gap_and_border_inset() {
    let node = TemplatePaneNodeData {
        role: "Button".into(),
        icon_name: "toolbar/save.svg".into(),
        text: "Save".into(),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 30.0,
    };

    let image = image_rect_for_node_with_metrics(&node, &rect, 16, 16, metrics());

    assert_eq!(image.x, rect.x + 5.0);
    assert_eq!(image.y, rect.y + 5.0);
    assert_eq!(image.width, 20.0);
    assert_eq!(image.height, 20.0);
}

#[test]
fn icon_only_geometry_remains_relative_and_centred() {
    let node = TemplatePaneNodeData {
        role: "Icon".into(),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 8.0,
        y: 12.0,
        width: 24.0,
        height: 24.0,
    };

    let image = image_rect_for_node_with_metrics(&node, &rect, 16, 16, metrics());

    let image_center_x = image.x + image.width * 0.5;
    let image_center_y = image.y + image.height * 0.5;
    assert!((image_center_x - (rect.x + rect.width * 0.5)).abs() <= f32::EPSILON);
    assert!((image_center_y - (rect.y + rect.height * 0.5)).abs() <= f32::EPSILON);
    assert!(image.width < rect.width);
    assert_eq!(image.width, image.height);
}

#[test]
fn leading_icon_geometry_caps_shared_inset_inside_narrow_slots() {
    let node = TemplatePaneNodeData {
        role: "Button".into(),
        icon_name: "toolbar/save.svg".into(),
        text: "Save".into(),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 4.0,
        height: 4.0,
    };

    let image = image_rect_for_node_with_metrics(&node, &rect, 16, 16, metrics());

    assert_eq!(image.x, 1.0);
    assert!(image.width > 0.0);
    assert_eq!(image.width, image.height);
    assert!(image.x >= rect.x && image.x + image.width <= rect.x + rect.width);
    assert!(image.y >= rect.y && image.y + image.height <= rect.y + rect.height);
}

#[test]
fn ordinary_image_geometry_preserves_source_aspect_ratio() {
    let node = TemplatePaneNodeData {
        role: "Image".into(),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 120.0,
        height: 80.0,
    };

    let image = image_rect_for_node_with_metrics(&node, &rect, 200, 100, metrics());

    assert_eq!(image.width, 120.0);
    assert_eq!(image.height, 60.0);
    assert_eq!(image.y, 10.0);
}

#[test]
fn image_geometry_rejects_non_finite_or_collapsed_containers() {
    let node = TemplatePaneNodeData {
        role: "Image".into(),
        ..TemplatePaneNodeData::default()
    };
    let invalid_frames = [
        FrameRect {
            x: f32::NAN,
            y: 0.0,
            width: 80.0,
            height: 60.0,
        },
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: f32::INFINITY,
            height: 60.0,
        },
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 0.0,
        },
    ];

    for rect in invalid_frames {
        let image = image_rect_for_node_with_metrics(&node, &rect, 200, 100, metrics());

        assert_eq!(image.width, 0.0);
        assert_eq!(image.height, 0.0);
        assert!(image.x.is_finite());
        assert!(image.y.is_finite());
    }
}
