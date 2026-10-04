use super::*;

#[test]
fn empty_fractional_scissors_survive_paint_snapping() {
    for dpi_scale in [1.0, 1.25, 1.5, 2.0] {
        let metrics = UiLayoutMetrics {
            dpi_scale,
            ..Default::default()
        };
        for clip in [
            UiFrame::new(40.25, 40.25, 0.0, 0.0),
            UiFrame::new(40.25, 40.25, 0.0, 12.0),
            UiFrame::new(40.25, 40.25, 12.0, 0.0),
        ] {
            for kind in [
                UiRenderCommandKind::Quad,
                UiRenderCommandKind::Text,
                UiRenderCommandKind::Image,
            ] {
                let mut command = command(kind);
                command.clip_frame = Some(clip);
                let elements = command.to_paint_elements_with_metrics(0, metrics);
                assert!(!elements.is_empty());
                for element in elements {
                    assert!(!matches!(element.payload, UiPaintPayload::Empty));
                    assert_eq!(element.clip.unwrap().frame, clip);
                    assert_eq!(element.geometry.clip_frame, Some(clip));
                }
            }
        }
    }
}

#[test]
fn positive_scissors_keep_pixel_snapping_and_absent_scissors_stay_absent() {
    let metrics = UiLayoutMetrics {
        dpi_scale: 1.5,
        ..Default::default()
    };
    let mut command = command(UiRenderCommandKind::Quad);
    let clip = UiFrame::new(40.25, 40.25, 12.0, 12.0);
    command.clip_frame = Some(clip);
    let element = command.to_paint_element_with_metrics(0, metrics);
    assert_eq!(element.clip.unwrap().frame, clip.pixel_snapped(1.5));
    command.clip_frame = None;
    assert!(command
        .to_paint_element_with_metrics(0, metrics)
        .clip
        .is_none());
}

fn command(kind: UiRenderCommandKind) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(1),
        kind,
        frame: UiFrame::new(40.25, 40.25, 80.0, 40.0),
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle {
            background_color: Some("#123456".into()),
            ..Default::default()
        },
        text_layout: None,
        text: (kind == UiRenderCommandKind::Text).then(|| "hidden".into()),
        image: (kind == UiRenderCommandKind::Image)
            .then(|| UiVisualAssetRef::Image("hidden-image".into())),
        opacity: 1.0,
    }
}
