use super::super::super::data::{FrameRect, HostViewportImageSet, PaneData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_primitives::{
    draw_gpu_image_clipped_with_resource_key, draw_shared_rgba_image_clipped_with_resource_key,
};

pub(in crate::ui::retained_host::host_contract) fn draw_viewport_image(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    viewport_images: &HostViewportImageSet,
) -> bool {
    let Some(image) = viewport_images
        .for_pane(pane.kind.as_str())
        .filter(|image| image.is_valid())
    else {
        return false;
    };
    let drew_base = match image.rgba() {
        Some(rgba) => draw_shared_rgba_image_clipped_with_resource_key(
            frame,
            body.clone(),
            Some(clip),
            image.resource_key.as_str(),
            image.width,
            image.height,
            rgba,
        ),
        None => draw_gpu_image_clipped_with_resource_key(
            frame,
            body.clone(),
            Some(clip),
            image.resource_key.as_str(),
            image.width,
            image.height,
        ),
    };
    if drew_base {
        if let Some(overlay) = image.overlay() {
            let scale_x = body.width / image.width as f32;
            let scale_y = body.height / image.height as f32;
            let overlay_frame = FrameRect {
                x: body.x + overlay.x as f32 * scale_x,
                y: body.y + overlay.y as f32 * scale_y,
                width: overlay.width as f32 * scale_x,
                height: overlay.height as f32 * scale_y,
            };
            draw_shared_rgba_image_clipped_with_resource_key(
                frame,
                overlay_frame,
                Some(clip),
                overlay.resource_key.as_str(),
                overlay.width,
                overlay.height,
                &overlay.rgba,
            );
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::super::super::super::data::{HostViewportImageData, HostViewportOverlayImageData};
    use super::super::super::super::paint_frame::HostRecordedPaintKind;
    use super::*;

    #[test]
    fn simulate_gizmo_overlay_records_after_the_base_viewport_image() {
        let mut images = HostViewportImageSet::default();
        images.replace_scene(HostViewportImageData {
            resource_key: "play:test-frame".to_string(),
            width: 10,
            height: 10,
            rgba: Some(vec![0; 400].into()),
            play_frame_identity: None,
            overlay: Some(Arc::new(HostViewportOverlayImageData {
                resource_key: "play:test-gizmo".to_string(),
                x: 3,
                y: 4,
                width: 2,
                height: 2,
                rgba: vec![255; 16].into(),
            })),
        });
        let pane = PaneData {
            kind: "Scene".into(),
            ..PaneData::default()
        };
        let body = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 100.0,
        };
        let mut frame = HostRgbaFrame::recording_only(200, 200);

        assert!(draw_viewport_image(
            &mut frame, &pane, &body, &body, &images,
        ));
        let commands = frame.into_recorded_commands();
        assert_eq!(commands.len(), 2);
        let resource_keys = commands
            .iter()
            .map(|command| match &command.kind {
                HostRecordedPaintKind::Image { resource_key, .. } => resource_key.as_str(),
                _ => panic!("viewport composition must record image commands"),
            })
            .collect::<Vec<_>>();
        assert_eq!(resource_keys, ["play:test-frame", "play:test-gizmo"]);
        assert_eq!(commands[1].frame.x, 40.0);
        assert_eq!(commands[1].frame.y, 60.0);
        assert_eq!(commands[1].frame.width, 20.0);
        assert_eq!(commands[1].frame.height, 20.0);
    }

    #[test]
    fn valid_viewport_content_remains_present_outside_damage() {
        let mut images = HostViewportImageSet::default();
        images.replace_scene(HostViewportImageData {
            resource_key: "scene:test-frame".to_string(),
            width: 10,
            height: 10,
            rgba: Some(vec![0; 400].into()),
            play_frame_identity: None,
            overlay: None,
        });
        let pane = PaneData {
            kind: "Scene".into(),
            ..PaneData::default()
        };
        let body = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 100.0,
        };
        let mut frame = HostRgbaFrame::recording_only(300, 300);
        frame.replace_paint_clip(Some(FrameRect {
            x: 200.0,
            y: 200.0,
            width: 20.0,
            height: 20.0,
        }));

        assert!(draw_viewport_image(
            &mut frame, &pane, &body, &body, &images,
        ));
        assert!(frame.into_recorded_commands().is_empty());
    }
}
