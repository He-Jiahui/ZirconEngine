use super::super::backend_refresh::plan_asset_backend_refresh;
use super::super::*;
use counters::record_asset_refresh_plan_counters;
use events::AssetRefreshEvents;
use zircon_runtime_interface::resource::ResourceLocator;

mod apply;
mod counters;
mod events;
mod snapshots;

pub(in crate::ui::retained_host::app) use events::{
    AssetRefreshAccumulator, AssetRefreshQueueAgeState,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AssetMaintenanceFrameUpdate {
    None,
    Immediate,
    At(std::time::Instant),
}

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn refresh_project_assets(
        &mut self,
    ) -> Result<(), String> {
        zircon_runtime::profile_scope!("editor", "retained_host", "refresh_project_assets");
        let (events, backlog_pending) = self.drain_asset_refresh_events();
        let events = self.asset_refresh_accumulator.accumulate(
            events,
            backlog_pending,
            std::time::Instant::now(),
        );
        match asset_maintenance_frame_update(
            backlog_pending,
            self.asset_refresh_accumulator.next_commit_deadline(),
            self.active_scene_reload_admission_retry_deadline(),
        ) {
            AssetMaintenanceFrameUpdate::Immediate => {
                self.ui.request_maintenance_frame_update();
            }
            AssetMaintenanceFrameUpdate::At(deadline) => {
                self.ui.schedule_maintenance_frame_update(deadline);
            }
            AssetMaintenanceFrameUpdate::None => {
                self.ui.clear_maintenance_frame_update();
            }
        }
        let Some(events) = events else {
            return Ok(());
        };
        match visual_asset_cache_refresh(&events) {
            VisualAssetCacheRefresh::None => {}
            VisualAssetCacheRefresh::Paths(paths) => {
                let invalidated =
                    crate::ui::retained_host::host_contract::invalidate_visual_asset_pixel_paths(
                        &paths,
                    );
                let invalidated_svg_trees =
                    crate::ui::retained_host::host_contract::invalidate_svg_tree_paths(&paths);
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.asset_refresh.visual_asset_targeted_invalidation_count",
                    invalidated
                );
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.asset_refresh.svg_tree_targeted_invalidation_count",
                    invalidated_svg_trees
                );
            }
            VisualAssetCacheRefresh::Reconcile => {
                let invalidated =
                    crate::ui::retained_host::host_contract::reconcile_visual_asset_pixel_sources();
                let invalidated_svg_trees =
                    crate::ui::retained_host::host_contract::reconcile_svg_tree_sources();
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.asset_refresh.visual_asset_reconciled_invalidation_count",
                    invalidated
                );
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.asset_refresh.svg_tree_reconciled_invalidation_count",
                    invalidated_svg_trees
                );
            }
            VisualAssetCacheRefresh::All => {
                crate::ui::retained_host::host_contract::invalidate_editor_sprite_atlas_cache();
                crate::ui::retained_host::host_contract::clear_svg_tree_cache();
                crate::ui::retained_host::host_contract::clear_visual_asset_pixels_cache();
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.asset_refresh.visual_asset_full_invalidation_count",
                    1
                );
            }
        }
        if !events.asset_changes.is_empty() {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "asset_refresh_runtime_project"
            );
            let editor_asset_manager = self
                .editor_asset_manager_at_use_point()
                .map_err(|error| error.to_string())?;
            editor_asset_manager.project_runtime_asset_changes(&events.asset_changes);
            editor_asset_manager
                .refresh_from_runtime_project()
                .map_err(|error| error.to_string())?;
        }
        if events.is_empty() {
            return Ok(());
        }

        let selected_asset_uuid = self
            .runtime
            .editor_snapshot()
            .asset_activity
            .selected_asset_uuid;
        let active_scene_uri = self
            .editor_manager
            .active_scene_identity_for_session()
            .map(|identity| identity.scene_uri().to_owned());
        let mut plan = plan_asset_backend_refresh(
            selected_asset_uuid.as_deref(),
            active_scene_uri.as_deref(),
            &events.asset_changes,
            &events.editor_asset_changes,
            &events.resource_changes,
        );
        if events.resource_generation_lagged {
            plan.sync_resources = true;
            plan.mark_presentation_dirty = true;
        }
        if events.active_scene_reload_requested {
            plan.reload_active_scene = true;
            plan.mark_render_dirty = true;
            plan.mark_presentation_dirty = true;
        }
        record_asset_refresh_plan_counters(&plan);
        self.apply_asset_refresh_plan(&plan, &events)
    }
}

