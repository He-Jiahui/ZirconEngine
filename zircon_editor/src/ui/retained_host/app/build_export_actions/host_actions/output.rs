use std::path::{Path, PathBuf};

use crate::ui::retained_host::app::RetainedEditorHost;
use crate::ui::workbench::project::project_root_path;

use super::super::default_desktop_export_output_root;
use super::super::output_folder::{
    pick_output_folder, reveal_path_in_file_browser, stable_picker_initial_dir,
};
use super::super::DesktopExportActionError;

impl RetainedEditorHost {
    pub(super) fn choose_desktop_export_output(&mut self, profile_name: &str) {
        let project_path = self.runtime.editor_snapshot().project_path;
        let result = project_root_path(&project_path)
            .map_err(DesktopExportActionError::from)
            .and_then(|project_root| {
                let current_output =
                    self.effective_desktop_export_output_root(&project_root, profile_name);
                let initial_dir = stable_picker_initial_dir(&current_output, &project_root);
                pick_output_folder(&initial_dir)
            });

        match result {
            Ok(Some(output_root)) => {
                let status_line = format!(
                    "Desktop export output for {profile_name} set to {}",
                    output_root.display()
                );
                self.desktop_export_output_overrides
                    .insert(profile_name.to_string(), output_root);
                self.desktop_export_wizard_sessions
                    .invalidate_projection_overlay();
                self.mark_layout_dirty();
                self.set_status_line(status_line);
            }
            Ok(None) => self.set_status_line(format!(
                "Desktop export output picker cancelled for {profile_name}"
            )),
            Err(error) => self.set_status_line(format!("Build/export action failed: {error}")),
        }
    }

    pub(super) fn reveal_desktop_export_output(&mut self, profile_name: &str) {
        let project_path = self.runtime.editor_snapshot().project_path;
        let result = project_root_path(&project_path)
            .map_err(DesktopExportActionError::from)
            .and_then(|project_root| {
                let output_root =
                    self.effective_desktop_export_output_root(&project_root, profile_name);
                std::fs::create_dir_all(&output_root).map_err(|source| {
                    DesktopExportActionError::CreateOutput {
                        path: output_root.clone(),
                        source,
                    }
                })?;
                reveal_path_in_file_browser(&output_root)?;
                Ok(output_root)
            });

        match result {
            Ok(output_root) => self.set_status_line(format!(
                "Desktop export output for {profile_name} opened -> {}",
                output_root.display()
            )),
            Err(error) => self.set_status_line(format!("Build/export action failed: {error}")),
        }
    }

    pub(in crate::ui::retained_host::app) fn effective_desktop_export_output_root(
        &self,
        project_root: &Path,
        profile_name: &str,
    ) -> PathBuf {
        self.desktop_export_output_overrides
            .get(profile_name)
            .cloned()
            .unwrap_or_else(|| default_desktop_export_output_root(project_root, profile_name))
    }
}

#[cfg(test)]
mod performance_tests {
    use std::hint::black_box;
    use std::path::{Path, PathBuf};
    use std::time::Instant;

    #[test]
    fn optimization_batch_ef_export_output_path_is_formatted_before_direct_move() {
        let source = include_str!("output.rs");
        let success = source
            .split("Ok(Some(output_root)) =>")
            .nth(1)
            .expect("desktop export output success arm")
            .split("Ok(None)")
            .next()
            .expect("bounded desktop export output success arm");
        let status = success
            .find("let status_line = format!(")
            .expect("output status formatting");
        let insert = success
            .find(".insert(profile_name.to_string(), output_root);")
            .expect("direct output path move");

        assert!(status < insert);
        assert!(!success.contains("output_root.clone()"));
    }

    #[test]
    #[ignore = "release-only direct export output path move benchmark"]
    fn optimization_batch_ef_direct_export_output_path_move_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const PROJECTIONS_PER_SAMPLE: usize = 8_192;

        fn status_line(path: &Path) -> String {
            format!(
                "Desktop export output for Shipping set to {}",
                path.display()
            )
        }

        fn measure_legacy(fixture: &Path) -> u128 {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..PROJECTIONS_PER_SAMPLE {
                let output_root = black_box(fixture.to_path_buf());
                let stored = black_box(output_root.clone());
                let status = black_box(status_line(&output_root));
                checksum = checksum.wrapping_add(stored.as_os_str().len() + status.len());
                black_box((stored, status));
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(fixture: &Path) -> u128 {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..PROJECTIONS_PER_SAMPLE {
                let output_root = black_box(fixture.to_path_buf());
                let status = black_box(status_line(&output_root));
                let stored = black_box(output_root);
                checksum = checksum.wrapping_add(stored.as_os_str().len() + status.len());
                black_box((stored, status));
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn percentile(samples: &[u128], percentile: usize) -> u128 {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (sorted.len() * percentile).div_ceil(100);
            sorted[rank.saturating_sub(1)]
        }

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let fixture = PathBuf::from(format!("E:/ZirconExports/{}Shipping", "nested/".repeat(64)));
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..SAMPLE_PAIRS {
            if sample % 2 == 0 {
                legacy_samples.push(measure_legacy(&fixture));
                optimized_samples.push(measure_optimized(&fixture));
            } else {
                optimized_samples.push(measure_optimized(&fixture));
                legacy_samples.push(measure_legacy(&fixture));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        println!(
            "EDITOR368_DIRECT_EXPORT_OUTPUT_PATH_MOVE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             projections_per_sample={PROJECTIONS_PER_SAMPLE} path_bytes={} \
             pair_order=alternating_legacy_even legacy_extra_path_clones_per_sample={PROJECTIONS_PER_SAMPLE} \
             optimized_extra_path_clones_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
             optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            fixture.as_os_str().len(),
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(85),
            "moving the export output path must reduce P95 by at least 15%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
