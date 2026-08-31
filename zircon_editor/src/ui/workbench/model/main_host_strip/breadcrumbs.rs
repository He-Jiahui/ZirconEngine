use crate::ui::workbench::snapshot::{EditorChromeSnapshot, MainPageSnapshot, ViewContentKind};

use super::super::breadcrumb_model::BreadcrumbModel;
use super::active_view::active_view_in_workspace;

pub(super) fn breadcrumbs_for_page(
    page: &MainPageSnapshot,
    chrome: &EditorChromeSnapshot,
) -> Vec<BreadcrumbModel> {
    match page {
        MainPageSnapshot::Workbench {
            title, workspace, ..
        } => {
            let mut breadcrumbs = breadcrumb_buffer(title.clone());
            if let Some(active_view) = active_view_in_workspace(workspace) {
                breadcrumbs.push(BreadcrumbModel {
                    label: active_view.title.clone(),
                });
            }
            breadcrumbs
        }
        MainPageSnapshot::Exclusive { title, view, .. } => {
            let mut breadcrumbs = breadcrumb_buffer(title.clone());
            if view.content_kind == ViewContentKind::Welcome {
                breadcrumbs.push(BreadcrumbModel {
                    label: chrome.welcome.title.clone(),
                });
            } else if let Some(path) = view
                .serializable_payload
                .get("path")
                .and_then(|value| value.as_str())
            {
                breadcrumbs.push(BreadcrumbModel {
                    label: path.to_string(),
                });
            } else {
                breadcrumbs.push(BreadcrumbModel {
                    label: view.title.clone(),
                });
            }
            breadcrumbs
        }
    }
}

fn breadcrumb_buffer(label: String) -> Vec<BreadcrumbModel> {
    let mut breadcrumbs = Vec::with_capacity(2);
    breadcrumbs.push(BreadcrumbModel { label });
    breadcrumbs
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::breadcrumb_buffer;

    #[test]
    fn optimization_batch_dk_breadcrumb_buffer_reserves_the_pair() {
        let breadcrumbs = breadcrumb_buffer("Project".to_string());

        assert_eq!(breadcrumbs.len(), 1);
        assert!(breadcrumbs.capacity() >= 2);
        assert_eq!(breadcrumbs[0].label, "Project");
    }

    #[test]
    fn optimization_batch_dk_breadcrumb_paths_share_the_reserved_buffer_source() {
        let source = include_str!("breadcrumbs.rs");
        let function = source
            .split("pub(super) fn breadcrumbs_for_page")
            .nth(1)
            .expect("breadcrumb projection")
            .split("fn breadcrumb_buffer")
            .next()
            .expect("projection body");

        assert_eq!(
            function.matches("breadcrumb_buffer(title.clone())").count(),
            2
        );
        assert!(!function.contains("let mut breadcrumbs = vec!["));
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_dk_reserved_breadcrumb_pair_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const PAIRS_PER_SAMPLE: usize = 65_536;

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_pair_buffers(PAIRS_PER_SAMPLE, true));
                optimized_samples.push(measure_pair_buffers(PAIRS_PER_SAMPLE, false));
            } else {
                optimized_samples.push(measure_pair_buffers(PAIRS_PER_SAMPLE, false));
                legacy_samples.push(measure_pair_buffers(PAIRS_PER_SAMPLE, true));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "EDITOR347_RESERVED_BREADCRUMB_PAIR_BENCH_V1 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "reserved breadcrumb pair p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );
    }

    fn measure_pair_buffers(pairs: usize, legacy: bool) -> u128 {
        let started_at = Instant::now();
        let mut checksum = 0_u64;
        for _ in 0..pairs {
            let mut values = if legacy {
                vec![black_box(1_u64)]
            } else {
                let mut values = Vec::with_capacity(2);
                values.push(black_box(1_u64));
                values
            };
            values.push(black_box(2_u64));
            checksum = checksum.wrapping_add(values.capacity() as u64);
            black_box(values);
        }
        black_box(checksum);
        started_at.elapsed().as_nanos()
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        let index = samples
            .len()
            .saturating_mul(95)
            .div_ceil(100)
            .saturating_sub(1);
        samples[index]
    }
}
