from __future__ import annotations


HANDLE_STATE_ANCHORS = (
    "impl<TAsset: Asset> Copy for Handle<TAsset>",
    "AssetLoadState::NotLoaded",
    "pub fn failure_reason(&self, handle: Handle<TAsset>) -> Option<String>",
    "record.failure_reason().map(str::to_owned)",
    "pub fn failure_reason(&self) -> Option<&str>",
    "diagnostic.severity == ResourceDiagnosticSeverity::Error",
)

RESOURCE_RELOAD_ANCHORS = (
    "pub fn start_reload(",
    "ResourceMutationBatch::new().start_reload(id, diagnostics)",
    "ResourceState::Ready | ResourceState::Reloading | ResourceState::Error",
    "record.state = ResourceState::Reloading;",
    "entry.runtime_state = Some(RuntimeResourceState::Reloading);",
    "pub fn fail_reload(",
    "ResourceEventKind::ReloadFailed",
    ".is_some_and(|previous| previous.state == ResourceState::Error)",
    "&& !recover_from_error",
    "batch.upsert_imported_erased(metadata, imported.into_resource_data())",
)

WORKER_POOL_ANCHORS = (
    "pub struct AssetWorkerPoolOptions",
    "pub queue_depth: Option<usize>",
    "pub fn new(task_pool: TaskPool, options: AssetWorkerPoolOptions) -> Self",
    "pub fn spawn_worker_pool_with_frame_sampler(",
    "AssetWorkerPoolFrameSampler::from_pool(&pool)",
    "AssetWorkerThreadBudgetSource::TaskPoolIo",
    "task_pool.spawn(move ||",
    "scheduled_jobs: usize",
    '"asset request queue full: {request:?}"',
    "in_flight: HashMap<AssetRequest, Arc<CompletionEntry>>",
    "completed: HashMap<AssetRequest, CompletedEntry>",
    "pub struct AssetWorkerCompletionTicket",
    "Self::ProcessDefault => TaskTimer::process_default()?",
    "pub fn record_diagnostics(&self, store: &mut DiagnosticStore, frame_index: u64)",
    "pub struct AssetWorkerPoolFrameSampler",
    "pub fn sample(&mut self, pool: &AssetWorkerPool) -> AssetWorkerPoolFrameDiagnostics",
    "ASSET_WORKER_BUDGETED_THREADS_DIAGNOSTIC",
)

WORKER_DIAGNOSTIC_ANCHORS = (
    'pub const ASSET_WORKER_IN_FLIGHT_DIAGNOSTIC: &str = "asset.worker.in_flight";',
    'pub const ASSET_WORKER_COMPLETED_DIAGNOSTIC: &str = "asset.worker.completed";',
    'pub const ASSET_WORKER_FAILED_DIAGNOSTIC: &str = "asset.worker.failed";',
    'pub const ASSET_WORKER_QUEUE_PEAK_DIAGNOSTIC: &str = "asset.worker.queue_peak";',
    'pub const ASSET_WORKER_BUDGETED_THREADS_DIAGNOSTIC: &str = "asset.worker.budgeted_threads";',
    'pub const ASSET_WORKER_FRAME_COMPLETED_DIAGNOSTIC: &str =',
    'pub const ASSET_WORKER_FRAME_FAILED_DIAGNOSTIC: &str = "asset.worker.frame_failed";',
)

WATCHER_ANCHORS = (
    "pub const ASSET_WATCH_DEFAULT_DEBOUNCE: Duration = Duration::from_millis(120);",
    "pub struct AssetWatcherOptions",
    "pub debounce: Duration",
    "options.debounce",
    "after(next_wakeup(now, started_at, last_event_at, options))",
    "on_error(AssetWatchError::from_notify_error(",
    "pub struct AssetWatchError",
    "watch_error_subscribers",
    "pub(in crate::asset::pipeline::manager) fn broadcast_watch_error",
)

ARTIFACT_CACHE_ANCHORS = (
    "mod cache_payload;",
    "mod chunk_residency;",
    "mod json_value;",
    "mod mesh;",
    "mod scene;",
    "mod toml_value;",
    "ArtifactCacheAsset",
    "ArtifactCacheJsonValue",
    "ArtifactCacheMeshAsset",
    "ArtifactCacheSceneScriptBindingAsset",
    "ArtifactCacheTomlValue",
    "ArtifactChunkInventory",
    "ArtifactChunkResidencyDiagnostics",
    "SceneScriptBindingAsset",
)

