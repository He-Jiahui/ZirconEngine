use crate::ui::{
    event_ui::UiNodeId,
    layout::{UiFrame, UiLayoutMetrics, UiPixelSnapping, UiPoint, UiSize},
    surface::{UiClipMode, UiRenderCommand, UiRenderCommandKind, UiRenderList, UiResolvedStyle},
};

#[test]
fn ui_geometry_metrics_snap_render_bounds_without_changing_layout_frame() {
    let frame = UiFrame::new(10.25, 20.25, 12.25, 5.25);
    let metrics = scaled_metrics(UiPixelSnapping::Enabled);

    let geometry = crate::ui::layout::UiGeometry::from_frame_with_metrics(frame, metrics);

    assert_eq!(geometry.absolute_frame, frame);
    assert_eq!(geometry.local_size, UiSize::new(12.25, 5.25));
    assert_eq!(geometry.render_bounds, UiFrame::new(10.0, 20.0, 12.5, 5.5));
    assert_eq!(geometry.layout_transform.scale, UiPoint::new(1.5, 1.5));
    assert_eq!(geometry.render_transform.scale, UiPoint::new(2.0, 2.0));
    assert_eq!(geometry.pixel_snapping, UiPixelSnapping::Enabled);

    let unsnapped = crate::ui::layout::UiGeometry::from_frame_with_metrics(
        frame,
        scaled_metrics(UiPixelSnapping::Disabled),
    );
    assert_eq!(unsnapped.absolute_frame, frame);
    assert_eq!(unsnapped.render_bounds, frame);
    assert_eq!(unsnapped.pixel_snapping, UiPixelSnapping::Disabled);
}

#[test]
fn ui_render_command_metrics_snap_paint_bounds_and_clip_only_for_render() {
    let frame = UiFrame::new(10.25, 20.25, 12.25, 5.25);
    let clip = UiFrame::new(9.25, 19.25, 14.25, 6.25);
    let command = UiRenderCommand {
        node_id: UiNodeId::new(42),
        kind: UiRenderCommandKind::Quad,
        frame,
        clip_frame: Some(clip),
        z_index: 3,
        style: UiResolvedStyle {
            background_color: Some("#203040".to_string()),
            ..UiResolvedStyle::default()
        },
        text_layout: None,
        text: None,
        image: None,
        opacity: 1.0,
    };

    let element =
        command.to_paint_element_with_metrics(7, scaled_metrics(UiPixelSnapping::Enabled));

    assert_eq!(element.geometry.absolute_frame, frame);
    assert_eq!(
        element.geometry.render_bounds,
        UiFrame::new(10.0, 20.0, 12.5, 5.5)
    );
    assert_eq!(element.geometry.clip_frame, Some(clip));
    assert_eq!(
        element.clip.as_ref().expect("clip").frame,
        UiFrame::new(9.0, 19.0, 14.5, 6.5)
    );
    assert_eq!(
        element.clip.as_ref().expect("clip").mode,
        UiClipMode::Scissor
    );

    let unsnapped =
        command.to_paint_element_with_metrics(7, scaled_metrics(UiPixelSnapping::Disabled));
    assert_eq!(unsnapped.geometry.absolute_frame, frame);
    assert_eq!(unsnapped.geometry.render_bounds, frame);
    assert_eq!(unsnapped.clip.as_ref().expect("clip").frame, clip);

    let list = UiRenderList {
        commands: vec![command],
    };
    let elements = list.to_paint_elements_with_metrics(scaled_metrics(UiPixelSnapping::Enabled));
    assert_eq!(elements[0].geometry.absolute_frame, frame);
    assert_eq!(
        elements[0].geometry.render_bounds,
        UiFrame::new(10.0, 20.0, 12.5, 5.5)
    );
}

fn scaled_metrics(pixel_snapping: UiPixelSnapping) -> UiLayoutMetrics {
    UiLayoutMetrics {
        logical_size: UiSize::new(800.0, 600.0),
        physical_size: UiSize::new(1600.0, 1200.0),
        dpi_scale: 2.0,
        font_scale: 1.0,
        layout_scale: 1.5,
        flow_direction: Default::default(),
        pixel_snapping,
    }
}
