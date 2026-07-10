---
related_code:
  - zircon_runtime/src/core/resource/mod.rs
  - zircon_runtime/src/core/resource/registry.rs
  - zircon_runtime/src/core/resource/manager/resource_manager.rs
  - zircon_runtime/src/core/resource/manager/registry_export.rs
  - zircon_runtime/src/core/resource/manager/payload_ops.rs
  - zircon_runtime/src/core/resource/manager/registry_ops.rs
  - zircon_runtime/src/core/resource/manager/lease_ops.rs
  - zircon_runtime/src/core/resource/manager/events.rs
  - zircon_runtime/src/core/resource/manager/runtime_slot.rs
  - zircon_runtime/src/core/resource/snapshot.rs
  - zircon_runtime/src/core/resource/runtime.rs
  - zircon_runtime/src/core/resource/snapshot.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry/tests.rs
  - zircon_runtime/src/asset/tests/pipeline/manager/resource_revisions.rs
  - zircon_runtime/src/core/framework/error.rs
  - zircon_runtime/src/core/resource/tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings.rs
  - zircon_runtime_interface/src/resource/resource_record.rs
  - zircon_runtime_interface/src/resource/state.rs
  - zircon_runtime_interface/src/resource/diagnostic.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_pipeline.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_anchor_inventory.py
implementation_files:
  - zircon_runtime/src/core/framework/error.rs
  - zircon_runtime/src/core/resource/registry.rs
  - zircon_runtime/src/core/resource/manager/resource_manager.rs
  - zircon_runtime/src/core/resource/manager/registry_export.rs
  - zircon_runtime/src/core/resource/manager/payload_ops.rs
  - zircon_runtime/src/core/resource/manager/registry_ops.rs
  - zircon_runtime/src/core/resource/manager/lease_ops.rs
  - zircon_runtime/src/core/resource/manager/events.rs
  - zircon_runtime/src/core/resource/manager/runtime_slot.rs
  - zircon_runtime/src/core/resource/runtime.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry/tests.rs
  - zircon_runtime_interface/src/resource/resource_record.rs
  - zircon_runtime_interface/src/resource/state.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_pipeline.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_anchor_inventory.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/engine-code-structure-convention.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - zircon_runtime/src/core/resource/tests.rs::registry_rename_reports_missing_locator_with_core_error
  - zircon_runtime/src/core/resource/tests.rs::manager_failed_reload_keeps_last_good_payload_and_emits_events
  - zircon_runtime/src/core/resource/tests.rs::resource_state_rejects_error_to_ready_without_reloading
  - zircon_runtime/src/core/resource/tests.rs::resource_state_recovers_from_error_only_through_reloading
  - zircon_runtime/src/core/resource/tests.rs::resource_state_rejects_reload_failure_without_reload_boundary
  - zircon_runtime/src/core/resource/manager/resource_manager.rs::tests::resource_manager_accessors_recover_poisoned_state_locks
  - zircon_runtime/src/core/resource/manager/registry_export.rs::tests::resource_manager_exports_ready_records_for_kind_with_live_revisions
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs::shader_prewarm_resource_registry_overlay_uses_live_resource_manager_shader_revisions
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry/tests.rs::shader_prewarm_resource_registry_overlay_uses_ready_shader_revisions_only
  - zircon_runtime/src/asset/tests/pipeline/manager/resource_revisions.rs::shader_reimport_exports_updated_revision_for_prewarm_registry
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_live_resource_registry.rs::runtime_15_shader_prewarm_live_resource_manager_registry_export_is_wired
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings.rs::review_f6_core_resource_registry_rename_uses_core_error
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs::runtime_15_core_resource_manager_lock_poison_recovery_guard_covers_resource_manager
  - zircon_runtime/src/asset/facade/load_state.rs::tests::asset_load_state_projection_matches_resource_record_matrix
  - zircon_runtime/tests/resource_snapshot_contract.rs::resource_snapshot_never_pairs_a_new_revision_with_an_old_payload
doc_type: module-detail
---

# Runtime Core Resource