fn asset_maintenance_frame_update(
    backlog_pending: bool,
    accumulator_deadline: Option<std::time::Instant>,
    admission_retry_deadline: Option<std::time::Instant>,
) -> AssetMaintenanceFrameUpdate {
    if backlog_pending {
        return AssetMaintenanceFrameUpdate::Immediate;
    }
    match (accumulator_deadline, admission_retry_deadline) {
        (Some(accumulator), Some(admission)) => {
            AssetMaintenanceFrameUpdate::At(accumulator.min(admission))
        }
        (Some(deadline), None) | (None, Some(deadline)) => {
            AssetMaintenanceFrameUpdate::At(deadline)
        }
        (None, None) => AssetMaintenanceFrameUpdate::None,
    }
}

#[cfg(test)]
mod maintenance_frame_update_tests {
    use std::time::{Duration, Instant};

    use super::{asset_maintenance_frame_update, AssetMaintenanceFrameUpdate};

    #[test]
    fn empty_refresh_preserves_active_scene_retry_deadline() {
        let retry_deadline = Instant::now() + Duration::from_millis(128);

        assert_eq!(
            asset_maintenance_frame_update(false, None, Some(retry_deadline)),
            AssetMaintenanceFrameUpdate::At(retry_deadline)
        );
    }

    #[test]
    fn earliest_asset_owner_deadline_drives_the_shared_maintenance_slot() {
        let now = Instant::now();
        let accumulator_deadline = now + Duration::from_millis(32);
        let retry_deadline = now + Duration::from_millis(128);

        assert_eq!(
            asset_maintenance_frame_update(false, Some(accumulator_deadline), Some(retry_deadline)),
            AssetMaintenanceFrameUpdate::At(accumulator_deadline)
        );
    }
}

#[derive(Debug, PartialEq, Eq)]
enum VisualAssetCacheRefresh {
    None,
    Paths(Vec<String>),
    Reconcile,
    All,
}

fn visual_asset_cache_refresh(events: &AssetRefreshEvents) -> VisualAssetCacheRefresh {
    if events_reference_sprite_atlas(events) {
        return VisualAssetCacheRefresh::All;
    }
    if events.resource_generation_lagged {
        return VisualAssetCacheRefresh::Reconcile;
    }
    let mut paths = Vec::new();
    for change in &events.asset_changes {
        push_visual_asset_locator(&mut paths, &change.uri);
        if let Some(previous_uri) = &change.previous_uri {
            push_visual_asset_locator(&mut paths, previous_uri);
        }
    }
    for change in &events.editor_asset_changes {
        if let Some(locator) = change
            .locator
            .as_deref()
            .filter(|path| path_is_visual_asset(path))
        {
            paths.push(locator.to_owned());
        }
    }
    for change in &events.resource_changes {
        if let Some(locator) = &change.locator {
            push_visual_asset_locator(&mut paths, locator);
        }
        if let Some(previous_locator) = &change.previous_locator {
            push_visual_asset_locator(&mut paths, previous_locator);
        }
    }
    paths.sort();
    paths.dedup();
    if paths.is_empty() {
        VisualAssetCacheRefresh::None
    } else if paths.iter().any(|path| path_is_sprite_atlas_source(path)) {
        VisualAssetCacheRefresh::All
    } else {
        VisualAssetCacheRefresh::Paths(paths)
    }
}

fn events_reference_sprite_atlas(events: &AssetRefreshEvents) -> bool {
    events.asset_changes.iter().any(|change| {
        path_is_sprite_atlas_source(change.uri.path())
            || change
                .previous_uri
                .as_ref()
                .is_some_and(|locator| path_is_sprite_atlas_source(locator.path()))
    }) || events.editor_asset_changes.iter().any(|change| {
        change
            .locator
            .as_deref()
            .is_some_and(path_is_sprite_atlas_source)
    }) || events.resource_changes.iter().any(|change| {
        change
            .locator
            .as_ref()
            .is_some_and(|locator| path_is_sprite_atlas_source(locator.path()))
            || change
                .previous_locator
                .as_ref()
                .is_some_and(|locator| path_is_sprite_atlas_source(locator.path()))
    })
}

