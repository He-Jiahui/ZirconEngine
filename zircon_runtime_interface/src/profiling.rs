use serde::{Deserialize, Serialize};

use crate::buffer::{ZrByteSlice, ZrOwnedResultV2};
use crate::handles::ZrRuntimeSessionHandle;
use crate::runtime_build_set::ZrRuntimeModuleCompositionReceiptV1;
use crate::status::ZrStatus;

mod session_path;

pub use session_path::{profile_session_basename, PROFILE_SESSION_BASENAME_MAX_BYTES};

pub type ZrRuntimeProfileControlFnV2 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrByteSlice, *mut ZrOwnedResultV2) -> ZrStatus;

pub const PROFILE_TIMELINE_NATIVE_FILE: &str = "timeline.zrtrace.json";
pub const PROFILE_TIMELINE_PERFETTO_FILE: &str = "timeline.perfetto.json";
pub const PROFILE_HOTSPOTS_FILE: &str = "hotspots.json";
pub const PROFILE_COUNTER_HOTSPOTS_FILE: &str = "counter_hotspots.json";
pub const PROFILE_UI_HOTSPOTS_FILE: &str = "ui_hotspots.json";
pub const PROFILE_SUMMARY_FILE: &str = "summary.md";
pub const PROFILE_DEFAULT_OUTPUT_ROOT: &str = "target/zircon-profiles";
pub const PROFILE_DEFAULT_SESSION_ID: &str = "local";
pub const PROFILE_DEFAULT_FRAME_BUDGET_MS: f64 = 16.67;
pub const PROFILE_DEFAULT_MAX_FRAMES: usize = 512;
pub const PROFILE_DEFAULT_MAX_SPANS: usize = 16_384;
pub const PROFILE_DEFAULT_MAX_COUNTERS: usize = 4_096;
pub const PROFILE_CAPTURE_MAX_FRAMES: usize = 4_096;
pub const PROFILE_CAPTURE_MAX_SPANS: usize = 131_072;
pub const PROFILE_CAPTURE_MAX_COUNTERS: usize = 32_768;
pub const PROFILE_CAPTURE_MAX_FRAME_BUDGET_MS: f64 = 60_000.0;

/// Capture options shared by the in-process recorder and the dynamic-runtime ABI.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProfileCaptureConfig {
    pub session_id: String,
    pub output_root: String,
    pub max_frames: usize,
    pub max_spans: usize,
    pub max_counters: usize,
    pub frame_budget_ms: f64,
    pub include_perfetto: bool,
}

impl Default for ProfileCaptureConfig {
    fn default() -> Self {
        Self {
            session_id: PROFILE_DEFAULT_SESSION_ID.to_string(),
            output_root: PROFILE_DEFAULT_OUTPUT_ROOT.to_string(),
            max_frames: PROFILE_DEFAULT_MAX_FRAMES,
            max_spans: PROFILE_DEFAULT_MAX_SPANS,
            max_counters: PROFILE_DEFAULT_MAX_COUNTERS,
            frame_budget_ms: PROFILE_DEFAULT_FRAME_BUDGET_MS,
            include_perfetto: true,
        }
    }
}

impl ProfileCaptureConfig {
    /// 将宿主 JSON 配置归一到默认标识、输出目录、容量和帧预算上限；录制器创建或更新前复用同一资源约束。
    pub fn normalized(mut self) -> Self {
        if self.session_id.trim().is_empty() {
            self.session_id = PROFILE_DEFAULT_SESSION_ID.to_string();
        }
        if self.output_root.trim().is_empty() {
            self.output_root = PROFILE_DEFAULT_OUTPUT_ROOT.to_string();
        }
        self.max_frames = normalized_capacity(
            self.max_frames,
            PROFILE_DEFAULT_MAX_FRAMES,
            PROFILE_CAPTURE_MAX_FRAMES,
        );
        self.max_spans = normalized_capacity(
            self.max_spans,
            PROFILE_DEFAULT_MAX_SPANS,
            PROFILE_CAPTURE_MAX_SPANS,
        );
        self.max_counters = normalized_capacity(
            self.max_counters,
            PROFILE_DEFAULT_MAX_COUNTERS,
            PROFILE_CAPTURE_MAX_COUNTERS,
        );
        if !self.frame_budget_ms.is_finite() || self.frame_budget_ms <= 0.0 {
            self.frame_budget_ms = PROFILE_DEFAULT_FRAME_BUDGET_MS;
        } else {
            self.frame_budget_ms = self
                .frame_budget_ms
                .min(PROFILE_CAPTURE_MAX_FRAME_BUDGET_MS);
        }
        self
    }
}