RUNTIME_04_TEST_ANCHORS = (
    "runtime_04_asset_facade_query_surface_stays_manager_owned_and_server_free",
    "dangling_handle_queries_report_not_loaded_instead_of_panicking",
    "failed_asset_exposes_failure_reason_through_facade",
    "resource_state_rejects_error_to_ready_without_reloading",
    "resource_state_recovers_from_error_only_through_reloading",
    "resource_state_rejects_reload_failure_without_reload_boundary",
    "asset_load_state_projection_matches_resource_record_matrix",
    "worker_pool_default_budgets_are_hard_limits",
    "worker_pool_bounded_queue_rejects_overflow_with_explicit_error",
    "concurrent_requests_for_same_asset_share_one_immutable_payload_owner",
    "worker_pool_diagnostics_track_in_flight_and_failure_counts",
    "worker_pool_frame_sampler_records_per_job_completion_deltas",
    "project_asset_manager_uses_the_injected_runtime_io_pool",
    "rapid_successive_writes_within_debounce_window_emit_single_reload",
    "watcher_failure_on_removed_directory_surfaces_observable_error",
    "hot_reload_transitions_through_reloading_state_and_emits_modified_event",
    "reload_failure_emits_reload_failed_event_and_lands_failed_state",
    "artifact_store_roundtrips_scene_assets_with_mesh_references",
    "artifact_store_roundtrips_scene_assets_with_camera_targets",
    "artifact_store_roundtrips_scene_assets_with_physics_components",
    "artifact_store_roundtrips_scene_assets_with_script_binding_json_values",
    "artifact_store_rejects_raw_payload_over_read_budget_before_opening_chunks",
    "artifact_store_lazily_resides_only_requested_compressed_chunks",
    "artifact_store_rejects_a_corrupt_requested_chunk_without_residing_it",
    "artifact_store_unpublished_prepared_generation_keeps_last_good_manifest",
    "asset_worker_pool_matches_runtime_04_and_11_decisions",
    "runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation",
    "runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts",
)

RUNTIME_04_BEHAVIOR_TEST_ANCHORS = (
    "dangling_handle_queries_report_not_loaded_instead_of_panicking",
    "failed_asset_exposes_failure_reason_through_facade",
    "resource_state_rejects_error_to_ready_without_reloading",
    "resource_state_recovers_from_error_only_through_reloading",
    "resource_state_rejects_reload_failure_without_reload_boundary",
    "asset_load_state_projection_matches_resource_record_matrix",
    "worker_pool_default_budgets_are_hard_limits",
    "worker_pool_bounded_queue_rejects_overflow_with_explicit_error",
    "concurrent_requests_for_same_asset_share_one_immutable_payload_owner",
    "worker_pool_diagnostics_track_in_flight_and_failure_counts",
    "worker_pool_frame_sampler_records_per_job_completion_deltas",
    "project_asset_manager_uses_the_injected_runtime_io_pool",
    "rapid_successive_writes_within_debounce_window_emit_single_reload",
    "watcher_failure_on_removed_directory_surfaces_observable_error",
    "hot_reload_transitions_through_reloading_state_and_emits_modified_event",
    "reload_failure_emits_reload_failed_event_and_lands_failed_state",
    "artifact_store_roundtrips_scene_assets_with_mesh_references",
    "artifact_store_roundtrips_scene_assets_with_camera_targets",
    "artifact_store_roundtrips_scene_assets_with_physics_components",
    "artifact_store_roundtrips_scene_assets_with_script_binding_json_values",
    "artifact_store_rejects_raw_payload_over_read_budget_before_opening_chunks",
    "artifact_store_lazily_resides_only_requested_compressed_chunks",
    "artifact_store_rejects_a_corrupt_requested_chunk_without_residing_it",
    "artifact_store_unpublished_prepared_generation_keeps_last_good_manifest",
)

MIRROR_DOCS_GUARD = "runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts"

RUNTIME_04_DOC_ANCHORS = (
    "Runtime 04",
    "runtime_04_asset_facade_query_surface_stays_manager_owned_and_server_free",
    "Loading and status queries are split across manager/facade/service surfaces.",
    "AssetWorkerPoolOptions",
    "asset.worker.budgeted_threads",
    "asset.worker.frame_completed",
    "dangling_handle_queries_report_not_loaded_instead_of_panicking",
    "failed_asset_exposes_failure_reason_through_facade",
    "hot_reload_transitions_through_reloading_state_and_emits_modified_event",
    "reload_failure_emits_reload_failed_event_and_lands_failed_state",
    "artifact_store_roundtrips_scene_assets_with",
    "behavior_test_anchor_count = 24",
    "missing_behavior_test_anchors = []",
    "retired_worker_request_sender_references = []",
    "watcher` 7/7",
    "broader `asset::` / `worker_pool` Cargo filters",
    "runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation",
    MIRROR_DOCS_GUARD,
)

CARGO_GATE_ANCHORS = (
    r".\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter asset`",
    r".\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter asset::",
    r".\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter worker_pool",
    r".\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter watch`",
)
