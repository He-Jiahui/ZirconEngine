use super::super::super::*;
use crate::ui::host::editor_asset_manager::EditorAssetChangeKind;
use crate::ui::workbench::asset_content_layout::{
    AssetContentLayoutMetrics, AssetContentSurfaceProfile, AssetThumbnailGridMetrics,
};
use crate::ui::workbench::snapshot::{AssetViewMode, AssetWorkspaceSnapshot};
use std::collections::BTreeSet;
use std::ops::Range;
use zircon_runtime::resource::ResourceEvent;
use zircon_runtime_interface::ui::layout::UiSize;

const PREVIEW_OVERSCAN_ROWS: usize = 2;
const UNKNOWN_PREVIEW_VIEWPORT_ITEM_BUDGET: usize = 128;

struct PreviewDemand {
    selected: Option<String>,
    rows: BTreeSet<String>,
}

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn sync_asset_catalog(&mut self) {
        self.sync_asset_catalog_snapshot(&[]);
        self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
    }

    pub(super) fn sync_asset_catalog_snapshot(&mut self, changes: &[EditorAssetChange]) {
        if let Ok(editor_asset_manager) = self.editor_asset_manager_at_use_point() {
            let catalog = editor_asset_manager.catalog_snapshot();
            let exact_changes = !changes.is_empty()
                && changes.iter().all(|change| {
                    change.kind != EditorAssetChangeKind::CatalogChanged && change.uuid.is_some()
                });
            if exact_changes {
                let mut changed_asset_uuids = changes
                    .iter()
                    .filter_map(|change| change.uuid.clone())
                    .collect::<Vec<_>>();
                changed_asset_uuids.sort();
                changed_asset_uuids.dedup();
                self.runtime
                    .sync_asset_catalog_changes(catalog, &changed_asset_uuids);
            } else {
                self.runtime.sync_asset_catalog_data(catalog);
            }
        }
    }

    pub(in crate::ui::retained_host::app) fn sync_asset_resources(&mut self) {
        if self.sync_asset_resources_snapshot(&[], true) {
            self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
        }
    }

    pub(super) fn sync_asset_resources_snapshot(
        &mut self,
        changes: &[ResourceEvent],
        generation_lagged: bool,
    ) -> bool {
        if let Ok(resource_manager) = self.resolve_resource_manager() {
            let resources = resource_manager.resource_management_generation();
            if !changes.is_empty() && !generation_lagged {
                let mut changed_locators = changes
                    .iter()
                    .flat_map(|change| {
                        change
                            .locator
                            .iter()
                            .chain(change.previous_locator.iter())
                            .map(|locator| locator.to_string())
                    })
                    .collect::<Vec<_>>();
                changed_locators.sort();
                changed_locators.dedup();
                return self
                    .runtime
                    .sync_asset_resource_changes(resources, &changed_locators);
            }
            return self.runtime.sync_asset_resources_data(resources);
        }
        false
    }

    pub(in crate::ui::retained_host::app) fn refresh_selected_asset_details(&mut self) {
        let selected_uuid = self
            .runtime
            .editor_snapshot()
            .asset_activity
            .selected_asset_uuid;
        let details = self
            .editor_asset_manager_at_use_point()
            .ok()
            .and_then(|manager| {
                selected_uuid
                    .as_deref()
                    .and_then(|uuid| manager.asset_details(uuid))
            });
        self.runtime.sync_asset_details(details);
    }

    pub(in crate::ui::retained_host::app) fn refresh_visible_asset_previews(&mut self) {
        let Ok(asset_manager) = self.asset_manager_at_use_point() else {
            return;
        };
        if asset_manager.current_project().is_none() {
            return;
        }

        let chrome = self.build_chrome();
        let mut demands = Vec::with_capacity(2);

        if asset_surface_visible(&chrome, ViewContentKind::Assets) {
            let surface = &self.activity_asset_pointer;
            demands.push(preview_demand_for_surface(
                &chrome.asset_activity,
                AssetContentSurfaceProfile::Activity,
                surface.content_size,
                surface.content_state.scroll_offset,
            ));
        }

        if asset_surface_visible(&chrome, ViewContentKind::AssetBrowser) {
            let surface = &self.browser_asset_pointer;
            demands.push(preview_demand_for_surface(
                &chrome.asset_browser,
                AssetContentSurfaceProfile::Browser,
                surface.content_size,
                surface.content_state.scroll_offset,
            ));
        }

        let Ok(editor_asset_manager) = self.editor_asset_manager_at_use_point() else {
            return;
        };
        for uuid in ordered_preview_uuids(demands) {
            let _ = editor_asset_manager.request_preview_refresh(&uuid, true);
        }
    }

    pub(in crate::ui::retained_host::app) fn refresh_scrolled_asset_previews(
        &mut self,
        surface_mode: &str,
        snapshot: &AssetWorkspaceSnapshot,
        pane_size: UiSize,
        scroll_px: f32,
    ) {
        // The committed pointer snapshot contains all browser rows and selection without
        // rebuilding chrome. Activity's pointer projection omits folder rows, so use the
        // full snapshot there to keep the folder-to-item viewport offset correct.
        if surface_mode != "browser" {
            self.refresh_visible_asset_previews();
            return;
        }
        let Ok(asset_manager) = self.asset_manager_at_use_point() else {
            return;
        };
        if asset_manager.current_project().is_none() {
            return;
        }
        let demand = preview_demand_for_surface(
            snapshot,
            AssetContentSurfaceProfile::Browser,
            pane_size,
            scroll_px,
        );
        let Ok(editor_asset_manager) = self.editor_asset_manager_at_use_point() else {
            return;
        };
        for uuid in ordered_preview_uuids([demand]) {
            let _ = editor_asset_manager.request_preview_refresh(&uuid, true);
        }
    }

    pub(super) fn request_asset_preview_paint_only_redraw(&self) {
        let frame = self.ui.get_host_window_bootstrap().shell_frame;
        self.ui.request_redraw_region(frame);
    }
}

