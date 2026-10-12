//! Bounded physical Activity rows bound to one immutable logical generation.
use super::{
    compact_file_like_display_name, ActivityContentNodeRole, AssetContentLayoutMetrics,
    AssetContentRect, AssetContentSurfaceProfile, RuntimeFileNameCompaction,
};
use crate::ui::retained_host::measure_runtime_text_width;
use crate::ui::workbench::snapshot::{
    AssetFolderSnapshot, AssetItemSnapshot, AssetViewMode, AssetWorkspaceItemGeneration,
    AssetWorkspaceSnapshot,
};
use std::sync::Arc;
use zircon_runtime_interface::ui::design_tokens::{EditorDensityTokens, EditorTypographyTokens};

// Match the existing Browser and preview demand overscan policy.
pub(crate) const ACTIVITY_OVERSCAN_ROWS: usize = 2;

#[derive(Clone, Debug)]
pub(crate) struct ActivityAssetVirtualization {
    folders: Arc<[AssetFolderSnapshot]>,
    items: AssetWorkspaceItemGeneration,
    selected_folder: Option<String>,
    selected_asset: Option<String>,
    pub(crate) metrics: AssetContentLayoutMetrics,
    pub(crate) view_mode: AssetViewMode,
    pub(crate) panel: AssetContentRect,
    pub(crate) slot_count: usize,
}

#[derive(Clone, Copy)]
pub(crate) enum ActivityAssetRow<'a> {
    Folder(&'a AssetFolderSnapshot),
    Item(&'a AssetItemSnapshot),
}

pub(crate) struct ActivityAssetBinding<'a> {
    pub(crate) logical_index: usize,
    pub(crate) top: f32,
    pub(crate) height: f32,
    pub(crate) selected: bool,
    pub(crate) row: ActivityAssetRow<'a>,
}

impl ActivityAssetBinding<'_> {
    pub(crate) fn name(&self) -> &str {
        match self.row {
            ActivityAssetRow::Folder(row) => &row.display_name,
            ActivityAssetRow::Item(row) => &row.display_name,
        }
    }
    pub(crate) fn extension(&self) -> &str {
        match self.row {
            ActivityAssetRow::Folder(_) => "",
            ActivityAssetRow::Item(row) => &row.extension,
        }
    }
    pub(crate) fn badge(&self) -> &str {
        match self.row {
            ActivityAssetRow::Folder(_) => "DIR",
            ActivityAssetRow::Item(row) => &row.asset_type.badge,
        }
    }
    pub(crate) fn meta(&self) -> String {
        match self.row {
            ActivityAssetRow::Folder(row) => row.recursive_asset_count.to_string(),
            ActivityAssetRow::Item(row) if !row.diagnostics.is_empty() => "!".to_string(),
            ActivityAssetRow::Item(row) => match row.resource_revision {
                Some(revision) => format!("r{revision}"),
                None if row.dirty => "*".to_string(),
                None => "new".to_string(),
            },
        }
    }
    pub(crate) fn meta_tone(&self) -> &'static str {
        match self.row {
            ActivityAssetRow::Item(row) if !row.diagnostics.is_empty() => "danger",
            _ => "muted",
        }
    }
}

impl ActivityAssetVirtualization {
    pub(crate) fn new(snapshot: &AssetWorkspaceSnapshot, panel: AssetContentRect) -> Self {
        let metrics = AssetContentLayoutMetrics::for_surface(
            AssetContentSurfaceProfile::Activity,
            snapshot.view_mode,
        );
        let count = snapshot
            .visible_folders
            .len()
            .saturating_add(snapshot.visible_assets.len());
        let stride = metrics.folder_height.min(metrics.item_height) + metrics.row_gap;
        let slot_count = if panel.height.is_finite()
            && panel.height > 0.0
            && panel.width > 0.0
            && stride > 0.0
        {
            count.min((panel.height / stride).ceil() as usize + ACTIVITY_OVERSCAN_ROWS * 2 + 1)
        } else {
            0
        };
        Self {
            folders: Arc::clone(&snapshot.visible_folders),
            items: snapshot.visible_assets.clone(),
            selected_folder: snapshot.selected_folder_id.clone(),
            selected_asset: snapshot.selected_asset_uuid.clone(),
            metrics,
            view_mode: snapshot.view_mode,
            panel,
            slot_count,
        }
    }

