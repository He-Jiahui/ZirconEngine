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
fn mapped_file_events_filter_only_canonical_project_transaction_siblings() {
    let root = Path::new("C:/project/assets");
    let basename = "a".repeat(64);
    let transaction_id = format!("{}-42-3", "b".repeat(64));
    let retained = [
        "visible.zasset".to_owned(),
        format!(".{basename}.zr-project-stage-{transaction_id}-copy"),
        format!(".{basename}.zr-project-stage-{}-042-3", "b".repeat(64)),
        format!(".{basename}.zr-project-retired-backup-{transaction_id}"),
    ];
    let event = Event::new(EventKind::Create(notify::event::CreateKind::Any))
        .add_path(root.join(&retained[0]))
        .add_path(root.join(format!(".{basename}.zr-project-stage-{transaction_id}")))
        .add_path(root.join(format!(".{basename}.zr-project-backup-{transaction_id}")))
        .add_path(root.join(format!(
            ".{basename}.zr-project-rollback-stage-{transaction_id}"
        )))
        .add_path(root.join(&retained[1]))
        .add_path(root.join(&retained[2]))
        .add_path(root.join(&retained[3]));

    let mapped = map_notify_event(root, event);
    assert_eq!(mapped.len(), retained.len());
    for (event, file_name) in mapped.iter().zip(retained) {
        assert!(
            matches!(event, AssetWatchEvent::Added(uri) if uri.matches_display(&format!("res://{file_name}"))),
            "unexpected watcher result for {file_name}: {event:?}"
        );
    }
}

#[test]
fn mapped_remove_events_filter_replace_file_zmeta_temps_but_keep_lookalikes() {
    let root = Path::new("C:/project/assets");
    let event = Event::new(EventKind::Remove(notify::event::RemoveKind::Any))
        .add_path(root.join("shaders/pbr_shader.zmeta~RF63fc050.TMP"))
        .add_path(root.join("shaders/user.zmeta~RF63fc05.TMP"))
        .add_path(root.join("shaders/user.TMP"))
        .add_path(root.join("shaders/pbr_shader.zmeta"));

    let mapped = map_notify_event(root, event);
    assert_eq!(mapped.len(), 2);
    assert!(matches!(
        &mapped[0],
        AssetWatchEvent::Removed(uri) if uri.matches_display("res://shaders/user.zmeta~RF63fc05.TMP")
    ));
    assert!(matches!(
        &mapped[1],
        AssetWatchEvent::Removed(uri) if uri.matches_display("res://shaders/user.TMP")
    ));
}

#[test]
fn mapped_directory_modify_echoes_keep_compound_roots_and_file_changes() {
    let root = test_root("directory_modify");
    let ordinary_shaders = root.join("shaders");
    let ordinary_models = root.join("models");
    let compound_root = ordinary_shaders.join("city.bundle");
    let material = root.join("materials/default.zmaterial");
    std::fs::create_dir_all(&compound_root).unwrap();
    std::fs::create_dir_all(&ordinary_models).unwrap();
    std::fs::create_dir_all(material.parent().unwrap()).unwrap();
    std::fs::write(
        compound_root.with_file_name("city.bundle.zmeta"),
        b"descriptor",
    )
    .unwrap();
    std::fs::write(&material, b"source").unwrap();

    let event = Event::new(EventKind::Modify(ModifyKind::Any))
        .add_path(ordinary_shaders.clone())
        .add_path(ordinary_models.clone())
        .add_path(compound_root.clone())
        .add_path(material.clone());
    let mapped = map_notify_event(&root, event);

    assert_eq!(mapped.len(), 2);
    assert!(matches!(
        &mapped[0],
        AssetWatchEvent::Modified(uri) if uri.matches_display("res://shaders/city.bundle")
    ));
    assert!(matches!(
        &mapped[1],
        AssetWatchEvent::Modified(uri) if uri.matches_display("res://materials/default.zmaterial")
    ));

    let create = Event::new(EventKind::Create(notify::event::CreateKind::Folder))
        .add_path(ordinary_shaders.clone());
    assert!(matches!(
        map_notify_event(&root, create).as_slice(),
        [AssetWatchEvent::Added(uri)] if uri.matches_display("res://shaders")
    ));
    std::fs::remove_dir_all(root).unwrap();
}

fn test_root(label: &str) -> PathBuf {
    static NEXT_TEST_ROOT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    std::env::temp_dir().join(format!(
        "zircon_watch_{label}_{}_{}",
        std::process::id(),
        NEXT_TEST_ROOT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ))
}

#[test]
fn mapped_file_events_reserve_the_input_upper_bound() {
    let source = include_str!("../map_notify_event.rs");
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
