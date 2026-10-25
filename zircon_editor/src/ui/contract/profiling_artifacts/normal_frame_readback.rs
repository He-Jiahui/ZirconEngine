//! Bounded actual GPU artifacts written by the existing injected Editor export-job owner.
use crate::core::jobs::{
    EditorJob, EditorJobAdmissionRequest, EditorJobSpec, EditorJobSystem, JobCategory, JobContext,
    JobError, JobId, JobSubmitError,
};
use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};
use zircon_runtime::rhi::{
    UiSurfaceFramePixelFormat, UiSurfaceFrameReadback, UiSurfaceImageAlphaMode,
    UiSurfaceImageReadback,
};
use zircon_runtime_interface::runtime_build_set::ZrRuntimeDigestV1;
pub(in crate::ui::retained_host::host_contract) struct NormalReadbackExport {
    pub(super) directory: PathBuf,
    pub(super) metadata: Value,
    pub(super) result: UiSurfaceFrameReadback,
}
pub(in crate::ui::retained_host::host_contract) fn submit_normal_readback(
    jobs: &EditorJobSystem,
    export: NormalReadbackExport,
) -> Result<Option<JobId>, JobSubmitError> {
    let bytes = export_bytes(&export.result);
    let reservation = jobs.reserve_batch_admission(vec![EditorJobAdmissionRequest::new(
        JobCategory::Export,
        bytes,
    )])?;
    let mut tickets = reservation.commit(vec![(
        EditorJobSpec::new("Export actual normal GPU frame", JobCategory::Export)
            .with_primary_progress_visible(false)
            .with_progress_notification_visible(false)
            .with_estimated_bytes(bytes),
        NormalReadbackJob { export },
    )])?;
    Ok(tickets.pop().map(|ticket| ticket.id()))
}
struct NormalReadbackJob {
    export: NormalReadbackExport,
}
fn write_new(path: PathBuf, bytes: &[u8]) -> Result<(), JobError> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(JobError::failed)?;
    file.write_all(bytes).map_err(JobError::failed)
}
fn export_bytes(result: &UiSurfaceFrameReadback) -> usize {
    let window = result.color8.as_ref().map_or(0, Vec::len);
    match result.image.as_ref() {
        // Reserve both owned inputs plus bounded RGBA conversion and PNG workspace in one job.
        Some(image) => window
            .saturating_add(image.color8.as_ref().map_or(0, Vec::len))
            .saturating_mul(4)
            .saturating_add(128 * 1024),
        None => window.saturating_add(64 * 1024),
    }
}
fn format_metadata(
    format: Option<UiSurfaceFramePixelFormat>,
) -> Option<(&'static str, &'static str, &'static str)> {
    match format {
        Some(UiSurfaceFramePixelFormat::Rgba8Srgb) => Some(("Rgba8UnormSrgb", "rgba8", "srgb")),
        Some(UiSurfaceFramePixelFormat::Bgra8Srgb) => Some(("Bgra8UnormSrgb", "bgra8", "srgb")),
        Some(UiSurfaceFramePixelFormat::Rgba8Linear) => Some(("Rgba8Unorm", "rgba8", "linear")),
        Some(UiSurfaceFramePixelFormat::Bgra8Linear) => Some(("Bgra8Unorm", "bgra8", "linear")),
        None => None,
    }
}
fn scene_png_rgba(image: &UiSurfaceImageReadback) -> Result<(Vec<u8>, &'static str), String> {
    if image.failure.is_some() {
        return Err("scene GPU readback failed".into());
    }
    let (_, layout, encoding) = format_metadata(image.pixel_format)
        .ok_or_else(|| "scene GPU sampling format is not proven".to_owned())?;
    let raw = image
        .color8
        .as_ref()
        .ok_or_else(|| "scene raw GPU bytes are absent".to_owned())?;
    let expected = u64::from(image.request.physical_size.0)
        .checked_mul(u64::from(image.request.physical_size.1))
        .and_then(|pixels| pixels.checked_mul(4));
    if expected != Some(raw.len() as u64)
        || raw.is_empty()
        || image.alpha_mode != Some(UiSurfaceImageAlphaMode::Opaque)
        || image.opaque_pixels != Some(true)
        || raw.chunks_exact(4).any(|pixel| pixel[3] != 255)
    {
        return Err("scene PNG requires exact opaque GPU bytes; premultiplied/unknown alpha remains diagnostic only".into());
    }
    let mut rgba = raw.clone();
    if layout == "bgra8" {
        for pixel in rgba.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }
    }
    if encoding == "linear" {
        // The actual default texture view samples UNorm as linear values. Preserve raw bytes,
        // explicitly encode these sampling values for a browser sRGB PNG, with shared rounding.
        for pixel in rgba.chunks_exact_mut(4) {
            for channel in &mut pixel[..3] {
                let linear = f64::from(*channel) / 255.0;
                let srgb = if linear <= 0.003_130_8 {
                    linear * 12.92
                } else {
                    1.055 * linear.powf(1.0 / 2.4) - 0.055
                };
                *channel = (255.0 * srgb + 0.5).floor().clamp(0.0, 255.0) as u8;
            }
        }
    }
    Ok((
        rgba,
        if encoding == "linear" {
            "linear8-to-rgba8-srgb-iec61966-2-1"
        } else if layout == "bgra8" {
            "bgra8-srgb-to-rgba8-srgb"
        } else {
            "identity-rgba8-srgb"
        },
    ))
}
fn encode_scene_png(size: (u32, u32), rgba: &[u8]) -> Result<Vec<u8>, image::ImageError> {
    let mut png = Vec::new();
    image::ImageEncoder::write_image(
        image::codecs::png::PngEncoder::new(&mut png),
        rgba,
        size.0,
        size.1,
        image::ExtendedColorType::Rgba8,
    )?;
    Ok(png)
}
fn write_scene_image(
    directory: &Path,
    nonce: &str,
    image: &UiSurfaceImageReadback,
    metadata: &mut Value,
    context: &JobContext,
) -> Result<(), JobError> {
    context.check_cancelled()?;
    if let Some((format, layout, encoding)) = format_metadata(image.pixel_format) {
        metadata["sourceFormat"] = json!(format);
        metadata["sourceByteLayout"] = json!(layout);
        metadata["colorEncoding"] = json!(encoding);
    }
    let Some(raw) = image.color8.as_ref() else {
        return Ok(());
    };
    let base = format!("normal-scene-{nonce}");
    let raw_sha = ZrRuntimeDigestV1::sha256(raw);
    metadata["rawFile"] = json!(format!("{base}.color8"));
    metadata["rawByteLength"] = json!(raw.len());
    metadata["rawRowBytes"] = json!(u64::from(image.request.physical_size.0) * 4);
    metadata["rawSha256"] = json!(raw_sha.as_str());
    write_new(directory.join(format!("{base}.color8")), raw)?;
    context.check_cancelled()?;
    if metadata["readbackCompleted"] != true || !metadata["failure"].is_null() {
        metadata["pngPendingReason"] =
            json!("scene readback is not a successful same-frame resource observation");
        return Ok(());
    }
    let (rgba, algorithm) = match scene_png_rgba(image) {
        Ok(converted) => converted,
        Err(reason) => {
            metadata["pngPendingReason"] = json!(reason);
            return Ok(());
        }
    };
    let rgba_sha = ZrRuntimeDigestV1::sha256(&rgba);
    metadata["rgba8Sha256"] = json!(rgba_sha.as_str());
    metadata["conversion"] = json!({"algorithm":algorithm,"inputSha256":raw_sha.as_str(),"outputSha256":rgba_sha.as_str()});
    let png = encode_scene_png(image.request.physical_size, &rgba).map_err(JobError::failed)?;
    context.check_cancelled()?;
    metadata["pngFile"] = json!(format!("{base}.png"));
    metadata["pngByteLength"] = json!(png.len());
    metadata["pngSha256"] = json!(ZrRuntimeDigestV1::sha256(&png).as_str());
    metadata["pngColorEncoding"] = json!("srgb");
    write_new(directory.join(format!("{base}.png")), &png)
}

