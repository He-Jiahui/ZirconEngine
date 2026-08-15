use std::fmt::Write;
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};
use zircon_runtime::core::framework::render::RenderGpuTimingStatus;
use zircon_runtime::graphics::{SceneRendererGpuPassTiming, SceneRendererGpuTimingReport};

pub(crate) const MAX_GPU_TIMING_RESOLVE_FRAMES: u32 = 8;
pub(crate) const GPU_TIMING_EVIDENCE_SCHEMA: &str =
    "zircon_shader_pbr_viewer_gpu_timing_evidence_v1";

pub(crate) fn gpu_timing_report_parent(path: &Path) -> Option<&Path> {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
}

pub(crate) fn validate_gpu_timing_report_output(
    screenshot_path: &Path,
    report_path: &Path,
) -> Result<(), String> {
    let screenshot_sidecar_path = ready_frame_sidecar_path(screenshot_path)?;
    if artifact_paths_match(screenshot_path, report_path)?
        || artifact_paths_match(&screenshot_sidecar_path, report_path)?
    {
        return Err(
            "--gpu-timing-report must not overwrite the --screenshot PNG or its Ready-frame sidecar"
                .to_owned(),
        );
    }
    Ok(())
}

fn ready_frame_sidecar_path(screenshot_path: &Path) -> Result<PathBuf, String> {
    let mut name = screenshot_path
        .file_name()
        .ok_or_else(|| {
            format!(
                "--screenshot requires a file name for its Ready-frame sidecar: {}",
                screenshot_path.display()
            )
        })?
        .to_os_string();
    name.push(".txt");
    Ok(screenshot_path.with_file_name(name))
}

fn artifact_paths_match(left: &Path, right: &Path) -> Result<bool, String> {
    let left = normalized_artifact_path(left)?;
    let right = normalized_artifact_path(right)?;
    Ok(left.eq_ignore_ascii_case(&right))
}