    pub(crate) fn folder_count(&self) -> usize {
        self.folders.len()
    }
    pub(crate) fn logical_count(&self) -> usize {
        self.folders.len().saturating_add(self.items.len())
    }
    pub(crate) fn content_extent(&self) -> f32 {
        self.metrics
            .list_height(self.folders.len(), self.items.len())
    }
    pub(crate) fn shares_generation_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.folders, &other.folders) && self.items.shares_items_with(&other.items)
    }

    pub(crate) fn binding(&self, scroll_px: f32, slot: usize) -> Option<ActivityAssetBinding<'_>> {
        if slot >= self.slot_count {
            return None;
        }
        let scroll = if scroll_px.is_finite() {
            scroll_px.max(0.0)
        } else {
            0.0
        };
        let relative_top = (scroll - self.metrics.first_row_y()).max(0.0);
        let folder_stride = self.metrics.folder_height + self.metrics.row_gap;
        let item_stride = self.metrics.item_height + self.metrics.row_gap;
        let folder_extent = self.folders.len() as f32 * folder_stride;
        let first_visible = if relative_top < folder_extent {
            (relative_top / folder_stride).floor() as usize
        } else {
            self.folders
                .len()
                .saturating_add(((relative_top - folder_extent) / item_stride).floor() as usize)
        };
        let start = first_visible
            .saturating_sub(ACTIVITY_OVERSCAN_ROWS)
            .min(self.logical_count().saturating_sub(self.slot_count));
        // Ring slots keep rows already on screen bound across a one-row scroll.
        let offset = (slot + self.slot_count - start % self.slot_count) % self.slot_count;
        let logical_index = start.checked_add(offset)?;
        let (row, height, row_top, selected) = if logical_index < self.folders.len() {
            let row = self.folders.get(logical_index)?;
            (
                ActivityAssetRow::Folder(row),
                self.metrics.folder_height,
                logical_index as f32 * folder_stride,
                row.selected || self.selected_folder.as_deref() == Some(row.folder_id.as_str()),
            )
        } else {
            let item_index = logical_index.checked_sub(self.folders.len())?;
            let row = self.items.get(item_index)?;
            (
                ActivityAssetRow::Item(row),
                self.metrics.item_height,
                folder_extent + item_index as f32 * item_stride,
                row.selected || self.selected_asset.as_deref() == Some(row.uuid.as_str()),
            )
        };
        Some(ActivityAssetBinding {
            logical_index,
            top: self.panel.y + self.metrics.first_row_y() + row_top,
            height,
            selected,
            row,
        })
    }
}

pub(crate) struct ActivityRowGeometry {
    pub(crate) row: AssetContentRect,
    pub(crate) badge: AssetContentRect,
    pub(crate) name: AssetContentRect,
    pub(crate) meta: AssetContentRect,
}
impl ActivityRowGeometry {
    pub(crate) fn for_role(&self, role: ActivityContentNodeRole) -> AssetContentRect {
        match role {
            ActivityContentNodeRole::Row => self.row,
            ActivityContentNodeRole::Badge | ActivityContentNodeRole::Type => self.badge,
            ActivityContentNodeRole::Name => self.name,
            ActivityContentNodeRole::Meta => self.meta,
        }
    }
}

/// The existing Activity row layout, shared by initial layout and paint-time rebinding.
pub(crate) fn activity_row_geometry(
    panel: AssetContentRect,
    metrics: AssetContentLayoutMetrics,
    view_mode: AssetViewMode,
    top: f32,
    height: f32,
    meta: &str,
) -> ActivityRowGeometry {
    let density = EditorDensityTokens::workbench_dense();
    let typography = EditorTypographyTokens::workbench_default();
    let row = AssetContentRect {
        x: panel.x + metrics.row_x,
        y: top,
        width: metrics.row_width(panel.width),
        height,
    };
    let badge_extent = match view_mode {
        AssetViewMode::List => density.row_height,
        AssetViewMode::Thumbnail => (height - density.gap_medium * 2.0).max(density.row_height),
    };
    let badge = AssetContentRect {
        x: row.x + density.gap_small,
        y: top + (height - badge_extent) * 0.5,
        width: badge_extent,
        height: badge_extent,
    };
    let meta_width = (measure_runtime_text_width(meta, typography.caption_size)
        + density.gap_medium)
        .max(density.row_height * 0.75);
    let meta_x = row.x + row.width - density.gap_small - meta_width;
    let name_x = badge.x + badge_extent + density.gap_medium;
    let label_height = typography.body_size * typography.line_height;
    let label_y = top + (height - label_height) * 0.5;
    ActivityRowGeometry {
        row,
        badge,
        name: AssetContentRect {
            x: name_x,
            y: label_y,
            width: (meta_x - density.gap_small - name_x).max(0.0),
            height: label_height,
        },
        meta: AssetContentRect {
            x: meta_x,
            y: label_y,
            width: meta_width,
            height: label_height,
        },
    }
}

pub(crate) fn activity_compact_name(name: &str, extension: &str, width: f32) -> String {
    compact_file_like_display_name(
        name,
        extension,
        RuntimeFileNameCompaction {
            max_width: width,
            font_size: EditorTypographyTokens::workbench_default().body_size,
            min_prefix_chars: 4,
            min_tail_stem_chars: 3,
            preferred_tail_stem_chars: 6,
        },
    )
}