Current Runtime 04 owner sync (2026-07-10): `expected_source_file_count = 22`, `expected_guard_file_count = 17`, `test_anchor_count = 24`, `behavior_test_anchor_count = 20`, `missing_behavior_test_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This current child-owner count supersedes the earlier 11-owner historical mirror; core resource state behavior is unchanged.

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

## Atomic Payload/Revision Snapshots

`ResourceManager::snapshot(...)` returns a typed `ResourceSnapshot<T>` containing one immutable payload `Arc` and the exact `ResourceRecord` revision that owns it. `register_ready(...)` publishes the record and payload while holding the registry-then-payload lock order used by snapshot readers, so a reader cannot pair an old payload with a newly published revision. `get(...)` and lease acquisition reuse this snapshot boundary instead of repeating a non-atomic record/payload read sequence.

The snapshot does not make revisions globally monotonic across remove/re-add; a consumer cache must still observe `ResourceEvent::{Added, Updated, Removed}` and invalidate the affected identity. Animation's compiled evaluator does so. `resource_snapshot_never_pairs_a_new_revision_with_an_old_payload` stress-updates one resource while reading snapshots and requires every observed payload version to match its revision.

## Failure Reasons

Failure reasons are stored in `ResourceRecord.diagnostics`. `ResourceRecord::failure_reason()` returns the first error diagnostic message for records whose state is `ResourceState::Error`, falling back to the first diagnostic when no explicit error severity exists. It returns `None` for non-error records, even if they still carry warnings or importer diagnostics.

This keeps `.zmeta`, artifact records, readiness reports, and facade failure displays on one diagnostic source. Runtime 04 deliberately did not add a separate `failure_reason` field to `ResourceRecord`; doing so would create a second persistence contract that could drift from the diagnostic list.

## Registry Rename Errors

F6 core resource registry typed errors closes the remaining `Result<_, String>` rename path inside this owner. `ResourceRegistry::rename(...)` and `ResourceManager::rename(...)` return `CoreResult<ResourceRecord>` and report missing records with `CoreError::MissingResourceRecordForLocator` or `CoreError::MissingResourceRecordForId` instead of `Err(format!())`.

The rename path resolves the source locator and record before mutating locator indexes, so the missing-record error path does not remove the original locator mapping as a side effect. `registry_rename_reports_missing_locator_with_core_error` covers the missing locator branch, while `review_f6_core_resource_registry_rename_uses_core_error` locks the source signature and documentation anchors. Status is recorded as `core_resource_registry_typed_errors_coremin_check_passed`; broader Runtime 02 core/root/generated/export_build_plan/app/editor/plugin gates remain pending.

The Runtime 04 structural mirror is split so resource/asset source-count ownership lives in `asset_pipeline_source_inventory.py`, resource reload and facade anchors live in `asset_pipeline_anchor_inventory.py`, audit reading/risk aggregation lives in the 328-line `asset_pipeline_boundary.py`, and Markdown rendering lives in the 117-line `asset_pipeline_markdown.py`. Current mirror evidence reports `expected_source_file_count = 22`, `expected_guard_file_count = 17`, `worker_diagnostic_count = 7`, `expected_worker_diagnostic_count = 7`, `artifact_store_roundtrip_count = 4`, `expected_artifact_store_roundtrip_count = 4`, `watcher_acceptance_reference_count = 1`, `expected_watcher_acceptance_count = 7`, `artifact_acceptance_reference_count = 3`, `test_anchor_count = 24`, `behavior_test_anchor_count = 20`, `missing_behavior_test_anchors = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `retired_worker_new_references = []`, `retired_worker_request_sender_references = []`, `old_watch_debounce_references = []`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts` keeps this resource doc aligned with Runtime 04, the runtime index, asset facade/worker/watcher/artifact docs, M0 review, and runtime-interface convergence; broader `asset::` / `worker_pool` Cargo filters remain pending.

## Live Ready Record Export

Live ResourceManager shader registry export for Plan 08 is exposed through `ResourceManager::ready_records_for_kind(kind)`. The helper remains resource-generic: it clones records from the authoritative registry, filters to the requested `ResourceKind`, requires `ResourceState::Ready`, drops zero revisions, and sorts by locator/id before returning `ResourceRecord` values for external handoff.

The shader prewarm side consumes this through `shader_resource_records_from_manager(&manager)`, then feeds those records into `ShaderPrewarmResourceRegistryOverlay::from_records(...)` so `.zmeta` shader scans can use live `ResourceRecord.revision` for `ShaderVariantKey.material_revision`. Status is `render_plan08_live_resource_manager_shader_registry_export_focused_tests_passed_renderdoc_deferred`; `resource_manager_exports_ready_records_for_kind_with_live_revisions`, `shader_prewarm_resource_registry_overlay_uses_live_resource_manager_shader_revisions`, and `runtime_15_shader_prewarm_live_resource_manager_registry_export_is_wired` lock the API, prewarm overlay handoff, docs anchors, and 800-line owner budgets.

Plan 08 now also guards the project edit path that feeds that export. `shader_reimport_exports_updated_revision_for_prewarm_registry` edits `res://shaders/pbr.wgsl`, reimports it through `ProjectAssetManager`, and verifies that the updated Ready Shader status is exported by `ready_records_for_kind(ResourceKind::Shader)` with the same `ResourceId` and higher revision. Status is `render_plan08_edited_shader_revision_export_static_guard_cargo_deferred`; direct-binary validation status `render_plan08_edited_shader_revision_export_direct_binary_passed_cargo_wrapper_deferred` records the focused filter passing 1/1. The Selected plugin/source-registry guard Cargo-wrapper backfill status `render_plan08_selected_plugin_source_registry_guards_cargo_wrapper_passed_renderdoc_deferred` records this same edited-revision guard passing 1/1 with 5839 filtered through fresh no-default-features Cargo.