fn preview_demand_for_surface(
    snapshot: &AssetWorkspaceSnapshot,
    profile: AssetContentSurfaceProfile,
    pane_size: UiSize,
    scroll_px: f32,
) -> PreviewDemand {
    let mut rows = BTreeSet::new();
    for index in preview_item_range(snapshot, profile, pane_size, scroll_px) {
        if let Some(item) = snapshot.visible_assets.get(index) {
            rows.insert(item.uuid.clone());
        }
    }
    PreviewDemand {
        selected: snapshot.selection.uuid.clone(),
        rows,
    }
}

fn ordered_preview_uuids(demands: impl IntoIterator<Item = PreviewDemand>) -> Vec<String> {
    let demands = demands.into_iter().collect::<Vec<_>>();
    let mut seen = BTreeSet::new();
    let mut ordered = Vec::new();
    for demand in &demands {
        if let Some(uuid) = demand.selected.as_ref() {
            if seen.insert(uuid.clone()) {
                ordered.push(uuid.clone());
            }
        }
    }
    for demand in demands {
        for uuid in demand.rows {
            if seen.insert(uuid.clone()) {
                ordered.push(uuid);
            }
        }
    }
    ordered
}

fn preview_item_range(
    snapshot: &AssetWorkspaceSnapshot,
    profile: AssetContentSurfaceProfile,
    pane_size: UiSize,
    scroll_px: f32,
) -> Range<usize> {
    let item_count = snapshot.visible_assets.len();
    if item_count == 0 {
        return 0..0;
    }
    // The first catalog refresh can precede pane geometry. Admit a bounded initial page;
    // later pointer and native scroll callbacks use their measured pane size and offset.
    if !pane_size.width.is_finite()
        || !pane_size.height.is_finite()
        || pane_size.width <= 0.0
        || pane_size.height <= 0.0
    {
        return 0..item_count.min(UNKNOWN_PREVIEW_VIEWPORT_ITEM_BUDGET);
    }

    let scroll_px = if scroll_px.is_finite() {
        scroll_px.max(0.0)
    } else {
        0.0
    };
    if profile == AssetContentSurfaceProfile::Browser
        && snapshot.view_mode == AssetViewMode::Thumbnail
    {
        let grid = AssetThumbnailGridMetrics::new(pane_size.width, item_count);
        let columns = grid.columns();
        let Some(first) = grid.item_frame(0) else {
            return 0..0;
        };
        let row_stride = grid
            .item_frame(columns)
            .map_or(first.height, |second| second.y - first.y);
        return preview_rows_in_viewport(
            item_count,
            columns,
            first.y,
            row_stride,
            scroll_px,
            scroll_px + pane_size.height,
        );
    }

    let metrics = AssetContentLayoutMetrics::for_surface(profile, snapshot.view_mode);
    let viewport = metrics.viewport_frame(pane_size);
    if viewport.height <= 0.0 {
        return 0..0;
    }
    let folder_count = if profile == AssetContentSurfaceProfile::Activity {
        snapshot.visible_folders.len()
    } else {
        0
    };
    let item_top =
        metrics.first_row_y() + folder_count as f32 * (metrics.folder_height + metrics.row_gap);
    preview_rows_in_viewport(
        item_count,
        1,
        item_top,
        metrics.item_height + metrics.row_gap,
        viewport.y + scroll_px,
        viewport.y + viewport.height + scroll_px,
    )
}

fn preview_rows_in_viewport(
    item_count: usize,
    columns: usize,
    first_row_y: f32,
    row_stride: f32,
    visible_top: f32,
    visible_bottom: f32,
) -> Range<usize> {
    if columns == 0 || row_stride <= 0.0 {
        return 0..0;
    }
    if visible_bottom < first_row_y - PREVIEW_OVERSCAN_ROWS as f32 * row_stride {
        return 0..0;
    }
    let first_row = ((visible_top - first_row_y) / row_stride).floor().max(0.0) as usize;
    let last_row = ((visible_bottom - first_row_y) / row_stride)
        .ceil()
        .max(0.0) as usize;
    let start = first_row
        .saturating_sub(PREVIEW_OVERSCAN_ROWS)
        .saturating_mul(columns)
        .min(item_count);
    let end = last_row
        .saturating_add(PREVIEW_OVERSCAN_ROWS)
        .saturating_mul(columns)
        .min(item_count);
    start..end.max(start)
}

#[cfg(test)]
#[path = "snapshots/tests/preview_demand_tests.rs"]
mod preview_demand_tests;
