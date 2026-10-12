use super::*;
use crate::ui::workbench::asset_content_layout::BROWSER_CONTENT_LIST_ROW_HEIGHT;
use crate::ui::workbench::snapshot::{
    AssetItemSnapshot, AssetTypeProjectionSnapshot, AssetViewMode, AssetWorkspaceSnapshot,
};
use std::hint::black_box;
use std::time::Instant;
use zircon_runtime_interface::resource::ResourceKind;

const LARGE_CATALOG_ITEM_COUNT: usize = 100_000;
const VIEWPORT: UiSize = UiSize::new(900.0, 620.0);

fn snapshot(item_count: usize, view_mode: AssetViewMode) -> AssetWorkspaceSnapshot {
    AssetWorkspaceSnapshot {
        view_mode,
        visible_assets: (0..item_count)
            .map(|index| {
                let uuid = format!("asset-{index:06}");
                AssetItemSnapshot {
                    locator: format!("res://assets/{uuid}.mesh"),
                    display_name: uuid.clone(),
                    file_name: format!("{uuid}.mesh"),
                    extension: "mesh".to_owned(),
                    kind: ResourceKind::Mesh,
                    asset_type: AssetTypeProjectionSnapshot::default(),
                    preview_artifact_path: String::new(),
                    dirty: false,
                    diagnostics: Vec::new(),
                    selected: false,
                    resource_state: None,
                    resource_revision: None,
                    uuid,
                }
            })
            .collect(),
        ..AssetWorkspaceSnapshot::default()
    }
}

fn scroll_to_item(view_mode: AssetViewMode, index: usize, item_count: usize) -> f32 {
    match view_mode {
        AssetViewMode::List => index as f32 * BROWSER_CONTENT_LIST_ROW_HEIGHT,
        AssetViewMode::Thumbnail => {
            AssetThumbnailGridMetrics::new(VIEWPORT.width, item_count)
                .item_frame(index)
                .expect("thumbnail item frame")
                .y
        }
    }
}

fn scroll_to_tail(view_mode: AssetViewMode, item_count: usize) -> f32 {
    match view_mode {
        AssetViewMode::List => {
            let metrics = AssetContentLayoutMetrics::for_surface(
                AssetContentSurfaceProfile::Browser,
                view_mode,
            );
            (metrics.list_height(0, item_count) - metrics.viewport_frame(VIEWPORT).height).max(0.0)
        }
        AssetViewMode::Thumbnail => (AssetThumbnailGridMetrics::new(VIEWPORT.width, item_count)
            .content_extent()
            - VIEWPORT.height)
            .max(0.0),
    }
}

#[test]
fn hundred_thousand_asset_preview_demand_tracks_list_and_thumbnail_scroll_windows() {
    for view_mode in [AssetViewMode::List, AssetViewMode::Thumbnail] {
        let mut snapshot = snapshot(LARGE_CATALOG_ITEM_COUNT, view_mode);
        snapshot.selection.uuid = Some("asset-099999".to_owned());
        let snapshot = snapshot.pointer_projection();
        let profile = AssetContentSurfaceProfile::Browser;
        let initial_ordered = ordered_preview_uuids([preview_demand_for_surface(
            &snapshot, profile, VIEWPORT, 0.0,
        )]);
        assert_eq!(
            initial_ordered.first().map(String::as_str),
            Some("asset-099999"),
            "offscreen selection must precede visible rows at admission"
        );
        let initial = initial_ordered.into_iter().collect::<BTreeSet<_>>();
        assert!(initial.contains("asset-000000"));
        assert!(
            initial.contains("asset-099999"),
            "selection must be demanded offscreen"
        );
        assert!(
            initial.len() <= 128,
            "preview requests must follow viewport capacity"
        );
        assert!(!initial.contains("asset-050000"));

        let middle_scroll = scroll_to_item(view_mode, 50_000, LARGE_CATALOG_ITEM_COUNT);
        let middle = ordered_preview_uuids([preview_demand_for_surface(
            &snapshot,
            profile,
            VIEWPORT,
            middle_scroll,
        )])
        .into_iter()
        .collect::<BTreeSet<_>>();
        assert!(!middle.contains("asset-000000"));
        assert!(middle.contains("asset-099999"));
        assert!(middle.contains("asset-050000"));
        assert!(middle.len() <= 128);

        let tail_scroll = scroll_to_tail(view_mode, LARGE_CATALOG_ITEM_COUNT);
        let tail = ordered_preview_uuids([preview_demand_for_surface(
            &snapshot,
            profile,
            VIEWPORT,
            tail_scroll,
        )])
        .into_iter()
        .collect::<BTreeSet<_>>();
        assert!(tail.contains("asset-099999"));
        assert!(
            tail.contains("asset-099998"),
            "tail neighbors must be demanded"
        );
        assert!(!tail.contains("asset-000000"));
        assert!(tail.len() <= 128);
    }
}

