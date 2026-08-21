use std::fmt::Write;
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};
use zircon_runtime::core::framework::render::RenderGpuTimingStatus;
use zircon_runtime::graphics::{SceneRendererGpuPassTiming, SceneRendererGpuTimingReport};

pub(crate) const GPU_TIMING_WARMUP_SAMPLE_COUNT: usize = 5;
pub(crate) const GPU_TIMING_MEASURED_SAMPLE_COUNT: usize = 31;
pub(crate) const MAX_GPU_TIMING_RESOLVE_FRAMES: u32 = 128;
pub(crate) const GPU_TIMING_EVIDENCE_SCHEMA: &str =
    "zircon_shader_pbr_viewer_gpu_timing_evidence_v2";

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
    Measured(GpuTimingEvidenceDistribution),
    Unavailable(RenderGpuTimingStatus),
    Invalid(String),
    TimedOut,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GpuTimingEvidenceDistribution {
    screenshot_frame_generation: u64,
    warmup_generations: Vec<u64>,
    samples: Vec<SceneRendererGpuTimingReport>,
    timestamp_period_ns_bits: u32,
    pass_names: Vec<String>,
}

impl GpuTimingEvidenceDistribution {
    pub(crate) const fn screenshot_frame_generation(&self) -> u64 {
        self.screenshot_frame_generation
    }

    pub(crate) fn first_measured_frame_generation(&self) -> u64 {
        self.samples
            .first()
            .expect("a completed timing distribution must contain samples")
            .frame_generation()
    }

    pub(crate) fn last_measured_frame_generation(&self) -> u64 {
        self.samples
            .last()
            .expect("a completed timing distribution must contain samples")
            .frame_generation()
    }

