//! Bounded actual GPU artifacts written by the existing injected Editor export-job owner.
use crate::core::jobs::{
    EditorJob, EditorJobAdmissionRequest, EditorJobSpec, EditorJobSystem, JobCategory, JobContext,
    JobError, JobId, JobSubmitError,
};
use serde_json::{json, Value};
use std::io::Write;
use std::path::PathBuf;
use zircon_runtime::rhi::{UiSurfaceFramePixelFormat, UiSurfaceFrameReadback};
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
    let bytes = export
        .result
        .color8
        .as_ref()
        .map_or(0, Vec::len)
        .saturating_add(64 * 1024);
    let reservation = jobs.reserve_batch_admission(vec![EditorJobAdmissionRequest::new(
        JobCategory::Export,
        bytes,
    )])?;
    let mut tickets = reservation.commit(vec![(
        EditorJobSpec::new("Export actual normal GPU frame", JobCategory::Export)
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
impl EditorJob for NormalReadbackJob {
    type Output = ();
    fn run(mut self, context: JobContext) -> Result<(), JobError> {
        context.check_cancelled()?;
        let nonce = &self.export.result.request.nonce;
        let file_base = format!("normal-gpu-{nonce}");
        let metadata = &mut self.export.metadata;
        let result = &self.export.result;
        let format = match result.pixel_format {
            Some(UiSurfaceFramePixelFormat::Rgba8Srgb) => Some(("Rgba8UnormSrgb", "rgba8", "srgb")),
            Some(UiSurfaceFramePixelFormat::Bgra8Srgb) => Some(("Bgra8UnormSrgb", "bgra8", "srgb")),
            Some(UiSurfaceFramePixelFormat::Rgba8Linear) => Some(("Rgba8Unorm", "rgba8", "linear")),
            Some(UiSurfaceFramePixelFormat::Bgra8Linear) => Some(("Bgra8Unorm", "bgra8", "linear")),
            None => None,
        };
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
        context.check_cancelled()?;
        // Publish the receipt last. GPU completion never sets desktop pixelFrameVerified.
        metadata["pixelFrameVerified"] = json!(false);
        write_new(
            self.export.directory.join(format!("{file_base}.json")),
            &serde_json::to_vec_pretty(metadata).map_err(JobError::failed)?,
        )
    }
}
