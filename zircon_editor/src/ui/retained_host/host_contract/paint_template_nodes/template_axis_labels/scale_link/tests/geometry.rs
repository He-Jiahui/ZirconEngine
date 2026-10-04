use super::*;

fn metrics() -> AxisLabelMetrics {
    AxisLabelMetrics {
        font_size: 11.0,
        line_height: 13.2,
        link_lobe_width: 6.0,
        link_lobe_height: 7.0,
        link_lobe_radius: 3.0,
        link_overlap: 2.0,
        link_connector_width: 1.0,
    }
}

#[test]
fn scale_link_geometry_centers_lobes_and_connector_from_metrics() {
    let node = TemplatePaneNodeData::default();
    let rect = FrameRect {
        x: 8.0,
        y: 8.0,
        width: 18.0,
        height: 24.0,
    };
    let geometry = scale_link_geometry(&node, &rect, &metrics());

    assert_eq!(geometry.lobes[0].x, 12.0);
    assert_eq!(geometry.lobes[0].y, 16.5);
    assert_eq!(geometry.lobes[1].x, 16.0);
    assert_eq!(geometry.connector.x, 17.0);
    assert_eq!(geometry.connector.y, 20.0);
    assert_eq!(geometry.connector.width, 2.0);
    assert_eq!(geometry.connector.height, 1.0);
}

#[test]
fn scale_link_geometry_scales_each_part_into_a_narrow_short_slot() {
    let node = TemplatePaneNodeData::default();
    let rect = FrameRect {
        x: 8.0,
        y: 8.0,
        width: 4.0,
        height: 3.0,
    };
    let geometry = scale_link_geometry(&node, &rect, &metrics());

    for part in [
        geometry.lobes[0].clone(),
        geometry.lobes[1].clone(),
        geometry.connector,
    ] {
        assert_contained(part, &rect);
    }
}

#[test]
fn non_finite_slot_collapses_scale_link_without_non_finite_origins() {
    let geometry = scale_link_geometry(
        &TemplatePaneNodeData::default(),
        &FrameRect {
            x: 8.0,
            y: 8.0,
            width: f32::NAN,
            height: f32::INFINITY,
        },
        &metrics(),
    );

    for part in [
        geometry.lobes[0].clone(),
        geometry.lobes[1].clone(),
        geometry.connector,
    ] {
        assert!(part.x.is_finite());
        assert!(part.y.is_finite());
        assert_eq!((part.width, part.height), (0.0, 0.0));
    }
}

#[test]
fn scale_link_asset_frame_honors_authored_icon_size_around_existing_center() {
    let node = TemplatePaneNodeData {
        layout_icon_size: 17.0,
        layout_offset_x: -12.0,
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 8.0,
        y: 8.0,
        width: 18.0,
        height: 24.0,
    };
    let frame = scale_link_asset_frame(&node, &rect, &metrics());

    assert_eq!((frame.width, frame.height), (17.0, 17.0));
    assert_eq!((frame.x, frame.y), (-3.5, 11.5));
}

fn assert_contained(part: FrameRect, parent: &FrameRect) {
    let epsilon = 0.000_1;
    assert!(part.x >= parent.x - epsilon);
    assert!(part.y >= parent.y - epsilon);
    assert!(part.x + part.width <= parent.x + parent.width + epsilon);
    assert!(part.y + part.height <= parent.y + parent.height + epsilon);
}