    pub(crate) fn samples(&self) -> &[SceneRendererGpuTimingReport] {
        &self.samples
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GpuTimingEvidenceRequest {
    screenshot_frame_generation: u64,
    resolve_frames: u32,
    accepted_last_generation: Option<u64>,
    warmup_generations: Vec<u64>,
    samples: Vec<SceneRendererGpuTimingReport>,
    timestamp_period_ns_bits: Option<u32>,
    pass_names: Option<Vec<String>>,
}

impl GpuTimingEvidenceRequest {
    pub(crate) fn new(screenshot_frame_generation: u64) -> Self {
        Self {
            screenshot_frame_generation,
            resolve_frames: 0,
            accepted_last_generation: None,
            warmup_generations: Vec::with_capacity(GPU_TIMING_WARMUP_SAMPLE_COUNT),
            samples: Vec::with_capacity(GPU_TIMING_MEASURED_SAMPLE_COUNT),
            timestamp_period_ns_bits: None,
            pass_names: None,
        }
    }

    pub(crate) const fn target_generation(&self) -> u64 {
        self.screenshot_frame_generation
    }

    pub(crate) fn observe(
        &mut self,
        report: Option<SceneRendererGpuTimingReport>,
        status: RenderGpuTimingStatus,
    ) -> GpuTimingEvidenceResolution {
        if matches!(
            status,
            RenderGpuTimingStatus::Disabled
                | RenderGpuTimingStatus::Unavailable
                | RenderGpuTimingStatus::Deferred
                | RenderGpuTimingStatus::CapacityExhausted
                | RenderGpuTimingStatus::NoPasses
        ) {
            return GpuTimingEvidenceResolution::Unavailable(status);
        }
        self.resolve_frames = self.resolve_frames.saturating_add(1);
        if let Some(report) = report {
            let resolution = self.observe_report(report);
            if !matches!(resolution, GpuTimingEvidenceResolution::Pending) {
                return resolution;
            }
        }
        if self.resolve_frames >= MAX_GPU_TIMING_RESOLVE_FRAMES {
            GpuTimingEvidenceResolution::TimedOut
        } else {
            GpuTimingEvidenceResolution::Pending
        }
    }

    fn observe_report(
        &mut self,
        report: SceneRendererGpuTimingReport,
    ) -> GpuTimingEvidenceResolution {
        let generation = report.frame_generation();
        if generation <= self.screenshot_frame_generation {
            return GpuTimingEvidenceResolution::Pending;
        }
        let expected_generation = self
            .accepted_last_generation
            .unwrap_or(self.screenshot_frame_generation)
            .checked_add(1);
        if expected_generation != Some(generation) {
            return GpuTimingEvidenceResolution::Invalid(format!(
                "non-consecutive GPU timing generation: expected={expected_generation:?}, actual={generation}"
            ));
        }
        let (timestamp_period_ns_bits, pass_names) = match validate_report(&report) {
            Ok(validated) => validated,
            Err(error) => return GpuTimingEvidenceResolution::Invalid(error),
        };
        if self
            .timestamp_period_ns_bits
            .is_some_and(|expected| expected != timestamp_period_ns_bits)
        {
            return GpuTimingEvidenceResolution::Invalid(
                "GPU timestamp period changed during one evidence distribution".to_owned(),
            );
        }
        if self
            .pass_names
            .as_ref()
            .is_some_and(|expected| expected != &pass_names)
        {
            return GpuTimingEvidenceResolution::Invalid(
                "GPU pass coverage changed during one evidence distribution".to_owned(),
            );
        }
        self.timestamp_period_ns_bits = Some(timestamp_period_ns_bits);
        self.pass_names = Some(pass_names);
        self.accepted_last_generation = Some(generation);

        if self.warmup_generations.len() < GPU_TIMING_WARMUP_SAMPLE_COUNT {
            self.warmup_generations.push(generation);
            return GpuTimingEvidenceResolution::Pending;
        }
        self.samples.push(report);
        if self.samples.len() < GPU_TIMING_MEASURED_SAMPLE_COUNT {
            return GpuTimingEvidenceResolution::Pending;
        }
        GpuTimingEvidenceResolution::Measured(GpuTimingEvidenceDistribution {
            screenshot_frame_generation: self.screenshot_frame_generation,
            warmup_generations: std::mem::take(&mut self.warmup_generations),
            samples: std::mem::take(&mut self.samples),
            timestamp_period_ns_bits: self
                .timestamp_period_ns_bits
                .expect("accepted timing reports must establish a timestamp period"),
            pass_names: self
                .pass_names
                .take()
                .expect("accepted timing reports must establish pass coverage"),
        })
    }
}

fn validate_report(report: &SceneRendererGpuTimingReport) -> Result<(u32, Vec<String>), String> {
    let timestamp_period_ns = report.timestamp_period_ns();
    if !timestamp_period_ns.is_finite() || timestamp_period_ns <= 0.0 {
        return Err("GPU timestamp period must be finite and positive".to_owned());
    }
    if report.pass_timings().is_empty() {
        return Err("GPU timing report must contain at least one pass".to_owned());
    }
    let mut pass_names = Vec::with_capacity(report.pass_timings().len());
    let mut total_gpu_time_us = 0_u64;
    for timing in report.pass_timings() {
        if !valid_pass_name(timing.pass_name()) {
            return Err(format!(
                "GPU timing pass name is not schema-safe: {}",
                timing.pass_name()
            ));
        }
        if pass_names.iter().any(|name| name == timing.pass_name()) {
            return Err(format!(
                "GPU timing report repeats pass: {}",
                timing.pass_name()
            ));
        }
        total_gpu_time_us = total_gpu_time_us
            .checked_add(timing.gpu_time_us())
            .ok_or_else(|| "GPU timing report total exceeds u64".to_owned())?;
        pass_names.push(timing.pass_name().to_owned());
    }
    pass_names.sort_unstable();
    Ok((report.timestamp_period_ns_bits(), pass_names))
}

fn valid_pass_name(pass_name: &str) -> bool {
    let mut bytes = pass_name.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
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
        GpuTimingEvidenceResolution::Invalid(reason) => {
            format!(
                "schema={GPU_TIMING_EVIDENCE_SCHEMA}\n\
                 status=invalid\n\
                 screenshot={screenshot_name}\n\
                 screenshot_sha256={screenshot_sha256}\n\
                 reason={}\n",
                reason.replace(['\n', '\r', '='], "_")
            )
        }
        GpuTimingEvidenceResolution::Measured(distribution) => {
            format_measured_distribution(screenshot_name, screenshot_sha256, distribution)
        }
    }
}

fn format_measured_distribution(
    screenshot_name: &str,
    screenshot_sha256: &str,
    distribution: &GpuTimingEvidenceDistribution,
) -> String {
    let timestamp_period_ns = f32::from_bits(distribution.timestamp_period_ns_bits);
    let timestamp_frequency_hz = 1_000_000_000.0_f64 / f64::from(timestamp_period_ns);
    let warmup_first = distribution
        .warmup_generations
        .first()
        .expect("a completed distribution must retain warmup generations");
    let warmup_last = distribution
        .warmup_generations
        .last()
        .expect("a completed distribution must retain warmup generations");
    let mut output = String::new();
    let _ = write!(
        output,
        "schema={GPU_TIMING_EVIDENCE_SCHEMA}\n\
         status=measured\n\
         screenshot={screenshot_name}\n\
         screenshot_sha256={screenshot_sha256}\n\
         screenshot_frame_generation={}\n\
         warmup_sample_count={}\n\
         warmup_first_frame_generation={warmup_first}\n\
         warmup_last_frame_generation={warmup_last}\n\
         measured_sample_count={}\n\
         first_measured_frame_generation={}\n\
         last_measured_frame_generation={}\n\
         timestamp_period_ns_bits={}\n\
         timestamp_period_ns={timestamp_period_ns:.9}\n\
         timestamp_frequency_hz={timestamp_frequency_hz:.3}\n\
         percentile_policy=nearest_rank\n\
         outlier_policy=none_all_samples_retained\n\
         pass_coverage={}\n",
        distribution.screenshot_frame_generation,
        distribution.warmup_generations.len(),
        distribution.samples.len(),
        distribution.first_measured_frame_generation(),
        distribution.last_measured_frame_generation(),
        distribution.timestamp_period_ns_bits,
        distribution.pass_names.join(","),
    );

    let total_samples = distribution
        .samples
        .iter()
        .map(report_total_gpu_time_us)
        .collect::<Vec<_>>();
    append_distribution_stats(&mut output, "total", &total_samples);
    for pass_name in &distribution.pass_names {
        let samples = distribution
            .samples
            .iter()
            .map(|report| report_pass_gpu_time_us(report, pass_name))
            .collect::<Vec<_>>();
        append_distribution_stats(&mut output, &format!("pass.{pass_name}"), &samples);
    }
    for (index, report) in distribution.samples.iter().enumerate() {
        let _ = writeln!(
            output,
            "sample.{index:03}.frame_generation={}",
            report.frame_generation()
        );
        let _ = writeln!(
            output,
            "sample.{index:03}.total_us={}",
            report_total_gpu_time_us(report)
        );
        for pass_name in &distribution.pass_names {
            let _ = writeln!(
                output,
                "sample.{index:03}.pass.{pass_name}_us={}",
                report_pass_gpu_time_us(report, pass_name)
            );
        }
    }
    output
}

fn report_total_gpu_time_us(report: &SceneRendererGpuTimingReport) -> u64 {
    report
        .pass_timings()
        .iter()
        .map(SceneRendererGpuPassTiming::gpu_time_us)
        .sum()
}

fn report_pass_gpu_time_us(report: &SceneRendererGpuTimingReport, pass_name: &str) -> u64 {
    report
        .pass_timings()
        .iter()
        .find(|timing| timing.pass_name() == pass_name)
        .expect("validated timing distributions must retain stable pass coverage")
        .gpu_time_us()
}

fn append_distribution_stats(output: &mut String, prefix: &str, samples: &[u64]) {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let _ = writeln!(output, "{prefix}.min_us={}", sorted[0]);
    let _ = writeln!(output, "{prefix}.median_us={}", nearest_rank(&sorted, 50));
    let _ = writeln!(output, "{prefix}.p95_us={}", nearest_rank(&sorted, 95));
    let _ = writeln!(
        output,
        "{prefix}.max_us={}",
        sorted[sorted.len().saturating_sub(1)]
    );
}

fn nearest_rank(sorted_samples: &[u64], percentile: usize) -> u64 {
    let rank = sorted_samples
        .len()
        .saturating_mul(percentile)
        .saturating_add(99)
        / 100;
    sorted_samples[rank.saturating_sub(1)]
}

fn screenshot_sha256(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path)
        .map_err(|error| format!("read GPU timing screenshot {}: {error}", path.display()))?;
    Ok(screenshot_sha256_bytes(&bytes))
}

