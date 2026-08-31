use crate::asset::assets::ui_v2_asset_references;
use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::asset::{AssetReference, AssetUri, UiV2ViewAsset};
use crate::ui::template::{UiAssetDependencyIndex, UiAssetWatchInvalidationReport};

const VIEW_WITH_REFERENCES: &str = r##"
[asset]
kind = "view"
id = "res://ui/views/main.zui"
version = 2

[imports]
widgets = ["res://ui/components/toolbar_button.zui#ToolbarButton"]
styles = ["res://ui/theme/base.zui"]
resources = [
  { kind = "font", uri = "res://fonts/inter.font.toml", fallback = { mode = "placeholder", uri = "res://fonts/system.ttf" } },
  { kind = "image", uri = "res://ui/icons/run.svg", fallback = { mode = "optional" } },
]

[root]
node = "root"

[nodes.root]
component = "ToolbarButton"
control_id = "MainToolbarButton"
props = { text = "Run" }
"##;

#[test]
fn dependency_index_records_bidirectional_refs_from_v2_asset_references() {
    let view = UiV2ViewAsset::from_toml_str(VIEW_WITH_REFERENCES).unwrap();
    let references = ui_v2_asset_references(&view.document);
    let mut index = UiAssetDependencyIndex::new();

    index.record_compiled("res://ui/views/main.zui", &references);

    assert_eq!(index.asset_count(), 1);
    assert_eq!(
        reference_uris(index.references_of("res://ui/views/main.zui")),
        vec![
            "res://fonts/inter.font.toml",
            "res://fonts/system.ttf",
            "res://ui/components/toolbar_button.zui",
            "res://ui/icons/run.svg",
            "res://ui/theme/base.zui",
        ]
    );
    assert_eq!(
        index
            .dependents_of("res://ui/theme/base.zui")
            .collect::<Vec<_>>(),
        vec!["res://ui/views/main.zui"]
    );
    assert_eq!(
        index.cascade_invalidation_targets("res://fonts/inter.font.toml"),
        vec!["res://ui/views/main.zui"]
    );
}

#[test]
fn dependency_index_cascades_invalidation_through_reverse_edges() {
    let mut index = UiAssetDependencyIndex::new();
    index.record_compiled(
        "res://ui/components/button.zui",
        &[asset_ref("res://ui/theme/base.theme.toml")],
    );
    index.record_compiled(
        "res://ui/views/main.zui",
        &[asset_ref("res://ui/components/button.zui")],
    );

    assert_eq!(
        index.cascade_invalidation_targets("res://ui/theme/base.theme.toml"),
        vec!["res://ui/components/button.zui", "res://ui/views/main.zui",]
    );
}

#[test]
fn dependency_index_replaces_stale_edges_and_removes_assets() {
    let mut index = UiAssetDependencyIndex::new();
    index.record_compiled(
        "res://ui/views/main.zui",
        &[asset_ref("res://ui/theme/old.theme.toml")],
    );
    index.record_compiled(
        "res://ui/views/main.zui",
        &[asset_ref("res://ui/theme/new.theme.toml")],
    );

    assert!(index
        .dependents_of("res://ui/theme/old.theme.toml")
        .next()
        .is_none());
    assert_eq!(
        index
            .dependents_of("res://ui/theme/new.theme.toml")
            .collect::<Vec<_>>(),
        vec!["res://ui/views/main.zui"]
    );

    let removed = index.remove("res://ui/views/main.zui").unwrap();

    assert_eq!(
        reference_uris(&removed),
        vec!["res://ui/theme/new.theme.toml"]
    );
    assert!(index.is_empty());
    assert!(index
        .dependents_of("res://ui/theme/new.theme.toml")
        .next()
        .is_none());
}

#[test]
fn dependency_index_deduplicates_and_avoids_readding_changed_asset_in_cycles() {
    let mut index = UiAssetDependencyIndex::new();
    index.record_compiled(
        "res://ui/a.zui",
        &[asset_ref("res://ui/b.zui"), asset_ref("res://ui/b.zui")],
    );
    index.record_compiled("res://ui/b.zui", &[asset_ref("res://ui/a.zui")]);

    assert_eq!(
        reference_uris(index.references_of("res://ui/a.zui")),
        vec!["res://ui/b.zui"]
    );
    assert_eq!(
        index.cascade_invalidation_targets("res://ui/a.zui"),
        vec!["res://ui/b.zui"]
    );
}

