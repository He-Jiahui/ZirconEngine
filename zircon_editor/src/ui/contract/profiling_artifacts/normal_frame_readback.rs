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
#[path = "tests/viewport_media.rs"]
mod normal_viewport_media_tests;
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