#[test]
fn activity_preview_demand_accounts_for_folder_rows_and_unknown_geometry() {
    let mut snapshot = snapshot(1_000, AssetViewMode::List);
    snapshot.visible_folders = (0..20)
        .map(
            |index| crate::ui::workbench::snapshot::AssetFolderSnapshot {
                folder_id: format!("res://folder-{index}"),
                ..Default::default()
            },
        )
        .collect();
    snapshot.selection.uuid = Some("asset-000999".to_owned());

    let activity = ordered_preview_uuids([preview_demand_for_surface(
        &snapshot,
        AssetContentSurfaceProfile::Activity,
        UiSize::new(320.0, 140.0),
        0.0,
    )])
    .into_iter()
    .collect::<BTreeSet<_>>();
    assert!(
        !activity.contains("asset-000000"),
        "folders occupy the viewport"
    );
    assert_eq!(activity.len(), 1, "offscreen selection remains eligible");
    assert!(activity.contains("asset-000999"));

    let unknown = ordered_preview_uuids([preview_demand_for_surface(
        &snapshot,
        AssetContentSurfaceProfile::Browser,
        UiSize::new(0.0, 0.0),
        0.0,
    )])
    .into_iter()
    .collect::<BTreeSet<_>>();
    assert_eq!(
        unknown.len(),
        129,
        "unknown geometry gets bounded prefix and selection"
    );
    assert!(unknown.contains("asset-000000"));
    assert!(unknown.contains("asset-000999"));
}

#[test]
fn selected_previews_from_both_surfaces_precede_bounded_rows_without_duplicates() {
    let mut activity = snapshot(100, AssetViewMode::List);
    activity.selection.uuid = Some("asset-000099".to_owned());
    let mut browser = activity.clone();
    browser.selection.uuid = Some("asset-000098".to_owned());
    let ordered = ordered_preview_uuids([
        preview_demand_for_surface(
            &activity,
            AssetContentSurfaceProfile::Activity,
            VIEWPORT,
            0.0,
        ),
        preview_demand_for_surface(&browser, AssetContentSurfaceProfile::Browser, VIEWPORT, 0.0),
    ]);
    assert_eq!(ordered[0], "asset-000099");
    assert_eq!(ordered[1], "asset-000098");
    assert_eq!(ordered.iter().collect::<BTreeSet<_>>().len(), ordered.len());
    assert!(ordered.len() <= 128);
}

#[test]
#[ignore = "managed Windows Release comparison; run in the combined Editor validation batch"]
fn editor57_hundred_thousand_asset_preview_demand_release_benchmark() {
    const WARMUPS: usize = 5;
    const SAMPLES: usize = 31;
    const MARKER: &str = "EDITOR57_100K_PREVIEW_DEMAND_BENCH_V1";

    for view_mode in [AssetViewMode::List, AssetViewMode::Thumbnail] {
        let snapshot = snapshot(LARGE_CATALOG_ITEM_COUNT, view_mode).pointer_projection();
        let label = match view_mode {
            AssetViewMode::List => "list",
            AssetViewMode::Thumbnail => "thumbnail",
        };
        let scroll_px = scroll_to_item(view_mode, 50_000, LARGE_CATALOG_ITEM_COUNT);
        let legacy = || {
            snapshot
                .visible_assets
                .iter()
                .map(|asset| asset.uuid.clone())
                .collect::<BTreeSet<_>>()
        };
        let bounded = || {
            ordered_preview_uuids([preview_demand_for_surface(
                &snapshot,
                AssetContentSurfaceProfile::Browser,
                VIEWPORT,
                scroll_px,
            )])
        };
        assert_eq!(legacy().len(), LARGE_CATALOG_ITEM_COUNT);
        assert!(bounded().len() <= 128);

        let timed_legacy = || {
            let start = Instant::now();
            black_box(legacy());
            start.elapsed().as_nanos()
        };
        let timed_bounded = || {
            let start = Instant::now();
            black_box(bounded());
            start.elapsed().as_nanos()
        };
        for pair in 0..WARMUPS {
            if pair % 2 == 0 {
                black_box(legacy());
                black_box(bounded());
            } else {
                black_box(bounded());
                black_box(legacy());
            }
        }
        let mut legacy_samples_ns = Vec::with_capacity(SAMPLES);
        let mut bounded_samples_ns = Vec::with_capacity(SAMPLES);
        for pair in 0..SAMPLES {
            if pair % 2 == 0 {
                legacy_samples_ns.push(timed_legacy());
                bounded_samples_ns.push(timed_bounded());
            } else {
                bounded_samples_ns.push(timed_bounded());
                legacy_samples_ns.push(timed_legacy());
            }
        }
        let mut legacy_sorted = legacy_samples_ns.clone();
        let mut bounded_sorted = bounded_samples_ns.clone();
        legacy_sorted.sort_unstable();
        bounded_sorted.sort_unstable();
        let p95_index = (SAMPLES * 95).div_ceil(100) - 1;
        assert!(
            bounded_sorted[p95_index].saturating_mul(4) <= legacy_sorted[p95_index],
            "viewport demand p95 must improve by at least 75%"
        );

        for (phase, raw_samples_ns, sorted) in [
            ("all_filtered", &legacy_samples_ns, &legacy_sorted),
            ("viewport", &bounded_samples_ns, &bounded_sorted),
        ] {
            let percentile = |percent: usize| sorted[(SAMPLES * percent).div_ceil(100) - 1];
            println!(
                "PERF_RESULT {MARKER} mode={label} phase={phase} items={LARGE_CATALOG_ITEM_COUNT} viewport=900x620 warmups={WARMUPS} samples={SAMPLES} p50_ns={} p95_ns={} p99_ns={} raw_samples_ns={raw_samples_ns:?} os={} arch={} package_version={}",
                percentile(50),
                percentile(95),
                percentile(99),
                std::env::consts::OS,
                std::env::consts::ARCH,
                env!("CARGO_PKG_VERSION")
            );
        }
    }
}
