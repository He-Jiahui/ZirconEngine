use std::collections::HashSet;

use zircon_runtime_interface::project::{validate_engine_version_req, RelPath};
use zircon_runtime_interface::resource::ResourceScheme;

use super::{ProjectManifest, ProjectManifestError};
use crate::asset::project::ProjectPaths;
use std::path::PathBuf;

pub(super) fn default_asset_roots() -> Vec<RelPath> {
    vec![RelPath::project_assets()]
}

impl ProjectManifest {
    pub fn validate(&self) -> Result<(), ProjectManifestError> {
        validate_engine_version_req(self.engine_version_req.as_deref())?;
        if self.asset_roots.is_empty() {
            return Err(ProjectManifestError::EmptyAssetRoots);
        }
        let mut roots = HashSet::new();
        for root in &self.asset_roots {
            if !roots.insert(root.as_str()) {
                return Err(ProjectManifestError::DuplicateAssetRoot {
                    root: root.to_string(),
                });
            }
        }
        for (index, left) in self.asset_roots.iter().enumerate() {
            for right in self.asset_roots.iter().skip(index + 1) {
                if is_descendant(left, right) {
                    return Err(ProjectManifestError::OverlappingAssetRoots {
                        ancestor: left.to_string(),
                        descendant: right.to_string(),
                    });
                }
                if is_descendant(right, left) {
                    return Err(ProjectManifestError::OverlappingAssetRoots {
                        ancestor: right.to_string(),
                        descendant: left.to_string(),
                    });
                }
            }
        }
        let mut ui_roots = HashSet::new();
        for root in &self.ui_roots {
            if root.scheme() != ResourceScheme::Res {
                return Err(ProjectManifestError::InvalidUiRootScheme {
                    root: root.to_string(),
                });
            }
            if root.path().trim().is_empty() {
                return Err(ProjectManifestError::EmptyUiRoot);
            }
            if root.label().is_some() {
                return Err(ProjectManifestError::LabelledUiRoot {
                    root: root.to_string(),
                });
            }
            if !ui_roots.insert(root.to_string()) {
                return Err(ProjectManifestError::DuplicateUiRoot {
                    root: root.to_string(),
                });
            }
        }
        Ok(())
    }

    pub fn primary_asset_root(&self) -> Result<&RelPath, ProjectManifestError> {
        self.asset_roots
            .first()
            .ok_or(ProjectManifestError::EmptyAssetRoots)
    }

    pub fn primary_asset_root_path(
        &self,
        paths: &ProjectPaths,
    ) -> Result<PathBuf, ProjectManifestError> {
        self.primary_asset_root().map(|root| paths.asset_root(root))
    }

    pub fn asset_root_paths(&self, paths: &ProjectPaths) -> Vec<PathBuf> {
        self.asset_roots
            .iter()
            .map(|root| paths.asset_root(root))
            .collect()
    }
}

fn is_descendant(ancestor: &RelPath, candidate: &RelPath) -> bool {
    candidate
        .as_str()
        .strip_prefix(ancestor.as_str())
        .is_some_and(|suffix| suffix.starts_with('/'))
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::BTreeSet;
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use crate::asset::AssetUri;

    use super::*;

    const ROOT_ADMISSION_COUNT: usize = 65_536;
    const UNIQUE_ROOT_COUNT: usize = 8_192;
    const SAMPLE_COUNT: usize = 17;

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() - 1) * 95 / 100]
    }

    fn root_ids() -> Vec<String> {
        (0..ROOT_ADMISSION_COUNT)
            .map(|index| {
                format!(
                    "generated/project/assets/with/long/shared/root_{:05}",
                    (index * 4_099) % UNIQUE_ROOT_COUNT
                )
            })
            .collect()
    }

    fn ordered_unique_count(roots: &[String]) -> usize {
        let mut unique = BTreeSet::new();
        roots
            .iter()
            .filter(|root| unique.insert(root.as_str()))
            .count()
    }

    fn hash_unique_count(roots: &[String]) -> usize {
        let mut unique = HashSet::new();
        roots
            .iter()
            .filter(|root| unique.insert(root.as_str()))
            .count()
    }

    #[test]
    fn optimization_batch_20260826ae_runtime04_hash_root_validation_preserves_first_duplicate_error(
    ) {
        let mut manifest = ProjectManifest::new(
            "Hash Root Validation",
            AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
            1,
        );
        manifest.asset_roots = vec![
            RelPath::parse("assets").unwrap(),
            RelPath::parse("shared-assets").unwrap(),
            RelPath::parse("assets").unwrap(),
        ];

        assert!(matches!(
            manifest.validate(),
            Err(ProjectManifestError::DuplicateAssetRoot { root }) if root == "assets"
        ));
    }

    #[test]
    fn optimization_batch_20260826ae_runtime04_project_root_validation_uses_hash_membership() {
        let source = include_str!("validation.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("use std::collections::HashSet;"));
        assert_eq!(production.matches("HashSet::new()").count(), 2);
        assert!(production.contains("roots.insert(root.as_str())"));
        assert!(production.contains("ui_roots.insert(root.to_string())"));
        assert!(!production.contains("BTreeSet"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_20260826ae_runtime04_project_root_hash_validation_performance_evidence() {
        let roots = root_ids();
        assert_eq!(ordered_unique_count(&roots), hash_unique_count(&roots));

        let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(ordered_unique_count(black_box(&roots)));
                ordered_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(hash_unique_count(black_box(&roots)));
                hash_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(hash_unique_count(black_box(&roots)));
                hash_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(ordered_unique_count(black_box(&roots)));
                ordered_samples.push(started.elapsed());
            }
        }

        let ordered_p95 = percentile_95(&mut ordered_samples);
        let hash_p95 = percentile_95(&mut hash_samples);
        println!(
            "RUNTIME04_PROJECT_ROOT_HASH_VALIDATION_BENCH_V1 \
             admissions={ROOT_ADMISSION_COUNT} unique_roots={UNIQUE_ROOT_COUNT} \
             borrowed_asset_root_identity=true ordered_p95_ns={} hash_p95_ns={}",
            ordered_p95.as_nanos(),
            hash_p95.as_nanos(),
        );
        assert!(
            hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
            "hash-validation P95 {:?} exceeded 60% of ordered-validation P95 {:?}",
            hash_p95,
            ordered_p95,
        );
    }
}