fn locator_is_visual_asset(locator: &ResourceLocator) -> bool {
    path_is_visual_asset(locator.path())
}

fn push_visual_asset_locator(paths: &mut Vec<String>, locator: &ResourceLocator) {
    if locator_is_visual_asset(locator) {
        paths.push(locator.path().to_owned());
    }
}

fn path_is_sprite_atlas_source(path: &str) -> bool {
    let path = path.replace('\\', "/").to_ascii_lowercase();
    path.contains("editor-sprite-atlases/") || path.ends_with("editor-sprite-atlases")
}

fn path_is_visual_asset(path: &str) -> bool {
    let path = path
        .split_once('#')
        .map_or(path, |(path, _)| path)
        .to_ascii_lowercase();
    [".svg", ".png", ".jpg", ".jpeg", ".webp", ".bmp", ".gif"]
        .iter()
        .any(|extension| path.ends_with(extension))
}

#[cfg(test)]
mod cache_invalidation_tests {
    use super::{
        path_is_sprite_atlas_source, path_is_visual_asset, visual_asset_cache_refresh,
        AssetRefreshEvents, VisualAssetCacheRefresh,
    };
    use zircon_runtime::resource::ResourceEvent;
    use zircon_runtime_interface::resource::{ResourceEventKind, ResourceId, ResourceKind};

    #[test]
    fn visual_asset_detection_ignores_non_image_resource_churn() {
        assert!(!path_is_visual_asset("models/cube.mesh"));
        assert!(!path_is_visual_asset("scenes/main.scene.toml"));
        assert!(!path_is_visual_asset(".zircon/cache/assets/chunks/01.bin"));
    }

    #[test]
    fn visual_asset_detection_accepts_supported_image_sources() {
        assert!(path_is_visual_asset("icons/Save.SVG"));
        assert!(path_is_visual_asset("textures/albedo.png#preview"));
    }

    #[test]
    fn sprite_atlas_products_keep_the_conservative_full_invalidation_path() {
        assert!(path_is_sprite_atlas_source(
            ".zircon/cache/editor-sprite-atlases/icons.png"
        ));
        assert!(path_is_sprite_atlas_source(
            "editor-sprite-atlases/icons.toml"
        ));
        assert!(!path_is_sprite_atlas_source("assets/icons/save.svg"));
    }

    #[test]
    fn resource_stream_lag_reconciles_resident_sources_instead_of_clearing_all_caches() {
        let events = AssetRefreshEvents {
            resource_generation_lagged: true,
            ..AssetRefreshEvents::default()
        };

        assert_eq!(
            visual_asset_cache_refresh(&events),
            VisualAssetCacheRefresh::Reconcile
        );
    }

    #[test]
    fn unlocated_runtime_texture_does_not_invalidate_file_backed_visual_assets() {
        let events = AssetRefreshEvents {
            resource_changes: vec![ResourceEvent {
                kind: ResourceEventKind::Updated,
                resource_kind: ResourceKind::Texture,
                id: ResourceId::new(),
                locator: None,
                previous_locator: None,
                revision: 1,
            }],
            ..AssetRefreshEvents::default()
        };

        assert_eq!(
            visual_asset_cache_refresh(&events),
            VisualAssetCacheRefresh::None
        );
    }

    #[test]
    fn resource_stream_lag_reconciles_even_with_unlocated_runtime_texture_churn() {
        let events = AssetRefreshEvents {
            resource_changes: vec![ResourceEvent {
                kind: ResourceEventKind::Updated,
                resource_kind: ResourceKind::Texture,
                id: ResourceId::new(),
                locator: None,
                previous_locator: None,
                revision: 1,
            }],
            resource_generation_lagged: true,
            ..AssetRefreshEvents::default()
        };

        assert_eq!(
            visual_asset_cache_refresh(&events),
            VisualAssetCacheRefresh::Reconcile
        );
    }
}
