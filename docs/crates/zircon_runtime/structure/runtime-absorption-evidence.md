---
related_code:
plan_sources:
  - docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
  - docs/plans/zircon_runtime/runtime/15/2026-07-11-stable-evidence-owner-hard-cutover.md
tests:
  - tools/tests/test_runtime_absorption_current_source_fixture.py
  - runtime_15_structure_guards_use_durable_evidence_not_session_notes
  - runtime_architecture_implementation_output_is_tracked_plan_evidence
implementation_files:
---

# Runtime absorption evidence ownership

`zircon_runtime` 的 compile-time absorption guards 只能依赖 Git 跟踪的生产源码、模块文档、编号计划产出或归档产出。`.codex/sessions` 是会话临时状态，不是干净 checkout 可重建的测试 fixture，禁止通过 `include_str!` 或其他文件读取重新引入。

Runtime architecture implementation evidence 的唯一共享编译期 owner 是 `current_source_fixture.rs`。它直接读取 tracked Runtime 15 code-structure output archive；已经通过 `runtime_numbered_archive_sources()` 聚合编号归档的 plan-status guards 继续使用该聚合 owner，不重复挂载同一 evidence source。

`runtime_15_structure_guards_use_durable_evidence_not_session_notes` 从 `tests/runtime_absorption` 根递归扫描全部 Rust guard，而不是只扫描 `structure_convention` 子树。静态 Python regression 同时锁定共享 owner、15 个普通直接 consumer、7 个必须向 `concat!` 提供字面量的直接 `include_str!` consumer、5 个编号归档聚合 consumer，以及全树零 `.codex/sessions` 路径。

该切换只改变测试证据所有权，不修改 Render05 ShadowAtlas sampler、WGPU binding、渲染行为或 guard 的断言锚点。

## Current independent runtime index

Current task-domain policy: [JobSystem](../core/job_system.md).

此处仅展示当前现状的概述；domain worklists, module policy, M0 review and interface convergence remain independent mirrors. This index records their declared scope and retained validation targets, not a fresh whole-engine result.

| Domain | Independent role |
|---|---|
| 01 | Dependency/backend governance and tech_stack_boundary |
| 02 | Core root/public surface and generated-code ownership |
| 03 | One time-advance authority, schedule/extract ordering and parallel reports |
| 04 | Asset/resource/worker/watcher gates; broader Cargo validation pending |
| 06 | Native namespace, descriptor/ABI and lifecycle validation |
| 08 | ECS identity/storage/commands and change-tick validation |
| 09 | UI interface/layout/input ownership; active lane and Cargo gates pending |
| 10 | Dynamic session ABI and event packaging |
| 11 | Explicit task pool/lifecycle, scope, stream and result budgets |
| 13 | Script bindings and host ledger; concrete VM backend belongs to its plugin |
| 15 | Structure conventions, typed errors and independent status mirrors |

| Domain | Declared worklist identity | Cross-domain scope | State |
|---|---|---|---|
| 09 | `09-ui-subsystem-architecture.md` | Runtime 02、Runtime 03 | in_progress: UI validation pending |
| 15 | `15-code-structure-and-module-conventions.md` | Runtime 01 至 Runtime 14 | in_progress: current source/guard work and Cargo validation pending |

### Retained mirror targets and dated evidence

Each metric below keeps its original source/date provenance. The source-specific UI, ECS, native and task inventories below are independent snapshots, not interchangeable current measurements. Later source synchronization does not erase earlier evidence and does not close Cargo/product gates.

Runtime04 keeps source25 and later26 records distinct and preserves all nine current mirror roles. Runtime06 retains native-only public ownership, the exact six activation types, five App consumers, panic/unknown-ABI/hot-reload/fallback gates, and real VM/native test mounts. Runtime11 uses explicit pools, bounded stream/result admission and close/join; WgpuRenderFramework receives an explicit TaskPool, not a private renderer pool.

The literal command identities below belong to historical/guard provenance. In particular the old workspace/all-target plugin command is not an executable current recipe: actual validation must use finite packages/features/targets/profile and independent archive admission. No current source is replaced by historical readback.

### Source and guard ownership

The finite source, guard and provenance inventory for these responsibilities is:

- `zircon_app/src/entry/product_composition/composition.rs`, `zircon_app/src/entry/product_composition/ownership.rs`, `zircon_app/src/entry/product_composition/request.rs`.
- `zircon_app/src/entry/tests/profile_bootstrap.rs`, `zircon_app/src/entry/export_bootstrap.rs`, `src/script/vm/tests/cases.rs`.
- `ecs_kernel_data/inventory.rs`, `job_system/mirror_docs.rs`, `zircon_runtime/src/ui`.
- `job_scheduler.rs`, `diagnostics.rs`.

### Validation interfaces

Keep these existing validation and domain interfaces in the gate:

- `runtime_06_plugin_surface_lifecycle_mirror_docs_match_structure_audit_counts`, `runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts`, `runtime_08_ecs_kernel_data_mirror_docs_match_structure_audit_counts`, `runtime_06_native_loader_tests_use_isolated_plugin_native_namespace`.
- `runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts`, `runtime_06_vm_lifecycle_fallback_failure_tests_are_folder_backed`, `runtime_11_job_system_mirror_docs_match_structure_audit_counts`, `core_spine_root_generated_boundary`.
- `plugin_surface_lifecycle_boundary`, `schedule_frame_loop_boundary`, `ui_architecture_boundary`, `ecs_kernel_data_boundary`.
- `asset_pipeline_boundary`, `runtime_diagnostics`, `zr_vm_language`.

### Recorded targets

These retained targets belong to the original source snapshots, not a new measurement. The source dates/identities below keep distinct inventories separate.

- `component_storage_private_reexport_anchors = 9/9`; source 2026-07-10:L231.
- `public native namespace re-export locations 2/2`; source eba15ac42775:L407.
- `retired_worker_request_sender_references = []`; source eba15ac42775:L227.
- `native loader V1/V2 implementation files 0/0`; source 2026-07-22:L382.
- `expected_artifact_store_roundtrip_count = 4`; source eba15ac42775:L227.
- dynamic-session `.tick_time(...)` calls 1/1; source eba15ac42775:L225.
- `root public native re-export locations 0/0`; source 2026-07-22:L382.
- `unclassified native namespace symbols 0/0`; source 2026-07-22:L382.
- `expected_taffy_production_file_count = 16`; source 2026-09-01:L106.
- `expected_legacy_production_file_count = 0`; source 2026-09-01:L106.
- `artifact_acceptance_reference_count = 3`; source eba15ac42775:L227.
- `watcher_acceptance_reference_count = 1`; source eba15ac42775:L227.
- `surface_source_module_entry_count = 43`; source 2026-09-30/2026-09-10:L110.
- `native test namespace import files 3/3`; source 2026-07-22:L382.
- `zircon_plugins` V1/V2 usage files 0/0; source 2026-07-22:L382.
- `expected_watcher_acceptance_count = 7`; source eba15ac42775:L227.
- `crate-visible graphics alias debt 0/0`; source 2026-08-24:L306.
- `unclassified native root symbols 0/0`; source 2026-07-22:L382.
- `native_namespace_reexport_count = 92`; source eba15ac42775:L407.
- `fallback lifecycle failure tests 4/4`; source 2026-07-22:L382.
- `expected_worker_diagnostic_count = 7`; source eba15ac42775:L227.
- `SystemStage` count and variants 9/9; source eba15ac42775:L225.
- `expected_legacy_production_hits = 0`; source 2026-09-01:L106.
- `retired_worker_new_references = []`; source eba15ac42775:L227.
- `old_watch_debounce_references = []`; source eba15ac42775:L227.
- `native namespace symbol groups 6/6`; source 2026-08-24:L386.
- `missing_behavior_test_anchors = []`; source 2026-07-19:L138.
- `component_identity_anchors = 18/18`; source 2026-07-10:L231.
- `runtime root public modules 21/21`; source 2026-08-24:L306.
- `resource_identity_anchors = 12/12`; source 2026-07-10:L231.
- `native test root import leaks 0/0`; source 2026-07-22:L382.
- `export_build_plan V1/V2 usage 0/0`; source 2026-07-22:L382.
- `expected_surface_entry_count = 44`; source 2026-09-01:L106.
- `runtime_08_guard_anchors = 21/21`; source 2026-07-10:L231.
- `pending_cargo_gate_anchors = 6/6`; source 2026-07-10:L231.
- `native namespace re-export 92/92`; source eba15ac42775:L407.
- `generated export templates 10/10`; source 2026-08-24:L306.
- `entity_lifecycle_anchors = 10/10`; source 2026-07-10:L231.
- `deferred_command_anchors = 11/11`; source 2026-07-10:L231.
- `missing_guard_test_anchors = []`; source 2026-08-24:L306.
- `missing_cargo_gate_anchors = []`; source 2026-09-01:L106.
- `expected_source_file_count = 49`; source 2026-09-01:L106.
- `expected_source_file_count = 26`; source 2026-08-14:L388.
- `expected_source_file_count = 25`; source eba15ac42775:L227.
- `expected_source_file_count = 20`; source 2026-08-24:L386.
- `behavior_test_anchor_count = 24`; source eba15ac42775:L227.
- `behavior_test_anchor_count = 16`; source 2026-07-10:L231.
- `behavior_test_anchor_count = 14`; source eba15ac42775:L225.
- `expected_legacy_full_hits = 15`; source 2026-09-01:L106.
- `expected_guard_file_count = 22`; source eba15ac42775:L227.
- `behavior_test_anchor_count = 6`; source 2026-08-27:L120.
- `Runtime 03 guard anchors 14/14`; source eba15ac42775:L225.
- M4 gate `classified-and-clear`; source 2026-07-22:L382.
- `interface_v2_anchor_count = 9`; source 2026-09-01:L106.
- `expected_test_file_count = 10`; source 2026-08-28:L112.
- `event_message_anchors = 12/12`; source 2026-07-10:L231.
- `runtime_v2_anchor_count = 10`; source 2026-09-01:L106.
- `native loader test files 4/4`; source 2026-07-22:L382.
- `guard_test_anchor_count = 26`; source 2026-08-24:L306.
- `expected_ui_entry_count = 22`; source 2026-09-01:L106.
- `expected_doc_file_count = 5`; source 2026-08-24:L386.
- `cargo_gate_anchor_count = 7`; source 2026-09-01:L106.
- public `pub use` sites 2/2; source 2026-08-24:L306.
- `native root re-export 0/0`; source 2026-07-22:L382.
- `change_tick_anchors = 6/6`; source 2026-07-10:L231.
- `archetype_anchors = 15/15`; source 2026-07-10:L231.
- `missing_doc_anchors = []`; source 2026-09-01:L106.
- `root_reexport_count = 0`; source 2026-07-22:L382.
- `guard_anchor_count = 19`; source 2026-09-01:L106.
- `core public modules 5/5`; source 2026-08-24:L306.
- `test_anchor_count = 28`; source eba15ac42775:L227.
- `observer_anchors = 8/8`; source 2026-07-10:L231.
- `guard/test files 11/11`; source eba15ac42775:L225.
- `storage_anchors = 9/9`; source 2026-07-10:L231.
- `fixed-loop stages 3/3`; source eba15ac42775:L225.
- `doc_anchor_count = 61`; source 2026-09-01:L106.
- `core root entries 6/6`; source 2026-08-24:L306.
- `doc_anchors = 13/13`; source 2026-07-10:L231.
- `doc_anchors = 10/10`; source eba15ac42775:L225.
- `source files 22/22`; source eba15ac42775:L225.
- `debt groups 0/0`; source 2026-07-22:L382.