#[test]
fn dependency_index_reports_browser_query_for_references_and_dependents() {
    let mut index = UiAssetDependencyIndex::new();
    index.record_compiled(
        "res://ui/components/button.zui",
        &[
            asset_ref("res://ui/theme/base.theme.toml"),
            asset_ref("res://ui/icons/run.icon.toml"),
        ],
    );
    index.record_compiled(
        "res://ui/views/main.zui",
        &[asset_ref("res://ui/components/button.zui")],
    );

    let theme = index.query_asset("res://ui/theme/base.theme.toml");
    assert_eq!(theme.asset_id, "res://ui/theme/base.theme.toml");
    assert!(theme.direct_references.is_empty());
    assert_eq!(
        theme.direct_dependents,
        vec!["res://ui/components/button.zui"]
    );
    assert_eq!(
        theme.cascade_dependents,
        vec!["res://ui/components/button.zui", "res://ui/views/main.zui",]
    );

    let component = index.query_asset("res://ui/components/button.zui");
    assert_eq!(
        reference_uris(&component.direct_references),
        vec![
            "res://ui/icons/run.icon.toml",
            "res://ui/theme/base.theme.toml",
        ]
    );
    assert_eq!(component.direct_dependents, vec!["res://ui/views/main.zui"]);
    assert_eq!(
        component.cascade_dependents,
        vec!["res://ui/views/main.zui"]
    );
}

#[test]
fn dependency_index_maps_modified_watch_changes_to_cascade_rebuild_targets() {
    let mut index = UiAssetDependencyIndex::new();
    index.record_compiled(
        "res://ui/components/button.zui",
        &[asset_ref("res://ui/theme/base.theme.toml")],
    );
    index.record_compiled(
        "res://ui/views/main.zui",
        &[asset_ref("res://ui/components/button.zui")],
    );

    let report = index.apply_watch_changes(&[AssetChange::new(
        AssetChangeKind::Modified,
        uri("res://ui/theme/base.theme.toml"),
        None,
    )]);

    assert_eq!(
        report,
        UiAssetWatchInvalidationReport {
            changed_assets: vec!["res://ui/theme/base.theme.toml".to_string()],
            rebuild_targets: vec![
                "res://ui/components/button.zui".to_string(),
                "res://ui/views/main.zui".to_string(),
            ],
            removed_assets: vec![],
        }
    );
}

#[test]
fn dependency_index_removes_deleted_assets_after_capturing_dependents() {
    let mut index = UiAssetDependencyIndex::new();
    index.record_compiled(
        "res://ui/components/button.zui",
        &[asset_ref("res://ui/theme/base.theme.toml")],
    );
    index.record_compiled(
        "res://ui/views/main.zui",
        &[asset_ref("res://ui/components/button.zui")],
    );

    let report = index.apply_watch_changes(&[AssetChange::new(
        AssetChangeKind::Removed,
        uri("res://ui/components/button.zui"),
        None,
    )]);

    assert_eq!(
        report,
        UiAssetWatchInvalidationReport {
            changed_assets: vec!["res://ui/components/button.zui".to_string()],
            rebuild_targets: vec!["res://ui/views/main.zui".to_string()],
            removed_assets: vec!["res://ui/components/button.zui".to_string()],
        }
    );
    assert!(index
        .dependents_of("res://ui/theme/base.theme.toml")
        .next()
        .is_none());
    assert!(index
        .references_of("res://ui/components/button.zui")
        .is_empty());
}

#[test]
fn dependency_index_renamed_watch_changes_remove_old_key_and_invalidate_new_dependents() {
    let mut index = UiAssetDependencyIndex::new();
    index.record_compiled(
        "res://ui/components/old_button.zui",
        &[asset_ref("res://ui/theme/base.theme.toml")],
    );
    index.record_compiled(
        "res://ui/views/old_consumer.zui",
        &[asset_ref("res://ui/components/old_button.zui")],
    );
    index.record_compiled(
        "res://ui/views/new_consumer.zui",
        &[asset_ref("res://ui/components/new_button.zui")],
    );

    let report = index.apply_watch_changes(&[AssetChange::new(
        AssetChangeKind::Renamed,
        uri("res://ui/components/new_button.zui"),
        Some(uri("res://ui/components/old_button.zui")),
    )]);

    assert_eq!(
        report,
        UiAssetWatchInvalidationReport {
            changed_assets: vec!["res://ui/components/new_button.zui".to_string()],
            rebuild_targets: vec![
                "res://ui/views/old_consumer.zui".to_string(),
                "res://ui/views/new_consumer.zui".to_string(),
            ],
            removed_assets: vec!["res://ui/components/old_button.zui".to_string()],
        }
    );
    assert!(index
        .references_of("res://ui/components/old_button.zui")
        .is_empty());
    assert!(index
        .dependents_of("res://ui/theme/base.theme.toml")
        .next()
        .is_none());
}

#[test]
fn dependency_index_deduplicates_watch_rebuild_targets_across_changes() {
    let mut index = UiAssetDependencyIndex::new();
    index.record_compiled(
        "res://ui/views/main.zui",
        &[
            asset_ref("res://ui/theme/base.theme.toml"),
            asset_ref("res://ui/icons/run.icon.toml"),
        ],
    );

    let report = index.apply_watch_changes(&[
        AssetChange::new(
            AssetChangeKind::Modified,
            uri("res://ui/theme/base.theme.toml"),
            None,
        ),
        AssetChange::new(
            AssetChangeKind::Modified,
            uri("res://ui/icons/run.icon.toml"),
            None,
        ),
    ]);

    assert_eq!(
        report,
        UiAssetWatchInvalidationReport {
            changed_assets: vec![
                "res://ui/theme/base.theme.toml".to_string(),
                "res://ui/icons/run.icon.toml".to_string(),
            ],
            rebuild_targets: vec!["res://ui/views/main.zui".to_string()],
            removed_assets: vec![],
        }
    );
}