const fn normalized_capacity(requested: usize, default: usize, hard_max: usize) -> usize {
    if requested == 0 {
        default
    } else if requested > hard_max {
        hard_max
    } else {
        requested
    }
}

/// A transport-safe timeline snapshot containing frame, span, and counter samples.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProfileSnapshot {
    pub session_id: String,
    pub output_root: String,
    pub active: bool,
    pub feature_enabled: bool,
    pub frame_budget_ms: f64,
    pub frames: Vec<ProfileFrameSnapshot>,
    pub spans: Vec<ProfileSpanSnapshot>,
    pub counters: Vec<ProfileCounterSnapshot>,
    #[serde(default)]
    pub recorder_retention: Vec<ProfileRecorderRetentionSnapshot>,
}

impl Default for ProfileSnapshot {
    fn default() -> Self {
        Self {
            session_id: PROFILE_DEFAULT_SESSION_ID.to_string(),
            output_root: PROFILE_DEFAULT_OUTPUT_ROOT.to_string(),
            active: false,
            feature_enabled: false,
            frame_budget_ms: PROFILE_DEFAULT_FRAME_BUDGET_MS,
            frames: Vec::new(),
            spans: Vec::new(),
            counters: Vec::new(),
            recorder_retention: Vec::new(),
        }
    }
}

/// Bounded-history evidence for one sample stream owned by one recorder.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileSampleRetentionSnapshot {
    pub capacity: u64,
    pub written: u64,
    pub overwritten: u64,
    pub retained: u64,
    pub oldest_sequence: Option<u64>,
    pub newest_sequence: Option<u64>,
}

