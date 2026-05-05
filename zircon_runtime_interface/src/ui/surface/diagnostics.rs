use serde::{Deserialize, Serialize};

use crate::ui::event_ui::{UiNodeId, UiNodePath, UiTreeId};
use crate::ui::layout::UiFrame;
use crate::ui::tree::{UiDirtyFlags, UiInputPolicy, UiVisibility};

use super::{UiFocusState, UiHitTestQuery, UiRenderCommandKind};

pub const UI_SURFACE_DEBUG_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiSurfaceDebugOptions {
    pub overdraw_sample_cell_size: f32,
    pub include_command_records: bool,
    pub include_hit_cells: bool,
    pub include_overdraw_cells: bool,
    pub include_overlay_primitives: bool,
}

impl Default for UiSurfaceDebugOptions {
    fn default() -> Self {
        Self {
            overdraw_sample_cell_size: 32.0,
            include_command_records: true,
            include_hit_cells: true,
            include_overdraw_cells: true,
            include_overlay_primitives: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiSurfaceDebugCaptureContext {
    pub schema_version: u32,
    pub surface_name: Option<String>,
    pub source_asset: Option<String>,
    pub frame_index: Option<u64>,
    pub captured_at_millis: Option<u64>,
    pub selected_node: Option<UiNodeId>,
    pub pick_query: Option<UiHitTestQuery>,
}

impl Default for UiSurfaceDebugCaptureContext {
    fn default() -> Self {
        Self {
            schema_version: UI_SURFACE_DEBUG_SCHEMA_VERSION,
            surface_name: None,
            source_asset: None,
            frame_index: None,
            captured_at_millis: None,
            selected_node: None,
            pick_query: None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiSurfaceDebugSnapshot {
    #[serde(default)]
    pub capture: UiSurfaceDebugCaptureContext,
    pub tree_id: UiTreeId,
    pub roots: Vec<UiNodeId>,
    pub nodes: Vec<UiWidgetReflectorNode>,
    pub rebuild: UiSurfaceRebuildDebugStats,
    pub render: UiRenderDebugStats,
    pub hit_test: UiHitGridDebugStats,
    pub overdraw: UiOverdrawDebugStats,
    pub focus_state: UiFocusState,
    #[serde(default)]
    pub invalidation: UiInvalidationDebugReport,
    #[serde(default)]
    pub damage: UiDamageDebugReport,
    #[serde(default)]
    pub events: Vec<UiDebugEventRecord>,
    #[serde(default)]
    pub overlay_primitives: Vec<UiDebugOverlayPrimitive>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiWidgetReflectorNode {
    pub node_id: UiNodeId,
    pub node_path: UiNodePath,
    pub parent: Option<UiNodeId>,
    pub children: Vec<UiNodeId>,
    pub frame: UiFrame,
    pub clip_frame: UiFrame,
    pub z_index: i32,
    pub paint_order: u64,
    pub visibility: UiVisibility,
    pub input_policy: UiInputPolicy,
    pub enabled: bool,
    pub clickable: bool,
    pub hoverable: bool,
    pub focusable: bool,
    pub control_id: Option<String>,
    pub render_command_count: usize,
    pub hit_entry_count: usize,
    pub hit_cell_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiRenderDebugStats {
    pub command_count: usize,
    pub group_count: usize,
    pub quad_count: usize,
    pub text_count: usize,
    pub image_count: usize,
    pub clipped_command_count: usize,
    pub opaque_command_count: usize,
    pub translucent_command_count: usize,
    pub material_batch_count: usize,
    pub estimated_draw_calls: usize,
    pub material_batches: Vec<UiMaterialBatchDebugStat>,
    #[serde(default)]
    pub command_records: Vec<UiRenderCommandDebugRecord>,
    #[serde(default)]
    pub measured: Option<UiBackendRenderDebugStats>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSurfaceRebuildDebugStats {
    pub dirty_flags: UiDirtyFlags,
    pub dirty_node_count: usize,
    pub layout_recomputed: bool,
    pub arranged_rebuilt: bool,
    pub hit_grid_rebuilt: bool,
    pub render_rebuilt: bool,
    pub arranged_node_count: usize,
    pub render_command_count: usize,
    pub hit_grid_entry_count: usize,
    pub hit_grid_cell_count: usize,
    pub layout_elapsed_micros: u64,
    pub arranged_elapsed_micros: u64,
    pub hit_grid_elapsed_micros: u64,
    pub render_elapsed_micros: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiMaterialBatchDebugStat {
    pub key: String,
    pub break_reason: String,
    pub command_kind: UiRenderCommandKind,
    pub command_count: usize,
    pub clipped_command_count: usize,
    pub node_ids: Vec<UiNodeId>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiRenderCommandDebugRecord {
    pub command_id: u64,
    pub node_id: UiNodeId,
    pub kind: UiRenderCommandKind,
    pub frame: UiFrame,
    pub clip_frame: Option<UiFrame>,
    pub visible_frame: Option<UiFrame>,
    pub z_index: i32,
    pub paint_order: u64,
    pub opacity: f32,
    pub material_key: String,
    pub batch_key: String,
    pub batch_break_reason: String,
    pub estimated_draw_calls: usize,
    pub text_summary: Option<String>,
    pub image_summary: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBackendRenderDebugStats {
    pub submitted_draw_calls: Option<u64>,
    pub pipeline_switches: Option<u64>,
    pub texture_switches: Option<u64>,
    pub glyph_batches: Option<u64>,
    pub clipped_batches: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiHitGridDebugStats {
    pub entry_count: usize,
    pub cell_count: usize,
    pub occupied_cell_count: usize,
    pub max_entries_per_cell: usize,
    pub average_entries_per_occupied_cell: f32,
    #[serde(default)]
    pub cell_records: Vec<UiHitGridCellDebugRecord>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiHitGridCellDebugRecord {
    pub cell_id: u64,
    pub bounds: UiFrame,
    pub entry_indices: Vec<usize>,
    pub entry_node_ids: Vec<UiNodeId>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiOverdrawDebugStats {
    pub sample_cell_size: f32,
    pub bounds: UiFrame,
    pub columns: u32,
    pub rows: u32,
    pub covered_cells: usize,
    pub overdrawn_cells: usize,
    pub max_layers: usize,
    pub total_layer_samples: usize,
    #[serde(default)]
    pub cells: Vec<UiOverdrawCellDebugRecord>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiOverdrawCellDebugRecord {
    pub cell_id: u64,
    pub bounds: UiFrame,
    pub layer_count: usize,
    pub node_ids: Vec<UiNodeId>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiInvalidationDebugReport {
    pub rebuild: UiSurfaceRebuildDebugStats,
    pub dirty_flags: UiDirtyFlags,
    pub dirty_node_count: usize,
    pub recompute_reasons: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiDamageDebugReport {
    pub damage_region: Option<UiFrame>,
    pub painted_pixels: Option<u64>,
    pub full_paint_count: Option<u64>,
    pub region_paint_count: Option<u64>,
    pub total_painted_pixels: Option<u64>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiDebugEventRecord {
    pub event_id: u64,
    pub kind: String,
    pub node_id: Option<UiNodeId>,
    pub route: Vec<UiNodeId>,
    pub summary: String,
    pub handled: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiDebugOverlayPrimitive {
    pub kind: UiDebugOverlayPrimitiveKind,
    pub node_id: Option<UiNodeId>,
    pub frame: UiFrame,
    pub label: Option<String>,
    pub severity: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiDebugOverlayPrimitiveKind {
    SelectedFrame,
    ClipFrame,
    HitCell,
    HitPath,
    RejectedBounds,
    OverdrawCell,
    MaterialBatchBounds,
    DamageRegion,
}