#[cfg(test)]
mod normal_viewport_media_tests {
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
}
impl EditorJob for NormalReadbackJob {
    type Output = ();
    fn run(mut self, context: JobContext) -> Result<(), JobError> {
        context.check_cancelled()?;
        let nonce = &self.export.result.request.nonce;
        let file_base = format!("normal-gpu-{nonce}");
        let metadata = &mut self.export.metadata;
        let result = &self.export.result;
        let format = format_metadata(result.pixel_format);
        if let Some((source_format, layout, encoding)) = format {
            metadata["sourceFormat"] = json!(source_format);
            metadata["sourceByteLayout"] = json!(layout);
            metadata["colorEncoding"] = json!(encoding);
        }
        if let Some(raw) = result.color8.as_ref() {
            metadata["rawByteLength"] = json!(raw.len());
            metadata["rawSha256"] = json!(ZrRuntimeDigestV1::sha256(raw).as_str());
            metadata["rawFile"] = json!(format!("{file_base}.color8"));
            write_new(
                self.export.directory.join(format!("{file_base}.color8")),
                raw,
            )?;
            context.check_cancelled()?;
            if result.failure.is_none()
                && result.opaque_pixels == Some(true)
                && format.is_some_and(|(_, _, encoding)| encoding == "srgb")
            {
                let mut rgba = raw.clone();
                if format.is_some_and(|(_, layout, _)| layout == "bgra8") {
                    for pixel in rgba.chunks_exact_mut(4) {
                        pixel.swap(0, 2);
                    }
                }
                metadata["rgba8Sha256"] = json!(ZrRuntimeDigestV1::sha256(&rgba).as_str());
                let png = self.export.directory.join(format!("{file_base}.png"));
                let mut png_bytes = Vec::new();
                image::ImageEncoder::write_image(
                    image::codecs::png::PngEncoder::new(&mut png_bytes),
                    &rgba,
                    result.request.physical_size.0,
                    result.request.physical_size.1,
                    image::ExtendedColorType::Rgba8,
                )
                .map_err(JobError::failed)?;
                metadata["pngFile"] = json!(format!("{file_base}.png"));
                metadata["pngSha256"] = json!(ZrRuntimeDigestV1::sha256(&png_bytes).as_str());
                write_new(png, &png_bytes)?;
            } else {
                metadata["desktopPixelComparisonPendingReason"]=json!("actual pixels require proven sRGB encoding and opaque alpha before desktop matching");
            }
        }
        if let Some(image) = result.image.as_ref() {
            if let Some(scene) = metadata
                .get_mut("sceneImage")
                .filter(|value| value.is_object())
            {
                write_scene_image(&self.export.directory, nonce, image, scene, &context)?;
            }
        }
        context.check_cancelled()?;
        // Publish the receipt last. GPU completion never sets desktop pixelFrameVerified.
        metadata["pixelFrameVerified"] = json!(false);
        write_new(
            self.export.directory.join(format!("{file_base}.json")),
            &serde_json::to_vec_pretty(metadata).map_err(JobError::failed)?,
        )
    }
}
