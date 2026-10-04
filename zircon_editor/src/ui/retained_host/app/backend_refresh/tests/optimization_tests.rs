use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use zircon_runtime::asset::project::PreviewState;
use zircon_runtime::asset::watch::{AssetChange, AssetChangeKind};
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime_interface::math::UVec2;
use zircon_runtime_interface::resource::{
    ResourceEvent, ResourceEventKind, ResourceId, ResourceKind, ResourceLocator,
};

use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogGeneration, EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord,
    EditorAssetChange, EditorAssetChangeKind, EditorAssetFolderRecord,
};
use crate::ui::host::{EditorHostEventController, EditorManager};
use crate::ui::workbench::state::EditorState;

use super::{plan_asset_backend_refresh, AssetBackendRefreshPlan};

// Exact pre-slice production planner, retained locally as an independent behavior comparator.
fn retired_plan(
    selected_asset_uuid: Option<&str>,
    active_scene_uri: Option<&str>,
    asset_changes: &[AssetChange],
    editor_changes: &[EditorAssetChange],
    resource_changes: &[ResourceEvent],
) -> AssetBackendRefreshPlan {
    let mut plan = AssetBackendRefreshPlan::default();

    for change in editor_changes {
        match change.kind {
            EditorAssetChangeKind::CatalogChanged => {
                plan.sync_catalog = true;
                plan.refresh_selected_asset_details = true;
                plan.refresh_visible_asset_previews = true;
                plan.mark_presentation_dirty = true;
            }
            EditorAssetChangeKind::AssetStateChanged => {
                plan.sync_catalog = true;
                plan.mark_presentation_dirty = true;
            }
            EditorAssetChangeKind::PreviewChanged => {
                plan.sync_catalog = true;
                plan.refresh_visible_asset_previews = true;
                plan.mark_paint_only_dirty = true;
            }
            EditorAssetChangeKind::PreviewAdmissionAvailable => {
                plan.refresh_visible_asset_previews = true;
            }
            EditorAssetChangeKind::ReferenceChanged => {
                plan.sync_catalog = true;
                plan.refresh_selected_asset_details = true;
                plan.mark_presentation_dirty = true;
            }
        }

        if change.uuid.as_deref() == selected_asset_uuid
            && matches!(
                change.kind,
                EditorAssetChangeKind::CatalogChanged | EditorAssetChangeKind::ReferenceChanged
            )
        {
            plan.refresh_selected_asset_details = true;
        }
    }

    if !resource_changes.is_empty() {
        plan.sync_resources = true;
        plan.mark_render_dirty = true;
        plan.mark_presentation_dirty |= resource_changes.iter().any(|change| {
            matches!(
                change.resource_kind,
                ResourceKind::UiLayout | ResourceKind::UiWidget | ResourceKind::UiStyle
            )
        });
        plan.mark_paint_only_dirty |= resource_changes
            .iter()
            .any(|change| change.resource_kind == ResourceKind::Texture);
    }

    if let Some(active_scene_uri) = active_scene_uri {
        let active_scene_locator = ResourceLocator::parse(active_scene_uri).ok();
        let active_scene_changed = asset_changes
            .iter()
            .any(|change| active_scene_locator.as_ref() == Some(&change.uri))
            || resource_changes.iter().any(|change| {
                change
                    .locator
                    .as_ref()
                    .is_some_and(|locator| active_scene_locator.as_ref() == Some(locator))
                    || change
                        .previous_locator
                        .as_ref()
                        .is_some_and(|locator| active_scene_locator.as_ref() == Some(locator))
            });
        if active_scene_changed {
            plan.reload_active_scene = true;
            plan.mark_render_dirty = true;
            plan.mark_presentation_dirty = true;
        }
    }

    plan
}

#[test]
fn editor57_backend_plan_is_selection_independent_for_every_editor_change_kind() {
    let selected = "asset-selected";
    for kind in [
        EditorAssetChangeKind::CatalogChanged,
        EditorAssetChangeKind::AssetStateChanged,
        EditorAssetChangeKind::PreviewChanged,
        EditorAssetChangeKind::PreviewAdmissionAvailable,
        EditorAssetChangeKind::ReferenceChanged,
    ] {
        for changed_uuid in [None, Some(selected), Some("asset-other")] {
            let changes = [EditorAssetChange {
                kind,
                catalog_revision: 7,
                uuid: changed_uuid.map(str::to_owned),
                locator: None,
            }];
            let current = plan_asset_backend_refresh(None, &[], &changes, &[]);
            for selected_uuid in [None, Some(selected), Some("asset-other")] {
                assert_eq!(
                    current,
                    retired_plan(selected_uuid, None, &[], &changes, &[]),
                    "kind={kind:?}, changed_uuid={changed_uuid:?}, selected_uuid={selected_uuid:?}"
                );
            }
            assert_eq!(
                current.refresh_selected_asset_details,
                matches!(
                    kind,
                    EditorAssetChangeKind::CatalogChanged | EditorAssetChangeKind::ReferenceChanged
                ),
                "kind={kind:?}"
            );
        }
    }
}

