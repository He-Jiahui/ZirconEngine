use super::*;
use zircon_runtime::rhi::{UiSurfaceFrameReadbackRequest, UiSurfaceImageReadbackRequest};

fn scene(format: UiSurfaceFramePixelFormat, raw: Vec<u8>) -> UiSurfaceImageReadback {
    UiSurfaceImageReadback {
        request: UiSurfaceImageReadbackRequest {
            command_index: 12,
            resource_key: "actual-scene".into(),
            resource_generation: 5,
            physical_size: (2, 1),
            expected_pixel_format: None,
        },
        command_frame: None,
        command_clip: None,
        receipt: None,
        pixel_format: Some(format),
        alpha_mode: Some(UiSurfaceImageAlphaMode::Opaque),
        color8: Some(raw),
        opaque_pixels: Some(true),
        failure: None,
    }
}

#[test]
fn normal_viewport_media_srgb_png_preserves_raw_bytes_and_decodes_exact_rgba() {
    for (format, raw, expected, algorithm) in [
        (
            UiSurfaceFramePixelFormat::Rgba8Srgb,
            vec![10, 20, 30, 255, 90, 80, 70, 255],
            vec![10, 20, 30, 255, 90, 80, 70, 255],
            "identity-rgba8-srgb",
        ),
        (
            UiSurfaceFramePixelFormat::Bgra8Srgb,
            vec![30, 20, 10, 255, 70, 80, 90, 255],
            vec![10, 20, 30, 255, 90, 80, 70, 255],
            "bgra8-srgb-to-rgba8-srgb",
        ),
    ] {
        let image = scene(format, raw.clone());
        let (rgba, conversion) = scene_png_rgba(&image).unwrap();
        assert_eq!(conversion, algorithm);
        assert_eq!(rgba, expected);
        let png = encode_scene_png(image.request.physical_size, &rgba).unwrap();
        let decoded = image::load_from_memory_with_format(&png, image::ImageFormat::Png)
            .unwrap()
            .to_rgba8();
        assert_eq!(decoded.dimensions(), (2, 1));
        assert_eq!(decoded.into_raw(), expected);
        assert_eq!(image.color8.as_deref(), Some(raw.as_slice()));
    }
}

#[test]
fn normal_viewport_media_linear_png_uses_explicit_iec_conversion_with_shared_rounding() {
    for (format, raw) in [
        (
            UiSurfaceFramePixelFormat::Rgba8Linear,
            vec![0, 128, 255, 255, 1, 2, 3, 255],
        ),
        (
            UiSurfaceFramePixelFormat::Bgra8Linear,
            vec![255, 128, 0, 255, 3, 2, 1, 255],
        ),
    ] {
        let image = scene(format, raw.clone());
        let (rgba, algorithm) = scene_png_rgba(&image).unwrap();
        assert_eq!(algorithm, "linear8-to-rgba8-srgb-iec61966-2-1");
        assert_eq!(rgba, [0, 188, 255, 255, 13, 22, 28, 255]);
        assert_ne!(
            ZrRuntimeDigestV1::sha256(&rgba),
            ZrRuntimeDigestV1::sha256(&raw)
        );
        let png = encode_scene_png(image.request.physical_size, &rgba).unwrap();
        assert_eq!(
            image::load_from_memory_with_format(&png, image::ImageFormat::Png)
                .unwrap()
                .to_rgba8()
                .into_raw(),
            rgba
        );
        assert_eq!(image.color8.as_deref(), Some(raw.as_slice()));
    }
}

#[test]
fn normal_viewport_media_png_rejects_unproven_format_alpha_extent_and_failed_readback() {
    for mutation in 0..8 {
        let mut image = scene(
            UiSurfaceFramePixelFormat::Rgba8Srgb,
            vec![10, 20, 30, 255, 90, 80, 70, 255],
        );
        match mutation {
            0 => image.pixel_format = None,
            1 => image.alpha_mode = Some(UiSurfaceImageAlphaMode::Premultiplied),
            2 => image.opaque_pixels = None,
            3 => image.color8.as_mut().unwrap()[3] = 254,
            4 => {
                image.color8.as_mut().unwrap().pop();
            }
            5 => image.request.physical_size = (0, 1),
            6 => image.color8 = None,
            _ => image.failure = Some("actual copy failed".into()),
        }
        assert!(scene_png_rgba(&image).is_err(), "mutation {mutation}");
    }
}