fn asset_ref(value: &str) -> AssetReference {
    AssetReference::from_locator(uri(value))
}

fn reference_uris(references: &[AssetReference]) -> Vec<String> {
    references
        .iter()
        .map(|reference| reference.locator.to_string())
        .collect()
}

fn uri(value: &str) -> AssetUri {
    AssetUri::parse(value).unwrap()
}

#[test]
fn optimization_batch_20260826x_runtime74_dependency_cascade_preserves_sorted_bfs_order() {
    let mut index = UiAssetDependencyIndex::new();
    index.record_compiled(
        "res://ui/components/b.zui",
        &[asset_ref("res://ui/theme/base.theme.toml")],
    );
    index.record_compiled(
        "res://ui/components/a.zui",
        &[asset_ref("res://ui/theme/base.theme.toml")],
    );
    index.record_compiled(
        "res://ui/views/main.zui",
        &[
            asset_ref("res://ui/components/a.zui"),
            asset_ref("res://ui/components/b.zui"),
        ],
    );

    assert_eq!(
        index.cascade_invalidation_targets("res://ui/theme/base.theme.toml"),
        vec![
            "res://ui/components/a.zui".to_string(),
            "res://ui/components/b.zui".to_string(),
            "res://ui/views/main.zui".to_string(),
        ]
    );
}

#[test]
fn optimization_batch_20260826x_runtime74_dependency_cascade_uses_borrowed_hash_visited() {
    let source = include_str!("../template/asset/dependency_index.rs");
    let cascade = source
        .split_once("pub fn cascade_invalidation_targets")
        .expect("dependency cascade must remain available")
        .1;

    assert!(cascade.contains("let mut seen: HashSet<&str>"));
    assert!(cascade.contains("let mut queue: VecDeque<&str>"));
    assert!(cascade.contains("dependents_by_asset.get(asset_id)"));
    assert!(!cascade.contains("queue.push_back(dependent.clone())"));
    assert!(cascade.contains("targets.push(dependent.to_string())"));
    assert!(!cascade.contains("let mut seen: BTreeSet<&str>"));
}

use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::{Duration, Instant};

const CASCADE_VISIT_COUNT: usize = 65_536;
const UNIQUE_CASCADE_NODE_COUNT: usize = 8_192;
const CASCADE_SAMPLE_COUNT: usize = 17;

fn cascade_percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn cascade_node_visits() -> Vec<String> {
    (0..CASCADE_VISIT_COUNT)
        .map(|index| {
            format!(
                "res://ui/components/generated/very_long_component_identity_{:05}.zui",
                (index * 4_099) % UNIQUE_CASCADE_NODE_COUNT
            )
        })
        .collect()
}

fn ordered_cascade_visit_count(node_visits: &[String]) -> usize {
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut admitted = 0;
    for node_id in node_visits {
        if seen.insert(node_id.as_str()) {
            admitted += 1;
        }
    }
    admitted
}

fn hash_cascade_visit_count(node_visits: &[String]) -> usize {
    let mut seen: HashSet<&str> = HashSet::new();
    let mut admitted = 0;
    for node_id in node_visits {
        if seen.insert(node_id.as_str()) {
            admitted += 1;
        }
    }
    admitted
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826x_runtime74_dependency_cascade_hash_visited_performance_evidence() {
    let node_visits = cascade_node_visits();
    assert_eq!(
        ordered_cascade_visit_count(&node_visits),
        hash_cascade_visit_count(&node_visits)
    );

    let mut ordered_samples = Vec::with_capacity(CASCADE_SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(CASCADE_SAMPLE_COUNT);
    for sample in 0..CASCADE_SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_cascade_visit_count(black_box(&node_visits)));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(hash_cascade_visit_count(black_box(&node_visits)));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_cascade_visit_count(black_box(&node_visits)));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_cascade_visit_count(black_box(&node_visits)));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p95 = cascade_percentile_95(&mut ordered_samples);
    let hash_p95 = cascade_percentile_95(&mut hash_samples);
    println!(
        "RUNTIME74_DEPENDENCY_CASCADE_HASH_VISITED_BENCH_V1 visits={CASCADE_VISIT_COUNT} \
         unique_nodes={UNIQUE_CASCADE_NODE_COUNT} ordered_lookup_class=log_n \
         hash_lookup_class=average_constant ordered_p95_ns={} hash_p95_ns={}",
        ordered_p95.as_nanos(),
        hash_p95.as_nanos(),
    );
    assert!(
        hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
        "hash-visited P95 {:?} exceeded 60% of ordered-visited P95 {:?}",
        hash_p95,
        ordered_p95,
    );
}
