use super::super::super::backend_refresh::AssetBackendRefreshPlan;

pub(super) fn record_asset_refresh_plan_counters(plan: &AssetBackendRefreshPlan) {
    #[cfg(not(feature = "profiling"))]
    let _ = plan;
    zircon_runtime::profile_counter!(
        "editor",
        "ui.asset_refresh.plan_sync_catalog",
        plan.sync_catalog as u8
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.asset_refresh.plan_sync_resources",
        plan.sync_resources as u8
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.asset_refresh.plan_refresh_selected_asset_details",
        plan.refresh_selected_asset_details as u8
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.asset_refresh.plan_refresh_visible_asset_previews",
        plan.refresh_visible_asset_previews as u8
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.asset_refresh.plan_reload_active_scene",
        plan.reload_active_scene as u8
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.asset_refresh.plan_mark_render_dirty",
        plan.mark_render_dirty as u8
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.asset_refresh.plan_mark_presentation_dirty",
        plan.mark_presentation_dirty as u8
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.asset_refresh.plan_mark_paint_only_dirty",
        plan.mark_paint_only_dirty as u8
    );
}