#[test]
fn normal_viewport_media_export_reserves_both_owned_buffers_in_the_existing_single_job() {
    let mut result = UiSurfaceFrameReadback {
        request: UiSurfaceFrameReadbackRequest {
            nonce: "quota".into(),
            physical_size: (2, 1),
            expected_draw_generation: None,
            deadline_ms: 1000,
            image: None,
        },
        surface_epoch: 1,
        present_index: 1,
        submission: None,
        receipt: None,
        pixel_format: Some(UiSurfaceFramePixelFormat::Rgba8Srgb),
        color8: Some(vec![255; 8]),
        opaque_pixels: Some(true),
        surface_alpha_mode: "Opaque".into(),
        native_target: None,
        image: None,
        failure: None,
    };
    let window_only = export_bytes(&result);
    assert_eq!(window_only, 8 + 64 * 1024);
    result.image = Some(scene(UiSurfaceFramePixelFormat::Rgba8Srgb, vec![255; 8]));
    let combined = export_bytes(&result);
    assert!(combined >= 4 * (8 + 8));
    assert!(combined > window_only);
    result.image.as_mut().unwrap().color8 = None;
    assert!(export_bytes(&result) < combined);
}

#[test]
fn capture_export_progress_normal_readback_submit_stays_hidden_and_cancellable() {
    use crate::core::editor_message::SharedEditorMessageBus;
    use crate::core::jobs::{test_job_scheduler, EditorJobLimits};
    use crate::core::notifications::EditorNotificationService;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    struct Gate(mpsc::SyncSender<()>, mpsc::Receiver<()>);
    impl EditorJob for Gate {
        type Output = ();
        fn run(self, _context: JobContext) -> Result<(), JobError> {
            self.0.send(()).unwrap();
            self.1.recv_timeout(Duration::from_secs(5)).unwrap();
            Ok(())
        }
    }
    let notifications = EditorNotificationService::default();
    let jobs = EditorJobSystem::with_scheduler_and_bus_and_progress_observer(
        test_job_scheduler(),
        SharedEditorMessageBus::default(),
        EditorJobLimits::default().with_limit(JobCategory::Export, 1),
        notifications.job_progress_observer(),
    );
    let (started_sender, started_receiver) = mpsc::sync_channel(1);
    let (release_sender, release_receiver) = mpsc::channel();
    let gate = jobs
        .submit(
            EditorJobSpec::new("Export scene", JobCategory::Export),
            Gate(started_sender, release_receiver),
        )
        .unwrap();
    started_receiver
        .recv_timeout(Duration::from_secs(5))
        .unwrap();
    let source = jobs.progress();
    let primary = source
        .primary_snapshot_if_changed(None)
        .unwrap()
        .generation();
    let root = std::env::temp_dir().join(format!(
        "zircon-normal-readback-isolation-{}",
        std::process::id()
    ));
    assert!(!root.exists());
    let result = UiSurfaceFrameReadback {
        request: UiSurfaceFrameReadbackRequest {
            nonce: "capture-isolation".into(),
            physical_size: (2, 1),
            expected_draw_generation: None,
            deadline_ms: 1000,
            image: None,
        },
        surface_epoch: 1,
        present_index: 1,
        submission: None,
        receipt: None,
        pixel_format: Some(UiSurfaceFramePixelFormat::Rgba8Srgb),
        color8: Some(vec![255; 8]),
        opaque_pixels: Some(true),
        surface_alpha_mode: "Opaque".into(),
        native_target: None,
        image: None,
        failure: None,
    };
    let expected_bytes = export_bytes(&result);
    let id = submit_normal_readback(
        &jobs,
        NormalReadbackExport {
            directory: root.clone(),
            metadata: json!({}),
            result,
        },
    )
    .unwrap()
    .unwrap();
    assert_eq!(jobs.admission_snapshot().pending_entries(), 1);
    assert_eq!(
        jobs.admission_snapshot().pending_estimated_bytes(),
        expected_bytes
    );
    assert_eq!(source.snapshot_for_ids([id])[0].id(), id);
    assert!(!source.progress_notification_visible(id));
    assert!(source.primary_snapshot_if_changed(Some(primary)).is_none());
    let visible = notifications.progress().snapshot(&source);
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].job().id(), gate.id());
    assert!(jobs.cancel(id));
    assert!(source.snapshot_for_ids([id]).is_empty());
    assert_eq!(jobs.admission_snapshot().pending_entries(), 0);
    release_sender.send(()).unwrap();
    gate.wait().unwrap();
    assert!(jobs
        .shutdown(Instant::now() + Duration::from_secs(5))
        .is_empty());
    assert!(
        !root.exists(),
        "pending capture cancellation must not write files"
    );
}