The prewarm overlay now mirrors this Ready-state gate even when records come from caller-provided or auto-exported shader resource registries rather than a live manager helper. `ShaderPrewarmResourceRegistryOverlay::from_records(...)` ignores non-`ResourceState::Ready` shader records before recording revisions, matching the build-tool `_is_usable_shader_record(...)` rule of `kind=Shader`, `state=Ready`, and positive `revision`. Status is `render_plan08_resource_registry_ready_shader_revision_contract_python_static_passed_cargo_deferred`; `shader_prewarm_resource_registry_overlay_uses_ready_shader_revisions_only`, `test_validate_registry_export_contract_rejects_non_ready_report_source_record`, `runtime_15_shader_prewarm_resource_registry_report_correlation_is_wired`, and `runtime_15_shader_prewarm_resource_registry_revision_overlay_is_wired` lock the Rust overlay and Python contract.

This closes only the live `ResourceManager` revision export seam. Full automatic project/plugin shader, shading-model, and geometry-source registry export, real Naga/WGPU prewarm compile, RenderDoc/product capture, and runtime pure-depth DepthPrepass migration remain separate Plan 08 follow-up work.

## Facade Projection

The asset facade exposes `failure_reason(handle)` on both `Assets<TAsset>` and `ProjectAssetManager`. Those methods validate the requested typed asset kind, read the current `ResourceRecord`, and return the resource failure reason without forcing payload residency or invoking importers.

`AssetLoadState::from_resource(...)` keeps the projection order strict: resource or runtime error maps to `Failed`, resource or runtime reloading maps to `Reloading`, pending/loading maps to `Loading`, ready with payload maps to `Loaded`, and ready without payload maps to `NotLoaded`.

## Lock Poison Recovery

Runtime 15 M3 core resource manager lock poison recovery extends the E9/F2 poison-safe lock rule to this core spine owner. `ResourceManager` now owns all access to its registry, payload, runtime-slot, and subscriber locks through `lock_registry_read()`, `lock_registry_write()`, `lock_payloads_read()`, `lock_payloads_write()`, `lock_runtime_read()`, `lock_runtime_write()`, and `lock_subscribers()`. Each helper recovers poisoned locks with `poisoned.into_inner()` instead of panicking.

The split manager owners consume those helpers directly: registry operations write through `lock_registry_write()`, payload operations read/write through payload helpers, lease operations use runtime/payload helpers, and event broadcast/runtime-state operations use subscriber/runtime helpers. This keeps the public `ResourceManager` API unchanged while preventing a previous panic during resource or event handling from permanently crashing later resource access.

`resource_manager_accessors_recover_poisoned_state_locks` deliberately poisons subscribers, registry, payloads, and runtime slots, then verifies subscription, ready registration, payload lookup, lease acquire/release, ref count, and runtime state still work. `runtime_15_core_resource_manager_lock_poison_recovery_guard_covers_resource_manager` rejects regressions to direct lock unwrap or `lock poisoned` panic strings across the manager owner files and mirrors the Runtime 15 status anchors. Status is `runtime_15_core_resource_manager_lock_poison_recovery_static_passed_cargo_deferred`; full `module_convention_gate` and core resource Cargo sweeps remain pending.

## Validation

The focused resource tests cover the transition boundary itself: failed reload keeps the last good payload, direct `Error -> Ready` registration is rejected, failed resources recover only through `Reloading`, and direct `Ready -> Error` failure is rejected. The asset load-state unit test covers the projection matrix, while facade tests verify that typed failed assets expose the same diagnostic reason through asset-facing APIs.