#[test]
fn editor57_backend_plan_matches_retired_for_mixed_scene_and_resource_changes() {
    let scene_uri = "res://scenes/main.scene.toml";
    let asset_changes = [AssetChange {
        kind: AssetChangeKind::Modified,
        uri: AssetUri::parse(scene_uri).unwrap(),
        previous_uri: None,
    }];
    let resource_changes = [
        ResourceEvent {
            kind: ResourceEventKind::Updated,
            resource_kind: ResourceKind::Texture,
            id: ResourceId::new(),
            locator: Some(ResourceLocator::parse("res://asset.png").unwrap()),
            previous_locator: None,
            revision: 8,
        },
        ResourceEvent {
            kind: ResourceEventKind::Updated,
            resource_kind: ResourceKind::UiLayout,
            id: ResourceId::new(),
            locator: Some(ResourceLocator::parse("res://ui/inspector.zui").unwrap()),
            previous_locator: Some(ResourceLocator::parse(scene_uri).unwrap()),
            revision: 9,
        },
    ];
    let editor_changes = [
        EditorAssetChange {
            kind: EditorAssetChangeKind::ReferenceChanged,
            catalog_revision: 7,
            uuid: None,
            locator: None,
        },
        EditorAssetChange {
            kind: EditorAssetChangeKind::PreviewChanged,
            catalog_revision: 7,
            uuid: Some("asset-other".to_string()),
            locator: Some("res://asset.png".to_string()),
        },
    ];
    for active_scene_uri in [None, Some(scene_uri), Some("res://other.scene.toml")] {
        let current = plan_asset_backend_refresh(
            active_scene_uri,
            &asset_changes,
            &editor_changes,
            &resource_changes,
        );
        for selected_uuid in [None, Some("asset-selected"), Some("asset-other")] {
            assert_eq!(
                current,
                retired_plan(
                    selected_uuid,
                    active_scene_uri,
                    &asset_changes,
                    &editor_changes,
                    &resource_changes,
                )
            );
        }
        assert!(current.sync_catalog && current.sync_resources);
        assert!(current.refresh_selected_asset_details && current.refresh_visible_asset_previews);
        assert!(current.mark_presentation_dirty && current.mark_render_dirty);
        assert!(current.mark_paint_only_dirty);
        assert_eq!(
            current.reload_active_scene,
            active_scene_uri == Some(scene_uri)
        );
    }
}

