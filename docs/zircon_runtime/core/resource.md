---
related_code:
  - zircon_runtime/src/core/resource/mod.rs
  - zircon_runtime/src/core/resource/manager/payload_ops.rs
  - zircon_runtime/src/core/resource/manager/registry_ops.rs
  - zircon_runtime/src/core/resource/manager/runtime_slot.rs
  - zircon_runtime/src/core/resource/runtime.rs
  - zircon_runtime/src/core/resource/tests.rs
  - zircon_runtime_interface/src/resource/resource_record.rs
  - zircon_runtime_interface/src/resource/state.rs
  - zircon_runtime_interface/src/resource/diagnostic.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_pipeline.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
implementation_files:
  - zircon_runtime/src/core/resource/manager/payload_ops.rs
  - zircon_runtime/src/core/resource/manager/registry_ops.rs
  - zircon_runtime/src/core/resource/manager/runtime_slot.rs
  - zircon_runtime/src/core/resource/runtime.rs
  - zircon_runtime_interface/src/resource/resource_record.rs
  - zircon_runtime_interface/src/resource/state.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_pipeline.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - zircon_runtime/src/core/resource/tests.rs::manager_failed_reload_keeps_last_good_payload_and_emits_events
  - zircon_runtime/src/core/resource/tests.rs::resource_state_rejects_error_to_ready_without_reloading
  - zircon_runtime/src/core/resource/tests.rs::resource_state_recovers_from_error_only_through_reloading
  - zircon_runtime/src/core/resource/tests.rs::resource_state_rejects_reload_failure_without_reload_boundary
  - zircon_runtime/src/asset/facade/load_state.rs::tests::asset_load_state_projection_matches_resource_record_matrix
doc_type: module-detail
---

# Runtime Core Resource

`zircon_runtime::core::resource` owns resource identity, records, runtime residency, payload storage, revision events, and the state machine used by asset facade queries. Asset, editor, and render callers should treat typed asset state as a projection over this owner rather than a second source of truth.

## State Ownership

`ResourceRecord.state` is the authoritative import/resource state. `RuntimeResourceState` is the residency/runtime projection for payload availability, reload execution, and lease-driven unloading. `AssetLoadState` reads both layers plus actual payload presence, but it does not mutate either layer.

The Runtime 04 asset alignment slice locks these transitions:

- Missing record to `Ready` is allowed through `register_ready(...)` and emits `Added`.
- `Pending -> Ready` is allowed through `register_ready(...)`.
- `Pending -> Error` is allowed through `fail_reload(...)` for initial import failure rows.
- `Ready -> Reloading` is allowed through `start_reload(...)`.
- `Reloading -> Ready` is allowed through `register_ready(...)`.
- `Reloading -> Error` is allowed through `fail_reload(...)`.
- `Error -> Reloading` is allowed so a failed resource can retry.
- `Error -> Ready` without first entering `Reloading` is rejected: the previous failed record, diagnostics, payload state, runtime state, and revision remain unchanged.
- `Ready -> Error` without first entering `Reloading` is rejected.

Project resource synchronization follows the same rule. If a project resource was previously failed and a later import succeeds, the sync path explicitly calls `start_reload(...)` before `register_ready(...)`; this keeps recovery visible as `Error -> Reloading -> Ready` instead of silently skipping the reload boundary.

## Failure Reasons

Failure reasons are stored in `ResourceRecord.diagnostics`. `ResourceRecord::failure_reason()` returns the first error diagnostic message for records whose state is `ResourceState::Error`, falling back to the first diagnostic when no explicit error severity exists. It returns `None` for non-error records, even if they still carry warnings or importer diagnostics.

This keeps `.zmeta`, artifact records, readiness reports, and facade failure displays on one diagnostic source. Runtime 04 deliberately did not add a separate `failure_reason` field to `ResourceRecord`; doing so would create a second persistence contract that could drift from the diagnostic list.

The Runtime 04 `asset_pipeline_boundary` mirror currently reports `expected_source_file_count = 19`, `expected_guard_file_count = 11`, `worker_diagnostic_count = 5`, `expected_worker_diagnostic_count = 5`, `artifact_store_roundtrip_count = 4`, `expected_artifact_store_roundtrip_count = 4`, `watcher_acceptance_reference_count = 1`, `expected_watcher_acceptance_count = 7`, `artifact_acceptance_reference_count = 3`, `test_anchor_count = 22`, `behavior_test_anchor_count = 18`, `missing_behavior_test_anchors = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `retired_worker_new_references = []`, `old_watch_debounce_references = []`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts` keeps this resource doc aligned with Runtime 04, the runtime index, asset facade/worker/watcher/artifact docs, M0 review, and runtime-interface convergence; broader `asset::` / `worker_pool` Cargo filters remain pending.

## Facade Projection

The asset facade exposes `failure_reason(handle)` on both `Assets<TAsset>` and `ProjectAssetManager`. Those methods validate the requested typed asset kind, read the current `ResourceRecord`, and return the resource failure reason without forcing payload residency or invoking importers.

`AssetLoadState::from_resource(...)` keeps the projection order strict: resource or runtime error maps to `Failed`, resource or runtime reloading maps to `Reloading`, pending/loading maps to `Loading`, ready with payload maps to `Loaded`, and ready without payload maps to `NotLoaded`.

## Validation

The focused resource tests cover the transition boundary itself: failed reload keeps the last good payload, direct `Error -> Ready` registration is rejected, failed resources recover only through `Reloading`, and direct `Ready -> Error` failure is rejected. The asset load-state unit test covers the projection matrix, while facade tests verify that typed failed assets expose the same diagnostic reason through asset-facing APIs.