/// Per-recorder retention evidence kept separate when snapshots are merged.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileRecorderRetentionSnapshot {
    pub frames: ProfileSampleRetentionSnapshot,
    pub spans: ProfileSampleRetentionSnapshot,
    pub counters: ProfileSampleRetentionSnapshot,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProfileFrameSnapshot {
    pub stream: String,
    pub name: String,
    pub frame_index: u64,
    pub start_us: u64,
    pub duration_us: u64,
    pub budget_ms: f64,
    pub over_budget: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProfileSpanSnapshot {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub frame_index: Option<u64>,
    pub stream: String,
    pub category: String,
    pub name: String,
    pub path: String,
    pub start_us: u64,
    pub duration_us: u64,
    pub depth: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProfileCounterSnapshot {
    pub stream: String,
    pub name: String,
    pub value: f64,
    pub timestamp_us: u64,
    pub frame_index: Option<u64>,
}

/// Aggregated span cost report generated from a `ProfileSnapshot` export.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HotspotReport {
    pub session_id: String,
    pub frame_budget_ms: f64,
    pub generated_from_span_count: usize,
    pub hotspots: Vec<HotspotEntry>,
    pub hints: Vec<String>,
}

impl Default for HotspotReport {
    fn default() -> Self {
        Self {
            session_id: PROFILE_DEFAULT_SESSION_ID.to_string(),
            frame_budget_ms: PROFILE_DEFAULT_FRAME_BUDGET_MS,
            generated_from_span_count: 0,
            hotspots: Vec::new(),
            hints: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HotspotEntry {
    pub stream: String,
    pub category: String,
    pub name: String,
    pub path: String,
    pub total_us: u64,
    pub avg_us: u64,
    pub p95_us: u64,
    pub max_us: u64,
    pub count: u64,
    pub frame_count: u64,
    pub over_budget_count: u64,
}

/// Generic counter aggregation used to rank measured runtime evidence streams.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CounterHotspotReport {
    pub session_id: String,
    pub frame_budget_ms: f64,
    pub generated_from_counter_count: usize,
    pub counters: Vec<CounterHotspotEntry>,
    pub hints: Vec<String>,
}

impl Default for CounterHotspotReport {
    fn default() -> Self {
        Self {
            session_id: PROFILE_DEFAULT_SESSION_ID.to_string(),
            frame_budget_ms: PROFILE_DEFAULT_FRAME_BUDGET_MS,
            generated_from_counter_count: 0,
            counters: Vec::new(),
            hints: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CounterHotspotEntry {
    pub stream: String,
    pub name: String,
    pub path: String,
    pub total: f64,
    pub avg: f64,
    pub p95: f64,
    pub max: f64,
    pub latest: f64,
    pub count: u64,
    pub frame_count: u64,
}

/// UI-specific counter aggregation used to detect retained-host slow paths.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UiHotspotReport {
    pub session_id: String,
    pub frame_budget_ms: f64,
    pub generated_from_counter_count: usize,
    pub scenarios: Vec<UiScenarioHotspot>,
    pub alerts: Vec<UiHotspotAlert>,
}

impl Default for UiHotspotReport {
    fn default() -> Self {
        Self {
            session_id: PROFILE_DEFAULT_SESSION_ID.to_string(),
            frame_budget_ms: PROFILE_DEFAULT_FRAME_BUDGET_MS,
            generated_from_counter_count: 0,
            scenarios: Vec::new(),
            alerts: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UiScenarioHotspot {
    pub scenario: String,
    pub frame_count: u64,
    pub frame_p95_us: u64,
    pub frame_max_us: u64,
    #[serde(default)]
    pub host_invalidation_transaction_count: u64,
    #[serde(default)]
    pub host_invalidation_scope_count: u64,
    #[serde(default)]
    pub host_invalidation_legacy_dirty_transaction_count: u64,
    #[serde(default)]
    pub host_invalidation_full_target_count: u64,
    #[serde(default)]
    pub host_invalidation_shell_content_target_count: u64,
    #[serde(default)]
    pub host_invalidation_workbench_projection_target_count: u64,
    #[serde(default)]
    pub host_invalidation_view_presentation_target_count: u64,
    #[serde(default)]
    pub host_invalidation_window_metrics_target_count: u64,
    #[serde(default)]
    pub host_invalidation_paint_only_target_count: u64,
    pub slow_path_rebuild_count: u64,
    pub render_path_count: u64,
    pub presentation_rebuild_count: u64,
    #[serde(default)]
    pub asset_editor_pane_presentation_build_count: u64,
    #[serde(default)]
    pub asset_editor_pane_reflection_build_count: u64,
    #[serde(default)]
    pub asset_editor_pane_preview_build_count: u64,
    #[serde(default)]
    pub asset_editor_pane_source_build_count: u64,
    #[serde(default)]
    pub asset_editor_pane_inspector_build_count: u64,
    #[serde(default)]
    pub asset_editor_pane_style_build_count: u64,
    #[serde(default)]
    pub asset_editor_pane_theme_build_count: u64,
    #[serde(default)]
    pub asset_editor_pane_command_availability_build_count: u64,
    pub full_paint_count: u64,
    pub region_paint_count: u64,
    pub painted_pixels: u64,
    #[serde(default)]
    pub presented_surface_pixels: u64,
    pub redraw_full_frame_count: u64,
    pub redraw_region_count: u64,
    pub dirty_layout_count: u64,
    pub dirty_presentation_count: u64,
    pub dirty_render_count: u64,
    pub dirty_paint_only_count: u64,
    pub chrome_snapshot_count: u64,
    pub workbench_model_build_count: u64,
    #[serde(default)]
    pub workbench_hit_index_build_count: u64,
    #[serde(default)]
    pub workbench_hit_index_query_count: u64,
    #[serde(default)]
    pub pane_popup_index_query_count: u64,
    #[serde(default)]
    pub pane_popup_index_candidate_count: u64,
    #[serde(default)]
    pub visual_asset_targeted_invalidation_count: u64,
    #[serde(default)]
    pub svg_tree_targeted_invalidation_count: u64,
    #[serde(default)]
    pub visual_asset_reconcile_source_visit_count: u64,
    #[serde(default)]
    pub visual_asset_reconciled_invalidation_count: u64,
    #[serde(default)]
    pub svg_tree_reconcile_source_visit_count: u64,
    #[serde(default)]
    pub svg_tree_reconciled_invalidation_count: u64,
    #[serde(default)]
    pub visual_asset_full_invalidation_count: u64,
    #[serde(default)]
    pub visual_asset_cache_hit_count: u64,
    #[serde(default)]
    pub visual_asset_cache_miss_count: u64,
    #[serde(default)]
    pub visual_asset_cache_candidate_build_count: u64,
    #[serde(default)]
    pub svg_tree_cache_memory_hit_count: u64,
    #[serde(default)]
    pub svg_tree_cache_miss_count: u64,
    pub chrome_command_full_rebuild_count: u64,
    pub chrome_command_patch_count: u64,
    pub software_fallback_present_count: u64,
    pub gpu_upload_bytes: u64,
    #[serde(default)]
    pub gpu_image_upload_write_count: u64,
    #[serde(default)]
    pub gpu_image_shared_resolve_count: u64,
    #[serde(default)]
    pub gpu_image_shared_upload_write_count: u64,
    #[serde(default)]
    pub gpu_image_shared_upload_bytes: u64,
    #[serde(default)]
    pub gpu_image_shared_resident_bytes: u64,
    #[serde(default)]
    pub gpu_image_cache_key_allocation_count: u64,
    #[serde(default)]
    pub gpu_image_cache_prune_visit_count: u64,
    #[serde(default)]
    pub gpu_image_cache_admission_reject_count: u64,
    #[serde(default)]
    pub gpu_image_invalid_payload_count: u64,
    #[serde(default)]
    pub gpu_image_cache_resident_bytes: u64,
    #[serde(default)]
    pub gpu_image_prepare_command_visit_count: u64,
    #[serde(default)]
    pub gpu_image_prepare_cache_hit_count: u64,
    pub gpu_draw_calls: u64,
    #[serde(default)]
    pub gpu_timestamp_supported_present_count: u64,
    #[serde(default)]
    pub gpu_time_sample_count: u64,
    #[serde(default)]
    pub gpu_time_p50_us: u64,
    #[serde(default)]
    pub gpu_time_p95_us: u64,
    #[serde(default)]
    pub gpu_time_max_us: u64,
    #[serde(default)]
    pub gpu_profile_latency_max_frames: u64,
    pub gpu_visible_commands: u64,
    pub gpu_visible_draw_items: u64,
    #[serde(default)]
    pub gpu_compiled_draw_items: u64,
    pub gpu_batch_layers: u64,
    pub gpu_batch_dependencies: u64,
    #[serde(default)]
    pub gpu_batch_plan_build_count: u64,
    #[serde(default)]
    pub gpu_batch_plan_cache_hit_count: u64,
    #[serde(default)]
    pub gpu_vertex_buffer_create_count: u64,
    #[serde(default)]
    pub gpu_vertex_upload_bytes: u64,
    #[serde(default)]
    pub gpu_retained_cache_copy_bytes: u64,
}

impl UiScenarioHotspot {
    pub fn empty(scenario: impl Into<String>) -> Self {
        Self {
            scenario: scenario.into(),
            frame_count: 0,
            frame_p95_us: 0,
            frame_max_us: 0,
            host_invalidation_transaction_count: 0,
            host_invalidation_scope_count: 0,
            host_invalidation_legacy_dirty_transaction_count: 0,
            host_invalidation_full_target_count: 0,
            host_invalidation_shell_content_target_count: 0,
            host_invalidation_workbench_projection_target_count: 0,
            host_invalidation_view_presentation_target_count: 0,
            host_invalidation_window_metrics_target_count: 0,
            host_invalidation_paint_only_target_count: 0,
            slow_path_rebuild_count: 0,
            render_path_count: 0,
            presentation_rebuild_count: 0,
            asset_editor_pane_presentation_build_count: 0,
            asset_editor_pane_reflection_build_count: 0,
            asset_editor_pane_preview_build_count: 0,
            asset_editor_pane_source_build_count: 0,
            asset_editor_pane_inspector_build_count: 0,
            asset_editor_pane_style_build_count: 0,
            asset_editor_pane_theme_build_count: 0,
            asset_editor_pane_command_availability_build_count: 0,
            full_paint_count: 0,
            region_paint_count: 0,
            painted_pixels: 0,
            presented_surface_pixels: 0,
            redraw_full_frame_count: 0,
            redraw_region_count: 0,
            dirty_layout_count: 0,
            dirty_presentation_count: 0,
            dirty_render_count: 0,
            dirty_paint_only_count: 0,
            chrome_snapshot_count: 0,
            workbench_model_build_count: 0,
            workbench_hit_index_build_count: 0,
            workbench_hit_index_query_count: 0,
            pane_popup_index_query_count: 0,
            pane_popup_index_candidate_count: 0,
            visual_asset_targeted_invalidation_count: 0,
            svg_tree_targeted_invalidation_count: 0,
            visual_asset_reconcile_source_visit_count: 0,
            visual_asset_reconciled_invalidation_count: 0,
            svg_tree_reconcile_source_visit_count: 0,
            svg_tree_reconciled_invalidation_count: 0,
            visual_asset_full_invalidation_count: 0,
            visual_asset_cache_hit_count: 0,
            visual_asset_cache_miss_count: 0,
            visual_asset_cache_candidate_build_count: 0,
            svg_tree_cache_memory_hit_count: 0,
            svg_tree_cache_miss_count: 0,
            chrome_command_full_rebuild_count: 0,
            chrome_command_patch_count: 0,
            software_fallback_present_count: 0,
            gpu_upload_bytes: 0,
            gpu_image_upload_write_count: 0,
            gpu_image_shared_resolve_count: 0,
            gpu_image_shared_upload_write_count: 0,
            gpu_image_shared_upload_bytes: 0,
            gpu_image_shared_resident_bytes: 0,
            gpu_image_cache_key_allocation_count: 0,
            gpu_image_cache_prune_visit_count: 0,
            gpu_image_cache_admission_reject_count: 0,
            gpu_image_invalid_payload_count: 0,
            gpu_image_cache_resident_bytes: 0,
            gpu_image_prepare_command_visit_count: 0,
            gpu_image_prepare_cache_hit_count: 0,
            gpu_draw_calls: 0,
            gpu_timestamp_supported_present_count: 0,
            gpu_time_sample_count: 0,
            gpu_time_p50_us: 0,
            gpu_time_p95_us: 0,
            gpu_time_max_us: 0,
            gpu_profile_latency_max_frames: 0,
            gpu_visible_commands: 0,
            gpu_visible_draw_items: 0,
            gpu_compiled_draw_items: 0,
            gpu_batch_layers: 0,
            gpu_batch_dependencies: 0,
            gpu_batch_plan_build_count: 0,
            gpu_batch_plan_cache_hit_count: 0,
            gpu_vertex_buffer_create_count: 0,
            gpu_vertex_upload_bytes: 0,
            gpu_retained_cache_copy_bytes: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UiHotspotAlert {
    pub scenario: String,
    pub rule: String,
    pub message: String,
}

/// JSON command carried through `ZrRuntimeProfileControlFnV2`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProfileControlCommand {
    StartCapture,
    StopCapture,
    Snapshot,
    RuntimeDiagnosticsSnapshot,
    RuntimeModuleCompositionReceipt,
    ExportReport,
    Reset,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProfileControlRequest {
    pub command: ProfileControlCommand,
    #[serde(default)]
    pub config: Option<ProfileCaptureConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProfileControlResponse {
    pub status: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<ProfileSnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_diagnostics: Option<RuntimeDiagnosticsSnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_composition_receipt: Option<ZrRuntimeModuleCompositionReceiptV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hotspot_report: Option<HotspotReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counter_hotspot_report: Option<CounterHotspotReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui_hotspot_report: Option<UiHotspotReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub export_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
}

impl ProfileControlResponse {
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            status: "ok".to_string(),
            message: message.into(),
            snapshot: None,
            runtime_diagnostics: None,
            module_composition_receipt: None,
            hotspot_report: None,
            counter_hotspot_report: None,
            ui_hotspot_report: None,
            export_dir: None,
            files: Vec::new(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: "error".to_string(),
            message: message.into(),
            snapshot: None,
            runtime_diagnostics: None,
            module_composition_receipt: None,
            hotspot_report: None,
            counter_hotspot_report: None,
            ui_hotspot_report: None,
            export_dir: None,
            files: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeInputDiagnosticsSnapshot {
    /// Viewport resize events successfully applied to the active runtime viewport.
    #[serde(default)]
    pub viewport_resize_count: u64,
    /// Pointer movement events successfully submitted to the active InputManager.
    pub pointer_move_count: u64,
    /// Mouse button press events successfully submitted to the active InputManager.
    pub mouse_button_press_count: u64,
    /// Mouse button release events successfully submitted to the active InputManager.
    pub mouse_button_release_count: u64,
    /// Keyboard press events successfully submitted to the active InputManager.
    pub keyboard_press_count: u64,
    /// Keyboard release events successfully submitted to the active InputManager.
    pub keyboard_release_count: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct RuntimeRenderDeviceDiagnosticsSnapshot {
    pub adapter_name: String,
    pub adapter_device_type: String,
    pub max_bind_groups: u32,
    pub max_texture_dimension_2d: u32,
    pub max_texture_array_layers: u32,
    pub max_sampled_textures_per_shader_stage: u32,
    #[serde(default)]
    pub max_binding_array_elements_per_shader_stage: u32,
    #[serde(default)]
    pub max_binding_array_sampler_elements_per_shader_stage: u32,
    pub max_storage_buffers_per_shader_stage: u32,
    pub max_storage_buffer_binding_size: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct RuntimeDiagnosticsSnapshot {
    pub frame_index: u64,
    /// Project identity observed when the Runtime opened the current project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_identity: Option<String>,
    /// Default scene URI observed when the Runtime opened the current project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene_uri: Option<String>,
    /// Model resource referenced by the loaded scene's canonical Cube mesh.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_model_resource_id: Option<String>,
    /// Material resource referenced by the loaded scene's canonical Cube mesh.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_material_resource_id: Option<String>,
    /// Backend selected by the Runtime for the captured render diagnostics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render_backend_name: Option<String>,
    /// Actual adapter identity and negotiated device limits for the active render backend.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render_device: Option<RuntimeRenderDeviceDiagnosticsSnapshot>,
    /// Product input evidence accepted by the Runtime session's active InputManager.
    #[serde(default)]
    pub input: RuntimeInputDiagnosticsSnapshot,
    pub diagnostic_series: Vec<RuntimeDiagnosticSeriesSnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene_asset_reload: Option<RuntimeSceneAssetReloadDiagnostics>,
    pub profile: ProfileSnapshot,
}

impl RuntimeDiagnosticsSnapshot {
    pub fn series(&self, path: &str) -> Option<&RuntimeDiagnosticSeriesSnapshot> {
        self.diagnostic_series
            .iter()
            .find(|series| series.path == path)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct RuntimeDiagnosticSeriesSnapshot {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    pub subsystem_tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub smoothed: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    pub history: Vec<RuntimeDiagnosticMeasurement>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct RuntimeDiagnosticMeasurement {
    pub frame_index: u64,
    pub value: f64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeSceneAssetReloadDiagnostics {
    pub enabled: bool,
    pub events_drained: usize,
    pub scheduled: usize,
    pub skipped: usize,
    pub skipped_removed: usize,
    pub skipped_reload_failed: usize,
    pub skipped_missing_locator: usize,
    pub skipped_stale_revision: usize,
    pub superseded_pending: usize,
    pub applied: usize,
    pub failed: usize,
    pub stale: usize,
    pub pending: usize,
    pub receiver_disconnected: bool,
}

#[cfg(test)]
#[path = "tests/profiling.rs"]
mod tests;
