use crate::ui::{
    event_ui::UiNodeId,
    layout::{UiFrame, UiLayoutMetrics, UiPixelSnapping, UiPixelSnappingPolicy, UiPoint, UiSize},
    surface::{
        UiBrushPayload, UiClipMode, UiPaintPayload, UiRenderCommand, UiRenderCommandKind,
        UiRenderList, UiResolvedStyle, UiVisualAssetRef,
    },
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

#[test]
fn per_command_pixel_snapping_policy_overrides_surface_metrics_at_fractional_dpi() {
    let cases = [(1.25, 10.6), (1.5, 8.166_667)];

    for (dpi_scale, logical_x) in cases {
        let frame = UiFrame::new(logical_x, 4.0, 0.25 / dpi_scale, 12.0);
        let clip = UiFrame::new(logical_x, 3.0, 8.25 / dpi_scale, 14.0);
        let command = |pixel_snapping| UiRenderCommand {
            node_id: UiNodeId::new(43),
            kind: UiRenderCommandKind::Quad,
            frame,
            clip_frame: Some(clip),
            z_index: 3,
            style: UiResolvedStyle {
                background_color: Some("#203040".to_string()),
                pixel_snapping,
                ..UiResolvedStyle::default()
            },
            text_layout: None,
            text: None,
            image: None,
            opacity: 1.0,
        };

        let unsnapped = command(UiPixelSnappingPolicy::Disabled)
            .to_paint_element_with_metrics(7, metrics(dpi_scale, UiPixelSnapping::Enabled));
        assert_eq!(unsnapped.geometry.absolute_frame, frame);
        assert_eq!(unsnapped.geometry.render_bounds, frame);
        assert_eq!(unsnapped.clip.as_ref().expect("clip").frame, clip);
        assert!((unsnapped.geometry.render_bounds.x * dpi_scale - 12.25).abs() < 0.000_1);

        let snapped = command(UiPixelSnappingPolicy::SnapToPixel)
            .to_paint_element_with_metrics(7, metrics(dpi_scale, UiPixelSnapping::Disabled));
        assert_eq!(snapped.geometry.absolute_frame, frame);
        assert_device_pixel_aligned(snapped.geometry.render_bounds.x, dpi_scale);
        assert_device_pixel_aligned(snapped.geometry.render_bounds.right(), dpi_scale);
        assert_device_pixel_aligned(snapped.clip.as_ref().expect("clip").frame.x, dpi_scale);
    }
}

#[test]
fn pixel_snapping_policy_participates_in_render_command_cache_generation() {
    let mut command = UiRenderCommand {
        node_id: UiNodeId::new(44),
        kind: UiRenderCommandKind::Quad,
        frame: UiFrame::new(0.25, 0.25, 12.0, 1.0),
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle::default(),
        text_layout: None,
        text: None,
        image: None,
        opacity: 1.0,
    };
    let inherited_generation = command.cache_generation();
    command.style.pixel_snapping = UiPixelSnappingPolicy::Disabled;

    assert_ne!(inherited_generation, command.cache_generation());
}

#[test]
fn image_raster_target_uses_the_same_resolved_snapping_metrics_as_paint_geometry() {
    let command = |pixel_snapping| UiRenderCommand {
        node_id: UiNodeId::new(45),
        kind: UiRenderCommandKind::Image,
        frame: UiFrame::new(0.5, 2.0, 10.0, 8.0),
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle {
            pixel_snapping,
            ..UiResolvedStyle::default()
        },
        text_layout: None,
        text: None,
        image: Some(UiVisualAssetRef::Icon("toolbar/test.svg".to_string())),
        opacity: 1.0,
    };

    let unsnapped = command(UiPixelSnappingPolicy::Disabled)
        .to_paint_element_with_metrics(0, metrics(1.5, UiPixelSnapping::Enabled));
    assert_eq!(unsnapped.geometry.render_bounds.width, 10.0);
    assert_eq!(image_pixel_size(&unsnapped.payload), Some((15.0, 12.0)));

    let snapped = command(UiPixelSnappingPolicy::SnapToPixel)
        .to_paint_element_with_metrics(0, metrics(1.5, UiPixelSnapping::Disabled));
    assert!((snapped.geometry.render_bounds.width - 10.666_667).abs() < 0.000_1);
    assert_eq!(image_pixel_size(&snapped.payload), Some((16.0, 12.0)));
}

fn scaled_metrics(pixel_snapping: UiPixelSnapping) -> UiLayoutMetrics {
    metrics(2.0, pixel_snapping)
}

fn metrics(dpi_scale: f32, pixel_snapping: UiPixelSnapping) -> UiLayoutMetrics {
    UiLayoutMetrics {
        logical_size: UiSize::new(800.0, 600.0),
        physical_size: UiSize::new(800.0 * dpi_scale, 600.0 * dpi_scale),
        dpi_scale,
        font_scale: 1.0,
        layout_scale: 1.5,
        flow_direction: Default::default(),
        pixel_snapping,
    }
}

fn assert_device_pixel_aligned(logical_value: f32, dpi_scale: f32) {
    let physical_value = logical_value * dpi_scale;
    assert!((physical_value - physical_value.round()).abs() < 0.000_1);
}

fn image_pixel_size(payload: &UiPaintPayload) -> Option<(f32, f32)> {
    let UiPaintPayload::Brush { brushes } = payload else {
        return None;
    };
    match brushes.fill.as_ref()? {
        UiBrushPayload::Image(image) | UiBrushPayload::Box(image) => {
            image.resource_state.pixel_size
        }
        UiBrushPayload::Vector(vector) => vector.resource_state.pixel_size,
        _ => None,
    }
}
