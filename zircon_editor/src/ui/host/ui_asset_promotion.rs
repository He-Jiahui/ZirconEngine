use std::{fmt::Write as _, path::PathBuf};

use zircon_runtime::asset::project::ProjectManager;

use super::editor_error::EditorError;
use super::project_access::resolve_project_asset_write_path;

pub(crate) struct UiAssetExternalWidgetTarget {
    pub(crate) source_path: PathBuf,
    pub(crate) asset_id: String,
    pub(crate) document_id: String,
}

pub(crate) struct UiAssetExternalStyleTarget {
    pub(crate) source_path: PathBuf,
    pub(crate) asset_id: String,
    pub(crate) document_id: String,
    pub(crate) display_name: String,
}

pub(crate) fn resolve_external_widget_target(
    project: &ProjectManager,
    preferred_asset_id: &str,
    _component_name: &str,
    preferred_document_id: &str,
) -> Result<UiAssetExternalWidgetTarget, EditorError> {
    let mut suffix = 0usize;
    let mut asset_id = initial_asset_id(preferred_asset_id);
    loop {
        let source_path = resolve_project_asset_write_path(project, &asset_id)?;
        if !source_path.exists() {
            return Ok(UiAssetExternalWidgetTarget {
                source_path,
                asset_id,
                document_id: candidate_document_id(preferred_document_id, suffix),
            });
        }
        suffix += 1;
        reset_suffixed_asset_id(&mut asset_id, preferred_asset_id, suffix);
    }
}

pub(crate) fn resolve_external_style_target(
    project: &ProjectManager,
    preferred_asset_id: &str,
    preferred_document_id: &str,
    preferred_display_name: &str,
) -> Result<UiAssetExternalStyleTarget, EditorError> {
    let mut suffix = 0usize;
    let mut asset_id = initial_asset_id(preferred_asset_id);
    loop {
        let source_path = resolve_project_asset_write_path(project, &asset_id)?;
        if !source_path.exists() {
            return Ok(UiAssetExternalStyleTarget {
                source_path,
                asset_id,
                document_id: candidate_document_id(preferred_document_id, suffix),
                display_name: candidate_display_name(preferred_display_name, suffix),
            });
        }
        suffix += 1;
        reset_suffixed_asset_id(&mut asset_id, preferred_asset_id, suffix);
    }
}

fn initial_asset_id(asset_id: &str) -> String {
    let mut candidate = String::with_capacity(asset_id.len() + 21);
    candidate.push_str(asset_id);
    candidate
}

fn reset_suffixed_asset_id(target: &mut String, asset_id: &str, suffix: usize) {
    target.clear();
    if let Some(base) = asset_id.strip_suffix(".zui") {
        target.push_str(base);
        write!(target, "_{suffix}.zui").expect("writing to a String cannot fail");
    } else {
        target.push_str(asset_id);
        write!(target, "_{suffix}").expect("writing to a String cannot fail");
    }
}

fn candidate_document_id(document_id: &str, suffix: usize) -> String {
    if suffix == 0 {
        document_id.to_owned()
    } else {
        format!("{document_id}_{suffix}")
    }
}

fn candidate_display_name(display_name: &str, suffix: usize) -> String {
    if suffix == 0 {
        display_name.to_owned()
    } else {
        format!("{display_name} {suffix}")
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::{
        candidate_display_name, candidate_document_id, initial_asset_id, reset_suffixed_asset_id,
    };

    #[test]
    fn optimization_batch_20260831gr_editor573_target_suffix_compatibility() {
        let mut candidate = initial_asset_id("panel.zui");
        assert_eq!(candidate, "panel.zui");
        reset_suffixed_asset_id(&mut candidate, "panel.zui", 4);
        assert_eq!(candidate, "panel_4.zui");
        reset_suffixed_asset_id(&mut candidate, "panel", 4);
        assert_eq!(candidate, "panel_4");
        assert_eq!(candidate_document_id("panel", 0), "panel");
        assert_eq!(candidate_document_id("panel", 4), "panel_4");
        assert_eq!(candidate_display_name("Panel", 0), "Panel");
        assert_eq!(candidate_display_name("Panel", 4), "Panel 4");
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_20260831gr_editor573_conflict_scan_deferred_allocations_benchmark() {
        const SAMPLE_PAIRS: usize = 21;
        const ITERATIONS: usize = 8_000;
        const CONFLICTS: usize = 32;
        let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
        let mut checksum = 0usize;
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                let (elapsed, value) = measure(ITERATIONS, CONFLICTS, legacy_scan);
                legacy_ns.push(elapsed);
                checksum ^= value;
                let (elapsed, value) = measure(ITERATIONS, CONFLICTS, optimized_scan);
                optimized_ns.push(elapsed);
                checksum ^= value;
            } else {
                let (elapsed, value) = measure(ITERATIONS, CONFLICTS, optimized_scan);
                optimized_ns.push(elapsed);
                checksum ^= value;
                let (elapsed, value) = measure(ITERATIONS, CONFLICTS, legacy_scan);
                legacy_ns.push(elapsed);
                checksum ^= value;
            }
        }
        let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
        let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
            "deferred target allocations P95 must be at least 30% below eager allocation: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
        println!(
            "EDITOR573_DEFERRED_TARGET_ALLOCATIONS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} iterations={ITERATIONS} conflicts={CONFLICTS} checksum={checksum} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
            join_samples(&legacy_ns),
            join_samples(&optimized_ns),
        );

        fn measure(
            iterations: usize,
            conflicts: usize,
            operation: fn(usize) -> usize,
        ) -> (u128, usize) {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..iterations {
                checksum = checksum.wrapping_add(operation(black_box(conflicts)));
            }
            (started.elapsed().as_nanos(), black_box(checksum))
        }

        fn legacy_scan(conflicts: usize) -> usize {
            for suffix in 0..=conflicts {
                let asset = if suffix == 0 {
                    "panel.zui".to_owned()
                } else {
                    format!("panel_{suffix}.zui")
                };
                let document = if suffix == 0 {
                    "panel".to_owned()
                } else {
                    format!("panel_{suffix}")
                };
                let display = if suffix == 0 {
                    "Panel".to_owned()
                } else {
                    format!("Panel {suffix}")
                };
                if suffix == conflicts {
                    return black_box(asset.len() + document.len() + display.len());
                }
                black_box((&asset, &document, &display));
            }
            unreachable!()
        }

        fn optimized_scan(conflicts: usize) -> usize {
            let mut asset = initial_asset_id("panel.zui");
            for suffix in 0..=conflicts {
                if suffix == conflicts {
                    return black_box(
                        asset.len()
                            + candidate_document_id("panel", conflicts).len()
                            + candidate_display_name("Panel", conflicts).len(),
                    );
                }
                black_box(&asset);
                reset_suffixed_asset_id(&mut asset, "panel.zui", suffix + 1);
            }
            unreachable!()
        }

        fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
            let mut ordered = samples.to_vec();
            ordered.sort_unstable();
            let rank = (ordered.len() * percentile).div_ceil(100).max(1);
            ordered[rank - 1]
        }

        fn join_samples(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }
    }
}