fn screenshot_sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::work_paths::viewer_test_artifact_root;

    fn report(generation: u64, pass_name: &str, gpu_time_us: u64) -> SceneRendererGpuTimingReport {
        SceneRendererGpuTimingReport::new(
            generation,
            1.0,
            [SceneRendererGpuPassTiming::new(pass_name, gpu_time_us)],
        )
    }

    fn direct_report(generation: u64, base_gpu_time_us: u64) -> SceneRendererGpuTimingReport {
        SceneRendererGpuTimingReport::new(
            generation,
            1.0,
            [
                SceneRendererGpuPassTiming::new("direct_gpu_scene_upload", 0),
                SceneRendererGpuPassTiming::new("direct_scene_content", base_gpu_time_us),
                SceneRendererGpuPassTiming::new("direct_output_transfer", base_gpu_time_us + 1),
                SceneRendererGpuPassTiming::new("direct_overlays", base_gpu_time_us + 2),
            ],
        )
    }

    fn measured_distribution(screenshot_generation: u64) -> GpuTimingEvidenceResolution {
        let mut request = GpuTimingEvidenceRequest::new(screenshot_generation);
        let mut resolution = GpuTimingEvidenceResolution::Pending;
        for generation in screenshot_generation + 1
            ..=screenshot_generation
                + (GPU_TIMING_WARMUP_SAMPLE_COUNT + GPU_TIMING_MEASURED_SAMPLE_COUNT) as u64
        {
            resolution = request.observe(
                Some(direct_report(generation, generation)),
                RenderGpuTimingStatus::Pending,
            );
        }
        resolution
    }

    #[test]
    fn request_discards_the_screenshot_frame_then_collects_warmup_and_distribution() {
        let mut request = GpuTimingEvidenceRequest::new(7);

        assert_eq!(
            request.observe(Some(direct_report(7, 42)), RenderGpuTimingStatus::Pending),
            GpuTimingEvidenceResolution::Pending
        );
        for generation in 8..8 + GPU_TIMING_WARMUP_SAMPLE_COUNT as u64 {
            assert_eq!(
                request.observe(
                    Some(direct_report(generation, generation)),
                    RenderGpuTimingStatus::Pending,
                ),
                GpuTimingEvidenceResolution::Pending
            );
        }
        for generation in 13..43 {
            assert_eq!(
                request.observe(
                    Some(direct_report(generation, generation)),
                    RenderGpuTimingStatus::Pending,
                ),
                GpuTimingEvidenceResolution::Pending
            );
        }
        let resolution =
            request.observe(Some(direct_report(43, 43)), RenderGpuTimingStatus::Pending);
        let GpuTimingEvidenceResolution::Measured(distribution) = resolution else {
            panic!("the thirty-first stable sample must complete the distribution");
        };
        assert_eq!(distribution.screenshot_frame_generation(), 7);
        assert_eq!(distribution.first_measured_frame_generation(), 13);
        assert_eq!(distribution.last_measured_frame_generation(), 43);
        assert_eq!(
            distribution.samples().len(),
            GPU_TIMING_MEASURED_SAMPLE_COUNT
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
    fn request_fails_closed_on_a_generation_gap_or_pass_coverage_drift() {
        let mut generation_gap = GpuTimingEvidenceRequest::new(7);
        assert_eq!(
            generation_gap.observe(Some(direct_report(8, 8)), RenderGpuTimingStatus::Pending,),
            GpuTimingEvidenceResolution::Pending
        );
        assert!(matches!(
            generation_gap.observe(Some(direct_report(10, 10)), RenderGpuTimingStatus::Pending,),
            GpuTimingEvidenceResolution::Invalid(_)
        ));

        let mut coverage_drift = GpuTimingEvidenceRequest::new(7);
        assert_eq!(
            coverage_drift.observe(Some(direct_report(8, 8)), RenderGpuTimingStatus::Pending,),
            GpuTimingEvidenceResolution::Pending
        );
        assert!(matches!(
            coverage_drift.observe(
                Some(report(9, "direct_scene_content", 9)),
                RenderGpuTimingStatus::Pending,
            ),
            GpuTimingEvidenceResolution::Invalid(_)
        ));
    }

    #[test]
    fn deferred_sampling_is_terminal_instead_of_silently_dropping_a_frame() {
        let mut request = GpuTimingEvidenceRequest::new(7);

        assert_eq!(
            request.observe(None, RenderGpuTimingStatus::Deferred),
            GpuTimingEvidenceResolution::Unavailable(RenderGpuTimingStatus::Deferred)
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
    fn measured_evidence_keeps_raw_samples_calibration_and_verified_percentiles() {
        let resolution = measured_distribution(7);

        let output = format_gpu_timing_evidence_with_screenshot_identity(
            "ready.png",
            "deadbeef",
            &resolution,
        );
        assert!(output.starts_with(
            "schema=zircon_shader_pbr_viewer_gpu_timing_evidence_v2\nstatus=measured\n"
        ));
        assert!(output.contains("screenshot_frame_generation=7\n"));
        assert!(output.contains("warmup_sample_count=5\n"));
        assert!(output.contains("measured_sample_count=31\n"));
        assert!(output.contains("first_measured_frame_generation=13\n"));
        assert!(output.contains("last_measured_frame_generation=43\n"));
        assert!(output.contains("timestamp_period_ns=1.000000000\n"));
        assert!(output.contains("timestamp_frequency_hz=1000000000.000\n"));
        assert!(output.contains("percentile_policy=nearest_rank\n"));
        assert!(output.contains("outlier_policy=none_all_samples_retained\n"));
        assert!(output.contains("pass.direct_scene_content.median_us=28\n"));
        assert!(output.contains("pass.direct_scene_content.p95_us=42\n"));
        assert!(output.contains("sample.000.frame_generation=13\n"));
        assert!(output.contains("sample.030.frame_generation=43\n"));
        assert!(output.contains("sample.030.pass.direct_overlays_us=45\n"));
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

        let output = format_gpu_timing_evidence(&screenshot_path, &measured_distribution(7))
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
