use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::project::validate_project_name;

use super::{NewProjectTemplate, ProjectAuthorityError};

/// Authoring request for a new project; validation belongs to ProjectAuthority.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewProjectDraft {
    pub project_name: String,
    pub location: String,
    pub template: NewProjectTemplate,
}

impl NewProjectDraft {
    pub fn renderable_empty_default() -> Self {
        Self {
            project_name: "ZirconProject".to_string(),
            location: default_project_location().to_string_lossy().into_owned(),
            template: NewProjectTemplate::RenderableEmpty,
        }
    }

    pub fn project_root(&self) -> Result<PathBuf, ProjectAuthorityError> {
        validate_project_name(&self.project_name)?;
        let project_name = self.project_name.as_str();
        let location = self.location.trim();
        if location.is_empty() {
            return Err(ProjectAuthorityError::EmptyProjectLocation);
        }
        Ok(PathBuf::from(location).join(project_name))
    }

    pub fn validate_for_creation(&self) -> Result<PathBuf, ProjectAuthorityError> {
        let root = super::filesystem::resolve_project_path(&self.project_root()?)?;
        super::filesystem::validate_creation_target(&root)?;
        Ok(root)
    }
}

fn default_project_location() -> PathBuf {
    #[cfg(target_os = "windows")]
    if let Some(home) = std::env::var_os("USERPROFILE") {
        return default_windows_project_location(home);
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join("ZirconProjects");
    }
    // Keep the fallback unresolved so the shared project-path resolver owns current-directory
    // resolution and Windows path identity rules.
    PathBuf::from(".")
}

#[cfg(any(target_os = "windows", test))]
fn default_windows_project_location(home: impl Into<PathBuf>) -> PathBuf {
    let mut location = home.into();
    location.reserve("Documents".len() + "ZirconProjects".len() + 2);
    location.push("Documents");
    location.push("ZirconProjects");
    location
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::path::{Path, PathBuf};
    use std::time::Instant;

    use super::{default_windows_project_location, NewProjectDraft, NewProjectTemplate};

    #[test]
    fn project_root_leaves_relative_locations_for_the_shared_path_resolver() {
        let draft = NewProjectDraft {
            project_name: "Resolver Owned Project".to_string(),
            location: "relative-project-parent".to_string(),
            template: NewProjectTemplate::RenderableEmpty,
        };

        assert_eq!(
            draft.project_root().unwrap(),
            PathBuf::from("relative-project-parent").join("Resolver Owned Project")
        );
    }

    #[cfg(windows)]
    #[test]
    fn drive_relative_creation_location_is_rejected_by_the_shared_path_resolver() {
        let draft = NewProjectDraft {
            project_name: "Resolver Owned Project".to_string(),
            location: r"C:ambiguous-project-parent".to_string(),
            template: NewProjectTemplate::RenderableEmpty,
        };

        assert!(matches!(
            draft.validate_for_creation(),
            Err(super::ProjectAuthorityError::Io { source, .. })
                if source.kind() == std::io::ErrorKind::InvalidInput
        ));
    }

    #[test]
    fn default_location_does_not_hide_current_directory_errors() {
        let source = include_str!("new_project_draft.rs");
        let swallowed_current_directory = ["current_dir()", ".unwrap_or_else"].concat();

        assert!(!source.contains(&swallowed_current_directory));
    }

    #[test]
    fn optimization_batch_ek_default_windows_location_preserves_layout() {
        let home = PathBuf::from("user-home");

        assert_eq!(
            default_windows_project_location(&home),
            home.join("Documents").join("ZirconProjects")
        );
    }

    #[test]
    fn optimization_batch_ek_default_windows_location_uses_one_preallocated_buffer() {
        let source = include_str!("new_project_draft.rs");
        let implementation = source
            .split("fn default_windows_project_location(")
            .nth(1)
            .expect("default Windows project location implementation")
            .split("#[cfg(test)]")
            .next()
            .expect("bounded default project location implementation");

        assert!(implementation.contains("let mut location = home.into()"));
        assert!(implementation.contains("location.reserve("));
        assert!(implementation.contains("location.push(\"Documents\")"));
        assert!(!implementation.contains(".join(\"Documents\").join("));
    }

    #[test]
    #[ignore = "release-only preallocated default project location benchmark"]
    fn optimization_batch_ek_preallocated_default_project_location_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const PATHS_PER_SAMPLE: usize = 4_096;

        fn legacy(home: &Path) -> PathBuf {
            PathBuf::from(home).join("Documents").join("ZirconProjects")
        }

        fn measure_legacy(home: &Path) -> u128 {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..PATHS_PER_SAMPLE {
                let path = black_box(legacy(black_box(home)));
                checksum = checksum.wrapping_add(path.as_os_str().len());
                black_box(path);
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(home: &Path) -> u128 {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..PATHS_PER_SAMPLE {
                let path = black_box(default_windows_project_location(black_box(home)));
                checksum = checksum.wrapping_add(path.as_os_str().len());
                black_box(path);
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

        let home = PathBuf::from(format!("C:\\{}home", "project-home-segment\\".repeat(256)));
        for _ in 0..4 {
            black_box(measure_legacy(&home));
            black_box(measure_optimized(&home));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..SAMPLE_PAIRS {
            if sample % 2 == 0 {
                legacy_samples.push(measure_legacy(&home));
                optimized_samples.push(measure_optimized(&home));
            } else {
                optimized_samples.push(measure_optimized(&home));
                legacy_samples.push(measure_legacy(&home));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        println!(
            "EDITOR373_PREALLOCATED_DEFAULT_PROJECT_LOCATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             paths_per_sample={PATHS_PER_SAMPLE} home_bytes={} \
             pair_order=alternating_legacy_even legacy_path_buffers_per_path=2 \
             optimized_path_buffers_per_path=1 legacy_p50_ns={legacy_p50_ns} \
             optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            home.as_os_str().len(),
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(75),
            "preallocated default project location must reduce P95 by at least 25%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
