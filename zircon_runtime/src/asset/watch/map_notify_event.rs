use notify::event::{ModifyKind, RenameMode};
use notify::{Event, EventKind};
use std::path::Path;

use super::{
    asset_watch_event::AssetWatchEvent, watched_asset_uri_for_path::watched_asset_uri_for_path,
};

pub(super) fn map_notify_event(assets_root: &Path, event: Event) -> Vec<AssetWatchEvent> {
    match event.kind {
        EventKind::Create(_) => {
            map_paths_with_capacity(assets_root, &event.paths, AssetWatchEvent::Added)
        }
        EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
            if let [from, to] = event.paths.as_slice() {
                if let (Ok(from), Ok(to)) = (
                    watched_asset_uri_for_path(assets_root, from),
                    watched_asset_uri_for_path(assets_root, to),
                ) {
                    return vec![AssetWatchEvent::Renamed { from, to }];
                }
            }
            Vec::new()
        }
        EventKind::Modify(_) => {
            map_paths_with_capacity(assets_root, &event.paths, AssetWatchEvent::Modified)
        }
        EventKind::Remove(_) => {
            map_paths_with_capacity(assets_root, &event.paths, AssetWatchEvent::Removed)
        }
        _ => Vec::new(),
    }
}

fn map_paths_with_capacity<F>(
    assets_root: &Path,
    paths: &[std::path::PathBuf],
    map: F,
) -> Vec<AssetWatchEvent>
where
    F: Fn(crate::asset::AssetUri) -> AssetWatchEvent,
{
    let mut events = Vec::with_capacity(paths.len());
    for path in paths {
        if let Ok(uri) = watched_asset_uri_for_path(assets_root, path) {
            events.push(map(uri));
        }
    }
    events
}

#[cfg(test)]
mod optimization_batch_20260830bq_runtime_tests {
    use std::path::PathBuf;
    use std::time::Instant;

    use notify::{Event, EventKind};

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const PATHS_PER_SAMPLE: usize = 4_096;

    #[test]
    fn mapped_file_events_preserve_order_and_filter_sidecars() {
        let root = Path::new("C:/project/assets");
        let event = Event::new(EventKind::Create(notify::event::CreateKind::Any))
            .add_path(root.join("first.zasset"))
            .add_path(root.join(".second.zasset.zr-staging-1-2"))
            .add_path(root.join("third.zasset"));

        let mapped = map_notify_event(root, event);
        assert_eq!(mapped.len(), 2);
        assert!(
            matches!(&mapped[0], AssetWatchEvent::Added(uri) if uri.matches_display("res://first.zasset"))
        );
        assert!(
            matches!(&mapped[1], AssetWatchEvent::Added(uri) if uri.matches_display("res://third.zasset"))
        );
    }

    #[test]
    fn mapped_file_events_reserve_the_input_upper_bound() {
        let source = include_str!("map_notify_event.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(implementation.contains("Vec::with_capacity(paths.len())"));
        assert!(implementation.contains("for path in paths"));
        assert!(!implementation.contains("filter_map(|path| watched_asset_uri_for_path"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830bq_runtime_watch_event_mapping_capacity_p95() {
        let root = Path::new("C:/project/assets");
        let paths = (0..PATHS_PER_SAMPLE)
            .map(|index| root.join(format!("asset-{index}.zasset")))
            .collect::<Vec<_>>();
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(root, &paths, false));
                optimized.push(measure(root, &paths, true));
            } else {
                optimized.push(measure(root, &paths, true));
                legacy.push(measure(root, &paths, false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME369_WATCH_EVENT_MAPPING_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} paths_per_sample={PATHS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            sample_csv(&legacy),
            sample_csv(&optimized),
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(root: &Path, paths: &[PathBuf], optimized: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..64 {
            let mapped = if optimized {
                map_paths_with_capacity(root, paths, AssetWatchEvent::Added)
            } else {
                paths
                    .iter()
                    .filter_map(|path| watched_asset_uri_for_path(root, path).ok())
                    .map(AssetWatchEvent::Added)
                    .collect::<Vec<_>>()
            };
            checksum ^= mapped.len();
        }
        std::hint::black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn sample_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