fn normalized_artifact_path(path: &Path) -> Result<String, String> {
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| format!("resolve artifact path {}: {error}", path.display()))?
            .join(path)
    };
    let mut normalized = PathBuf::new();
    for component in resolved.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir | Component::Normal(_) => normalized.push(component.as_os_str()),
        }
    }
    Ok(normalized.to_string_lossy().replace('/', "\\"))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GpuTimingEvidenceResolution {
    Pending,
    Measured(SceneRendererGpuTimingReport),
    Unavailable(RenderGpuTimingStatus),
    TimedOut,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GpuTimingEvidenceRequest {
    target_generation: u64,
    resolve_frames: u32,
}

impl GpuTimingEvidenceRequest {
    pub(crate) const fn new(target_generation: u64) -> Self {
        Self {
            target_generation,
            resolve_frames: 0,
        }
    }

    pub(crate) const fn target_generation(&self) -> u64 {
        self.target_generation
    }

    pub(crate) fn observe(
        &mut self,
        report: Option<SceneRendererGpuTimingReport>,
        status: RenderGpuTimingStatus,
    ) -> GpuTimingEvidenceResolution {
        if let Some(report) = report {
            if report.frame_generation() == self.target_generation {
                return GpuTimingEvidenceResolution::Measured(report);
            }
        }
        if matches!(
            status,
            RenderGpuTimingStatus::Disabled
                | RenderGpuTimingStatus::Unavailable
                | RenderGpuTimingStatus::CapacityExhausted
                | RenderGpuTimingStatus::NoPasses
        ) {
            return GpuTimingEvidenceResolution::Unavailable(status);
        }
        self.resolve_frames = self.resolve_frames.saturating_add(1);
        if self.resolve_frames >= MAX_GPU_TIMING_RESOLVE_FRAMES {
            GpuTimingEvidenceResolution::TimedOut
        } else {
            GpuTimingEvidenceResolution::Pending
        }
    }
}

pub(crate) fn format_gpu_timing_evidence(
    screenshot_path: &Path,
    resolution: &GpuTimingEvidenceResolution,
) -> Result<String, String> {
    let screenshot_name = screenshot_path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .ok_or_else(|| {
            format!(
                "GPU timing evidence requires a screenshot filename: {}",
                screenshot_path.display()
            )
        })?;
    let screenshot_sha256 = screenshot_sha256(screenshot_path)?;
    Ok(format_gpu_timing_evidence_with_screenshot_identity(
        screenshot_name,
        &screenshot_sha256,
        resolution,
    ))
}

fn format_gpu_timing_evidence_with_screenshot_identity(
    screenshot_name: &str,
    screenshot_sha256: &str,
    resolution: &GpuTimingEvidenceResolution,
) -> String {
    match resolution {
        GpuTimingEvidenceResolution::Pending => {
            format!(
                "schema={GPU_TIMING_EVIDENCE_SCHEMA}\n\
                 status=pending\n\
                 screenshot={screenshot_name}\n\
                 screenshot_sha256={screenshot_sha256}\n"
            )
        }
        GpuTimingEvidenceResolution::TimedOut => {
            format!(
                "schema={GPU_TIMING_EVIDENCE_SCHEMA}\n\
                 status=timed_out\n\
                 screenshot={screenshot_name}\n\
                 screenshot_sha256={screenshot_sha256}\n\
                 max_resolve_frames={MAX_GPU_TIMING_RESOLVE_FRAMES}\n"
            )
        }
        GpuTimingEvidenceResolution::Unavailable(status) => {
            format!(
                "schema={GPU_TIMING_EVIDENCE_SCHEMA}\n\
                 status=unavailable\n\
                 screenshot={screenshot_name}\n\
                 screenshot_sha256={screenshot_sha256}\n\
                 renderer_status={status:?}\n"
            )
        }
        GpuTimingEvidenceResolution::Measured(report) => {
            let mut output = format!(
                "schema={GPU_TIMING_EVIDENCE_SCHEMA}\n\
                 status=measured\n\
                 screenshot={screenshot_name}\n\
                 screenshot_sha256={screenshot_sha256}\n\
                 frame_generation={}\n",
                report.frame_generation()
            );
            for timing in report.pass_timings() {
                append_gpu_pass_timing(&mut output, timing);
            }
            output
        }
    }
}

fn screenshot_sha256(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path)
        .map_err(|error| format!("read GPU timing screenshot {}: {error}", path.display()))?;
    Ok(screenshot_sha256_bytes(&bytes))
}