#[test]
#[ignore = "managed Windows Release comparison; run in the combined Editor validation batch"]
fn editor57_hundred_thousand_asset_backend_plan_release_benchmark() {
    const ASSETS: usize = 100_000;
    const WARMUP_PAIRS: usize = 5;
    const MEASURED_PAIRS: usize = 31;
    const MARKER: &str = "EDITOR57_100K_BACKEND_PLAN_SELECTION_SNAPSHOT_BENCH_V1";

    assert!(!cfg!(debug_assertions), "this timing gate requires Release");
    let runtime = controller_with_catalog(ASSETS);
    let selected = "asset-000000";
    let changes = [EditorAssetChange {
        kind: EditorAssetChangeKind::CatalogChanged,
        catalog_revision: 2,
        uuid: Some(selected.to_string()),
        locator: Some("res://asset-000000.png".to_string()),
    }];
    let warm_snapshot = runtime.editor_snapshot();
    assert_eq!(warm_snapshot.asset_activity.visible_assets.len(), ASSETS);
    assert_eq!(
        warm_snapshot.asset_activity.selected_asset_uuid.as_deref(),
        Some(selected)
    );

    let measure = |retired: bool| {
        let started = Instant::now();
        let plan = black_box(if retired {
            let selected_uuid = runtime.editor_snapshot().asset_activity.selected_asset_uuid;
            retired_plan(
                selected_uuid.as_deref(),
                black_box(None),
                &[],
                black_box(&changes),
                &[],
            )
        } else {
            plan_asset_backend_refresh(black_box(None), &[], black_box(&changes), &[])
        });
        (started.elapsed().as_nanos(), plan)
    };
    let mut retired_raw_ns = Vec::with_capacity(MEASURED_PAIRS);
    let mut current_raw_ns = Vec::with_capacity(MEASURED_PAIRS);
    for pair in 0..WARMUP_PAIRS + MEASURED_PAIRS {
        let (retired, current) = if pair % 2 == 0 {
            (measure(true), measure(false))
        } else {
            let current = measure(false);
            (measure(true), current)
        };
        assert_eq!(retired.1, current.1, "paired plan flags differ");
        if pair >= WARMUP_PAIRS {
            retired_raw_ns.push(retired.0);
            current_raw_ns.push(current.0);
        }
    }

    let retired_p50 = percentile(&retired_raw_ns, 50);
    let retired_p95 = percentile(&retired_raw_ns, 95);
    let retired_p99 = percentile(&retired_raw_ns, 99);
    let current_p50 = percentile(&current_raw_ns, 50);
    let current_p95 = percentile(&current_raw_ns, 95);
    let current_p99 = percentile(&current_raw_ns, 99);
    println!(
        "PERF_RESULT {MARKER} assets={ASSETS} warmup_pairs={WARMUP_PAIRS} measured_pairs={MEASURED_PAIRS} \
         retired_raw_ns={retired_raw_ns:?} current_raw_ns={current_raw_ns:?} \
         retired_p50_ns={retired_p50} retired_p95_ns={retired_p95} retired_p99_ns={retired_p99} \
         current_p50_ns={current_p50} current_p95_ns={current_p95} current_p99_ns={current_p99} \
         os={} arch={} package_version={}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("CARGO_PKG_VERSION")
    );
    assert!(
        current_p95.saturating_mul(100) <= retired_p95.saturating_mul(20),
        "backend planner p95 should be <= 20% of the retired snapshot-plus-plan p95"
    );
}

fn controller_with_catalog(asset_count: usize) -> EditorHostEventController {
    let core = CoreRuntime::new();
    let manager = Arc::new(EditorManager::new(&core.handle()).expect("editor manager"));
    let mut state = EditorState::with_default_selection_with_context(
        DefaultLevelManager::default().create_default_level(),
        UVec2::new(1280, 720),
        Arc::clone(manager.context()),
    );
    let uuids = (0..asset_count)
        .map(|index| format!("asset-{index:06}"))
        .collect::<Vec<_>>();
    let assets = uuids
        .iter()
        .enumerate()
        .map(|(index, uuid)| {
            let file_name = format!("asset-{index:06}.png");
            EditorAssetCatalogRecord {
                uuid: uuid.clone(),
                id: format!("source-{index:06}"),
                locator: format!("res://{file_name}"),
                kind: ResourceKind::Texture,
                display_name: format!("asset-{index:06}"),
                file_name,
                extension: "png".to_string(),
                preview_state: PreviewState::Ready,
                meta_path: format!("E:/Profile/assets/asset-{index:06}.png.zmeta"),
                preview_artifact_path: String::new(),
                source_mtime_unix_ms: 0,
                source_hash: String::new(),
                dirty: false,
                diagnostics: Vec::new(),
                direct_reference_uuids: Vec::new(),
            }
        })
        .collect();
    let catalog = EditorAssetCatalogSnapshotRecord {
        project_name: "BackendPlanProfile".to_string(),
        project_root: "E:/Profile".to_string(),
        assets_root: "E:/Profile/assets".to_string(),
        cache_root: "E:/Profile/.zircon/cache".to_string(),
        default_scene_uri: String::new(),
        catalog_revision: 1,
        folders: vec![EditorAssetFolderRecord {
            folder_id: "res://".to_string(),
            parent_folder_id: None,
            locator_prefix: "res://".to_string(),
            display_name: "Assets".to_string(),
            child_folder_ids: Vec::new(),
            direct_asset_uuids: uuids,
            recursive_asset_count: asset_count,
        }],
        assets,
    };
    state.sync_asset_catalog(Arc::new(
        EditorAssetCatalogGeneration::from_snapshot_record(catalog, 1),
    ));
    state.mark_project_open();
    assert!(state.select_asset(Some("asset-000000".to_string())));
    EditorHostEventController::new(state, manager)
}

fn percentile(samples: &[u128], percentage: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentage).div_ceil(100);
    sorted[rank - 1]
}