### Historical command identities

These are exact original command identities, not current executable recipes. Actual execution needs explicit finite package/features/target/profile admission.

- `cargo check -p zircon_runtime --lib --locked`.

### Contract details

The following original contract labels remain in scope; they do not establish new validation:

- no `WorldDriver` second `advance_time_by(...)` references, `no dynamic-session raw-delta level tick references`, `NativePluginProjectActivationSelectionStatus`, `NativePluginProjectActivationSelectionResult`.
- `app NativePlugin current call-site files: 5`, `NativePluginProjectActivationCleanupReceipt`, `NativePluginProjectActivationRequest`, `NativePluginProjectActivationResult`.
- `mirror_docs_guard_present = true`, `hot reload failure injection`, `retired core root entries 0`, `direct_rayon_paths = 2`.
- `unknown ABI rejection`, `Runtime 06 plan`, `EngineTaskGraph`, `JobScheduler`.

The retained asset-pipeline audit snapshot records `risks = []`. This is its dated diagnostic inventory, not a fresh risk assessment, Cargo result or product acceptance.

### Clause provenance

These content identities preserve the source of this transfer. They are not fresh measurements, active-file dependencies or acceptance receipts.

- `index.md`, SHA-256 `eba15ac42775b6ad479b3ccd5d90869900041fdf944d9b46cee303e21f8b7107`, selected source lines 106-409.