fn screenshot_sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn append_gpu_pass_timing(output: &mut String, timing: &SceneRendererGpuPassTiming) {
    let _ = writeln!(
        output,
        "pass.{}={}",
        timing.pass_name(),
        timing.gpu_time_us()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::work_paths::viewer_test_artifact_root;

    fn report(generation: u64, pass_name: &str, gpu_time_us: u64) -> SceneRendererGpuTimingReport {
        SceneRendererGpuTimingReport::new(
            generation,
            [SceneRendererGpuPassTiming::new(pass_name, gpu_time_us)],
        )
    }

    #[test]
    fn request_accepts_only_its_screenshot_generation() {
        let mut request = GpuTimingEvidenceRequest::new(7);

        assert_eq!(
            request.observe(
                Some(report(6, "direct_scene_content", 42)),
                RenderGpuTimingStatus::Pending,
            ),
            GpuTimingEvidenceResolution::Pending
        );
        assert_eq!(
            request.observe(
                Some(report(7, "direct_scene_content", 42)),
                RenderGpuTimingStatus::Pending,
            ),
            GpuTimingEvidenceResolution::Measured(report(7, "direct_scene_content", 42))
        );
    }

    #[test]
    fn request_times_out_after_a_finite_nonblocking_resolution_budget() {
        let mut request = GpuTimingEvidenceRequest::new(7);

        for _ in 1..MAX_GPU_TIMING_RESOLVE_FRAMES {
            assert_eq!(
                request.observe(None, RenderGpuTimingStatus::Pending),
                GpuTimingEvidenceResolution::Pending
            );
        }
        assert_eq!(
            request.observe(None, RenderGpuTimingStatus::Pending),
            GpuTimingEvidenceResolution::TimedOut
        );
    }

    #[test]
    fn unavailable_timestamp_support_is_reported_without_waiting_for_the_budget() {
        let mut request = GpuTimingEvidenceRequest::new(7);

        assert_eq!(
            request.observe(None, RenderGpuTimingStatus::Unavailable),
            GpuTimingEvidenceResolution::Unavailable(RenderGpuTimingStatus::Unavailable)
        );
    }

    #[test]
    fn measured_evidence_keeps_frame_identity_pass_names_and_microseconds() {
        let resolution =
            GpuTimingEvidenceResolution::Measured(report(7, "direct_output_transfer", 123));

        assert_eq!(
            format_gpu_timing_evidence_with_screenshot_identity(
                "ready.png",
                "deadbeef",
                &resolution,
            ),
            "schema=zircon_shader_pbr_viewer_gpu_timing_evidence_v1\n\
             status=measured\n\
             screenshot=ready.png\n\
             screenshot_sha256=deadbeef\n\
             frame_generation=7\n\
             pass.direct_output_transfer=123\n"
        );
    }

    #[test]
    fn screenshot_identity_uses_the_standard_sha256_digest() {
        assert_eq!(
            screenshot_sha256_bytes(b"fixture"),
            "f16d05ec6b29248d2c61adb1e9263f78e4f7bace1b955014a2d17872cfe4064d"
        );
    }

    #[test]
    fn formatter_hashes_the_actual_ready_png_file() {
        let artifact_root = viewer_test_artifact_root("gpu-timing-screenshot-hash");
        let screenshot_path = artifact_root.join("ready.png");
        std::fs::write(&screenshot_path, b"fixture")
            .expect("controlled screenshot fixture should be written");

        let output = format_gpu_timing_evidence(
            &screenshot_path,
            &GpuTimingEvidenceResolution::Measured(report(7, "direct_scene_content", 123)),
        )
        .expect("a written Ready PNG should produce timing evidence");
        std::fs::remove_dir_all(&artifact_root)
            .expect("controlled screenshot fixture root should be removed");

        assert!(output.contains("screenshot=ready.png\n"));
        assert!(output.contains(
            "screenshot_sha256=f16d05ec6b29248d2c61adb1e9263f78e4f7bace1b955014a2d17872cfe4064d\n"
        ));
    }

    #[test]
    fn single_component_gpu_timing_report_path_has_no_directory_to_create() {
        assert_eq!(gpu_timing_report_parent(Path::new("timing.txt")), None);
        assert_eq!(
            gpu_timing_report_parent(Path::new("evidence/timing.txt")),
            Some(Path::new("evidence"))
        );
        assert!(
            include_str!("app.rs").contains("if let Some(parent) = gpu_timing_report_parent(path)")
        );
    }

    #[test]
    fn timing_output_must_not_collide_with_the_ready_png_or_its_sidecar() {
        for (screenshot_path, report_path) in [
            (
                "E:/evidence/pbr-ready.png",
                "E:/evidence/frames/../pbr-ready.png",
            ),
            ("E:/evidence/pbr-ready.png", "E:/evidence/pbr-ready.png.txt"),
            ("E:/Evidence/PBR-READY.PNG", "e:/evidence/pbr-ready.png"),
            (
                "docs/tests/runtime/shader/./pbr-ready.png",
                "./docs/tests/runtime/shader/pbr-ready.png",
            ),
        ] {
            let error = validate_gpu_timing_report_output(
                Path::new(screenshot_path),
                Path::new(report_path),
            )
            .expect_err("timing evidence must not overwrite Ready evidence");
            assert!(error.contains("must not overwrite"));
        }
        validate_gpu_timing_report_output(
            Path::new("E:/evidence/pbr-ready.png"),
            Path::new("E:/evidence/pbr-gpu-timing.txt"),
        )
        .expect("a separate GPU timing report should remain valid");
    }
}
