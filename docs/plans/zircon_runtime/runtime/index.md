---
related_code:
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/animation_assets.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/runtime_helpers.rs
  - zircon_runtime/src/core/framework/physics/query_interface.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/physics/runtime/src/plugin.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/plugin_importer_dx/d10_bridge_call.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/plugin_importer_dx/d11_test_runtime_fixture.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/scene/ecs/mod.rs
  - zircon_runtime/src/plugin/mod.rs
  - zircon_runtime/src/plugin/runtime_plugin/descriptor/builder/runtime_plugin_descriptor_builder.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/assets/texture/descriptor.rs
  - zircon_runtime/src/asset/assets/texture/texture_asset.rs
  - zircon_runtime/src/asset/importer/ingest/import_texture.rs
  - zircon_plugins/texture_importer/runtime/src/importers.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/p0_robustness.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/p0_robustness/priority_recommendation.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder/first_party_descriptors.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder/scaffold.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder/test_fixtures.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_privacy.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_privacy/constructor_retirement.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_privacy/private_fields.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_privacy/status_mirrors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_child_owners.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure_assertions.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/source_inventory.rs
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_plugins/ui_document_importer/runtime/src/lib.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_schema.rs
  - docs/zircon_runtime/asset/assets/font.md
  - docs/zircon_runtime/asset/assets/ui.md
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f12_dead_code.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/plugin_importer_dx.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/plugin_importer_dx/d1_capability_single_source.rs
  - zircon_plugins/plugin_sdk/src/manifest/importer_runtime.rs
  - zircon_plugins/plugin_sdk/src/manifest/feature_bundle_builder.rs
  - tools/plugin_structure_audits/capability.py
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs
  - zircon_app/Cargo.toml
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/entry_static_dependencies.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/entry_static_dependencies_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/legacy_standalone_references.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/legacy_standalone_references_markdown.py
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/host_requests.rs
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_markdown.py
  - zircon_runtime_interface/src/runtime_api/host/host_requests.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/cursor/request.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_markdown.py
  - zircon_runtime/src/tests/runtime_absorption/input_stack.rs
  - zircon_runtime/src/tests/runtime_absorption/input_stack/inventory.rs
  - zircon_runtime/src/tests/runtime_absorption/input_stack/inventory/module_sets.rs
  - zircon_runtime/src/tests/runtime_absorption/input_stack/inventory/public_surface.rs
  - zircon_runtime/src/tests/runtime_absorption/input_stack/inventory/guard_anchors.rs
  - zircon_runtime/src/tests/runtime_absorption/input_stack/inventory/behavior_anchors.rs
  - zircon_runtime/src/tests/runtime_absorption/input_stack/inventory/cursor_host_requests.rs
  - zircon_runtime/src/tests/runtime_absorption/input_stack/inventory/mirror_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/input_stack/inventory/split_layout.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_inventory_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_runtime_gaps.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_runtime_gaps_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_convention_gate.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_convention_gate_markdown.py
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/module_convention_gate.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic/texture_containers.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_asset_dynamic.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework/camera_controller.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework/render_fixtures.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_core_framework.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_core_framework_render_fixtures.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/render_contracts.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners/observer_callback_registry.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners/query_state_many_item_array.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners/component_storage_component_results.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners/split_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_core_scene.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_core_scene_ecs.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics/render_fixtures.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_graphics.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names/scene_dynamic.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_banned_names.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_markdown.py
  - dev/bevy/crates/bevy_app/src/main_schedule.rs
  - dev/bevy/crates/bevy_asset/src
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/UnrealEngine/Engine/Source/Runtime
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/naming_guard_maps.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/direct_assertion_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/typed_error_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review_guard_maps.rs
plan_sources:
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
  - docs/plans/zircon_runtime/render/index.md
---
# Zircon Runtime 架构完善与优化总体计划

Current-source UI architecture mirror 2026-08-14: `ui_architecture_boundary` reports `expected_source_file_count = 52`, `expected_ui_entry_count = 20`, `expected_surface_entry_count = 26`, `legacy_full_hits = 70`, `expected_legacy_full_hits = 70`, `legacy_production_hits = 0`, `expected_legacy_production_hits = 0`, `legacy_production_file_count = 0`, `expected_legacy_production_file_count = 0`, `taffy_production_hits = 175`, `expected_taffy_production_hits = 175`, `taffy_production_file_count = 10`, `expected_taffy_production_file_count = 10`, `runtime_v2_anchor_count = 10`, `interface_v2_anchor_count = 9`, `guard_anchor_count = 19`, `cargo_gate_anchor_count = 7`, `doc_anchor_count = 61`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This current snapshot supersedes older dated counts without rewriting their history.

Runtime 08 current hard-cut sync (2026-08-28): `ecs_kernel_data_boundary` now owns `expected_source_file_count = 76`; the inventory includes `component/registry/transferred.rs` as the transferred-descriptor transaction owner, retains the six-file archetype-table owner plus `typed_api/{component_row,projection_rebuild}.rs`, and keeps `expected_test_file_count = 10`. This supersedes earlier current-count paragraphs while preserving their dated historical evidence.

Runtime13 scene-transition contract sync (2026-08-15, source-owner inventory updated 2026-08-29): `script_binding_boundary` reports `expected_source_file_count = 28`, `expected_test_file_count = 3`, `expected_guard_file_count = 8`, `fixed_host_function_count = 61`, `builtin_callback_count = 20`, `gameplay_callback_count = 40`, `host_capability_count = 13`, `missing_source_files = []`, `missing_guard_files = []`, and `risks = []`. `script.rs` remains the public facade for `argument_views`, `call_frame`, `descriptors`, `hot_path_metrics`, and `value_contracts`; `argument_views.rs` routes to four focused folder-backed owners without changing the public marshalling contract. `request_scene_transition` remains capability-gated and emits a `ReplaceActive` project request for deferred project ownership. The managed script Cargo gates remain pending.

2026-07-28 Runtime11 JobSystem 历史镜像：`tasks/timer.rs` 成为共享、有界的进程级 deadline owner，`job_system_boundary` 当时的 `expected_module_count = 10`；它替代 asset worker 的私有维护线程而不扩展公共任务 API。

2026-08-26 Runtime02 M1 implementation registers `tasks/callback_dispatcher.rs` and `tasks/execution/` as canonical JobSystem owners (`expected_module_count = 12`). `ExecutionRuntime` and `ExecutionScope` provide a fallible, non-static execution foundation with scope admission closure, requested-versus-acknowledged cancellation, typed task terminal status, and drain census. `JobHandle` and `schedule_after(...)` now propagate prerequisite cancellation without launching dependent work, and dynamic-scene loading acknowledges observed scope cancellation. `ExecutionWorkerInventory` reports only the execution runtime's three domains and conserved worker total, leaving process-default pools, the timer, and private workers visibly outside the owner. Scope quiescence is not a worker-join or DLL-unload receipt; scene target staging, worker-owner hard cutover, the generic scheduler, process timer, and private workers remain open. This is implementation pending, not an accepted runtime status.

2026-08-27 Runtime11 adds bounded owners for terminal task observations in `tasks/diagnostic_observation/`, blocking stdout/stderr capture in `tasks/bounded_stream_io/`, and clone-safe retained-result reservations in `tasks/retained_byte_budget.rs`, so the focused JobSystem mirror is `expected_module_count = 15` and `behavior_test_anchor_count = 67`. The stream owner admits a multi-pipe capture atomically, caps readers by physical Runtime `Io` parallelism, bounds chunk/line/queue/drain memory and work, retries interrupted reads, and keeps blocked readers visible to `ExecutionScope` shutdown accounting without private threads. The retained-result owner bounds bytes and live lease count until the final result clone drops. The diagnostic journal remains independently enabled and bounded. `job_scheduler.rs = 347` and `diagnostics.rs = 427` preserve `oversized_modules = []`, and the audit rejects runtime-to-editor dependencies. Source contracts are implemented; managed Cargo, dynamic-scene sync-facade migration, Editor14 migration, product performance/power evidence, worker join and independent acceptance remain pending.

2026-08-27 Runtime11 current-source correction: the explicit `ExecutionRuntime` three-domain owner now retains the only strong Rayon backends, while cloned `TaskPool`/scheduler routes are weak and share an atomic admission gate. Custom-spawned worker `JoinHandle`s are retained and shutdown now closes scopes, releases all three backends, waits for exits, joins exact handles, and publishes per-domain expected/exited/joined census before `Stopped`; timeout stays retryable `Closing`. This source slice still awaits managed Cargo and product profiling. Remaining process-default consumers, timer/private workers, and scene staging remain open, so it is not an aggregate DLL-unload or Runtime11 acceptance receipt.

The same 2026-08-27 source cut makes `AssetModule` the first production process-default migration: its manager factory injects the activating `CoreRuntime` `Io` pool into `ProjectAssetManager`, and a module activation regression checks owner identity. Standalone default consumers and the asset expiry timer remain open; Runtime04/Runtime11 acceptance is unchanged.

The second 2026-08-27 production migration and its follow-up hard cut move Platform preference persistence onto the activating Core TaskGraph owner. `PlatformModule` declares its Tasks dependency, and both the builtin driver factory and the App factory that installs a host-provided backend inject that owner. `PlatformDriver::default()` is deleted, its backend constructor requires a `TaskPool`, and the crate-private adapter constructor is hard-cut from implicit `new(...)` to explicit `with_pool(...)`. Retained one-worker fixtures replace 42 driver/manager defaults and four adapter implicit constructions. Static counts across the Platform subsystem are implicit owner-selection routes `3 -> 0`, direct process-default sites `2 -> 0`, driver default test calls `42 -> 0`, and adapter implicit test calls `4 -> 0`; the JobSystem structure audit passes 3/3. The production scene module uses `DefaultLevelManager::with_core`; its default manager is now memory-only and rejects artifact I/O without an injected Runtime owner instead of acquiring the process pool. Native-plugin discovery remains a process-lifetime authority because it precedes Core composition, matching Unreal's application-global plugin manager; its prepared-root nonblocking ticket/last-good snapshot source contract is implemented, while Editor12 migration and managed validation remain open. The process timer remains measurement-gated. These are static ownership results, not elapsed-time, RSS, wakeup, or power evidence, and Runtime11 acceptance is unchanged.

The third 2026-08-27 production migration removes the editor settings lane's `JobScheduler::process_io()` lookup. `EditorManager` now derives a scheduler from the active Core `Io` pool, and the Builder/service constructors require that route explicitly. The follow-up hard cut deletes both `JobScheduler::default()` and the now-unused `process_io()` constructor; Runtime and Editor tests explicitly inject their shared process fixture pool through `from_pool(...)` instead of allocating a full-CPU private pool per scheduler. The JobSystem audit rejects both constructor snippets. Five editor settings source-contract suites pass 24/24; managed Cargo, full-harness thread counts, and shutdown profiling remain pending, so Runtime11 acceptance is unchanged.

The same 2026-08-27 source pass closes the remaining direct-Rayon consumer without changing its algorithm: MeshDraw batches are still source-index sorted, prepared against cache state on the owner thread, built from immutable owned plans, then merged and stored in order. The new neutral `parallel_map_ordered(Vec<T>, ...)` moves those plans into the active `TaskPool`; empty/single inputs stay on fast paths, and only `tasks/pool.rs` plus `tasks/parallel_for.rs` retain Rayon. The focused JobSystem audit passes 3/3. Managed Cargo and the planned Windows CPU/allocator/context-switch/power matrix remain pending, so no performance or aggregate Runtime11 acceptance claim is made.

The offline Font SDF owner hard cut on 2026-08-27 removes the build library's last implicit `TaskPools::process_default()` route. `bake_font_sdf_artifact(...)` requires the caller's pool; the CLI owns one `EngineTaskGraph`, shuts it down with a bounded receipt before artifact publication, and integration tests retain a fixed two-worker graph. Static routes are implicit owner `1 -> 0` and worker sets `3 -> 1`; at 16 logical processors the configured worker total remains 16 while generation-visible parallelism changes `4 -> 16`. The JobSystem static audit is now 4/4. Product renderer text, small-batch overhead, elapsed time, RSS, wakeups and power remain unvalidated, so Runtime11 is still `in_progress`.

Runtime 13 current child-owner sync (2026-07-10): `script_binding_boundary` reports `expected_source_file_count = 19`, `expected_test_file_count = 3`, `expected_guard_file_count = 9`, `missing_guard_files = []`, `fixed_host_module_count = 6`, `fixed_host_function_count = 52`, `type_descriptor_count = 2`, `builtin_callback_count = 11`, `gameplay_callback_count = 39`, `macro_host_function_count = 2`, `host_capability_count = 11`, `guard_anchor_count = 9`, `native_ecs_abi_references = []`, `oversized_test_files = []`, `mirror_docs_guard_present = true`, and `risks = []`. The nine guard owners include the two route parents plus ledger/capability/ECS-facade, gameplay-host/mirror, despawn behavior, and Runtime 13 Cargo children. `runtime_13_script_binding_mirror_docs_match_structure_audit_counts` keeps the plan, runtime index, function ledger, M0 review, and interface-convergence mirror aligned; script package gates remain pending.

Runtime 12 current child-owner sync (2026-07-10): input, action mapping, recording/replay, cursor host requests, and gamepad ABI each have explicit runtime/framework/test owners. The input module document, Runtime 12 plan, M0 review, and interface-convergence mirror remain aligned; package input/action/gamepad/app validation stays pending and production input behavior is unchanged. Detailed anchors and command evidence live in the Runtime 12 numbered archive.

2026-07-19 Runtime 11 历史 guard-owner 快照：`job_system_boundary` 当时报告 `expected_module_count = 9`、`expected_guard_file_count = 2`、`missing_guard_files = []`、`direct_rayon_paths = 2`、`schedule_parallel_executor_direct_rayon = []`、`diagnostic_anchor_count = 11`、`behavior_test_anchor_count = 27`、`missing_behavior_test_anchors = []`、`oversized_modules = []`、`mirror_docs_guard_present = true` 与 `risks = []`。2 个 guard owner 为 route parent `job_system.rs` 与真实 folder-backed owner `job_system/mirror_docs.rs`；`runtime_11_job_system_mirror_docs_match_structure_audit_counts` 保持计划、runtime index、JobSystem 模块文档、M0 review 与 interface convergence 一致。该历史快照不覆盖当前 execution scope、12-owner 或 31-anchor 状态。

2026-07-13 Runtime 05 closeout 已完成：`runtime_05_scene_1642_structure_1304_review_298_pmrem_parity_passed_closeout_acceptance_complete` 在同一 fresh Windows lib-test 程序通过 full `scene::` 1642/1642（5 ignored）、`structure_convention` 1304/1304、`code_review_findings` 298/298 与 PMREM parity 1/1；父计划和本索引只保存当前概述/路由，具体五列产出、命令与历史锚点由 Runtime 05 编号归档拥有。

2026-08-27 Runtime11 current single-owner cut supersedes the retained three-domain
snapshots above: `tasks/task_graph/EngineTaskGraph` now creates exactly one physical
worker set under `EngineTaskGraphOptions`, `TaskGraphWorkerInventory` reports one
set and its exact worker count, and Core no longer exposes work-kind pool selectors.
The default scheduler and migrated runtime/app/editor consumers share that owner.
Dynamic session shutdown drains its scope and modules before closing the TaskGraph.
The current mirror is `expected_module_count = 22` and
`behavior_test_anchor_count = 73`, including the canonical Runtime task
contracts and handle/fence lifecycle regressions, with `tasks/bounded_stream_io/` and
`tasks/retained_byte_budget.rs` unchanged. Managed Cargo, affinity/priority/quota,
keyed-I/O parallelism, process/private-owner convergence, and performance/power
evidence remain pending; Runtime11 stays `in_progress`.

2026-08-27 Runtime11 graphics execution-owner hard cut: all six production-visible
`WgpuRenderFramework` constructors now require an explicit `TaskPool`; the three private
full-compute allocation sites in `construct.rs` are zero and the unused implicit Solari constructor
is deleted. Runtime module-host graphics, the PBR viewer, and Runtime/Editor/plugin fixtures inject
the worker pool from a retained `CoreRuntime::task_graph()`. The structure audit rejects private
graphics pool/TaskGraph construction and passes 3/3; focused formatting, diff, and tracked plus
untracked legacy-call checks pass. This is source ownership evidence only. Navigation still has one
production private pool path at this point in the sequence; unrelated inline test fixtures and other
private-owner families also remain. Managed Cargo plus WPR/RSS/power evidence are pending, so
Runtime11 stays `in_progress`.

2026-08-27 Runtime11 Navigation execution-owner hard cut: `DefaultNavigationManager` now requires
an injected `TaskPool`, and `navigation.runtime` declares `TasksModule` before deriving the pool from
its activation Core. The production private allocation count is `1 -> 0`; 55 unit construction sites
use an explicit retained two-worker TaskGraph fixture. Under the recorded 16-logical-processor source
model, Navigation activation no longer adds a second 16-worker set, so the modeled Rayon bound changes
from `32 -> 16`. Unreal Recast worker-census and `GThreadPool` dispatch sources were reviewed before
the change; the bake algorithm was not modified. The structure audit passes 3/3. Managed Cargo and
WPR/RSS/power evidence remain pending, so Runtime11 stays `in_progress`.

## 产出记录迁移说明

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

Runtime 总索引顶部历史状态镜像已迁入编号子计划产出目录；此处只保留架构现状、子计划路由与全局约束。

- Runtime 05 迁入记录：[`05/2026-07-09-runtime-index-output-records.md`](05/2026-07-09-runtime-index-output-records.md)
- Runtime 07 迁入记录：[`07/2026-07-09-runtime-index-output-records.md`](07/2026-07-09-runtime-index-output-records.md)
- Runtime 15 迁入记录：[`../../_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md`](../../_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md)
## 1. 技术选型评审结论（2026-06-12 实仓核对）

总体结论:选型与 Bevy 同代生态一致（winit + wgpu/naga + glam + crossbeam + notify + taffy），自研 ECS/UI/资产管线的目录形状与 `bevy_ecs`/`bevy_asset` 可逐项对照，方向合理。但"声称技术栈"与实仓有 5 处失实，另有 4 个能力缺口需要决策。

### 1.1 声称栈核对表

| 声称                               | 实仓状态                                                                                                                       | 证据                                                              | 评估                                                                     |
| ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------- | ------------------------------------------------------------------------ |
| winit 窗口输入                     | ✓`0.31.0-beta.2`，optional + feature 门控                                                                                   | 根 `Cargo.toml`、`zircon_runtime/Cargo.toml`                  | 合理；**beta 版本风险**，见子计划 01                               |
| wgpu / naga 渲染                   | ✓ 29.0.1 / 29.0.1 配套一致                                                                                                    | 根 `Cargo.toml`                                                 | 合理                                                                     |
| taffy UI 布局                      | ✓ 0.10，自研 UI 经 `ui/layout/taffy_bridge/{mod,compute}.rs` 桥接                                                           | `zircon_runtime/Cargo.toml`                                     | 合理                                                                     |
| glam 数学                          | ✓ 0.32.1 (serde)                                                                                                              | 根 `Cargo.toml`                                                 | 合理                                                                     |
| fontdue / cosmic-text / glyphon    | **部分失实**:fontdue 0.9.3 仅在 editor;cosmic-text 不存在;runtime 实际为 glyphon 0.11 + fontsdf 0.5.3 + 自研 text shaper | `zircon_editor/Cargo.toml`、`zircon_runtime/Cargo.toml:77-78` | 三库职责口径需定稿，见子计划 01                                          |
| 自研 ECS                           | ✓`scene/ecs/`（archetype、query、schedule、parallel executor、conflict graph、observer、change detection）                  | `zircon_runtime/src/scene/ecs/`                                 | 形状对齐 `bevy_ecs`，合理                                              |
| kira 音频                          | **失实**:不存在;实际为 cpal 0.15（optional）+ sound 插件自研 DSP/HRTF/occlusion/mixer                                    | `zircon_plugins/sound/runtime/Cargo.toml`                       | 既有自研混音栈下 cpal 底座更合理，**不建议**再引 kira;矫正文档即可 |
| image / serde / crossbeam 异步加载 | ✓ 全部存在;worker pool + watcher 在 `asset/pipeline`、`asset/watch`                                                       | `zircon_runtime/src/asset/`                                     | 合理                                                                     |
| zip / tar 打包                     | **失实**:均不存在;仅 zstd 0.13.3;`ExportPackagingStrategy` 已有契约但无归档实现                                        | `zircon_runtime/Cargo.toml:100`、`plugin/export_profile.rs`   | **导出打包能力缺口**，见子计划 01                                  |
| gilrs 手柄                         | ✓ 0.11.0，optional，`gamepad-gilrs` feature                                                                                 | `zircon_app/Cargo.toml:84`                                      | 合理                                                                     |
| tracing / tracing-subscriber       | ✓ tracing 常驻;subscriber 0.3.20 仅在 `profiling-tracy` 后                                                                  | `zircon_runtime/Cargo.toml:24,93`                               | 合理;profiling 构建超时问题见子计划 07                                   |
| rfd / arboard 编辑器辅助           | **失实**:均不存在                                                                                                        | 全仓 Cargo.toml grep 无命中                                       | editor 侧文件对话框/剪贴板缺口，归 `zircon_editor` 决策                |

### 1.2 声称栈未列、但实际承重的依赖

rayon（ECS/资产并行）、tokio + hyper + reqwest + tokio-tungstenite（net 插件网络栈）、bincode/ron/toml/serde_json（多格式序列化）、gltf/tobj/dxf/ply/stl（模型导入）、notify 9.0.0-rc.3（资产监视）、libloading（cdylib runtime 与 native 插件）、zstd、accesskit（optional）、Recast C++ 经 cc 绑定（navigation 插件）、tauri 2.11（zircon_hub，**Slint 已不在任何 Cargo.toml**）、zr_vm_rust_binding（**指向仓库外 `../../zr_vm` 的路径依赖**）。

### 1.3 版本与依赖治理风险

1. `winit 0.31.0-beta.2`:beta 跟踪策略未定稿（锁定/升级 gate）。
2. `notify 9.0.0-rc.3`:RC 版本。
3. `zr_vm_rust_binding` 路径依赖逃逸出仓库根，影响 clone 即建的可复现性（optional 缓解，仍需文档化或 vendor 决策）。
4. `backend-jolt` + physics backend：Runtime 01 M3 的历史 unavailable slot 已由 Plugins 03 M1-T3 推进为 plugin-owned optional `joltc-sys` native backend；feature-on Ready/native step，feature-off typed Unavailable，均不静默降级 builtin；当前守卫为 `physics_backend_option_decision_keeps_jolt_feature_gated_and_plugin_owned`，Rapier 仍不进主路径。

## 2. 架构评审结论

### 2.1 已收敛项（旧计划假设需修正处）

> 请将产出记录放置在子计划中，子计划中记录超过10条则全部放到子目录，此处仅展示当前现状的概述

旧计划假设中的 root surface、插件扇出、服务注册、序列化守卫与渲染分层等已收敛为当前架构基线；切片级状态锚、守卫计数与验证补记已迁入 Runtime 15 产出目录。

Runtime 03 当前静态清册：`schedule_frame_loop_boundary` 报告 source files 22/22、guard/test files 11/11、`SystemStage` count and variants 9/9、fixed-loop stages 3/3、dynamic-session `.tick_time(...)` calls 1/1、Runtime 03 guard anchors 14/14、`behavior_test_anchor_count = 14`、`missing_behavior_test_anchors = []`、`doc_anchors = 10/10`、frame schedule module-doc anchors 3/3、`mirror_docs_guard_present = true`、no `WorldDriver` second `advance_time_by(...)` references、no dynamic-session raw-delta level tick references 与 `risks = []`；`runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts` 保持这些数字跨计划/模块文档同步。历史 Runtime 过滤门结果不覆盖本次 WorldTimeController 改动；当前切片仍等待受管 Cargo 验证。

Runtime 04 当前静态镜像：`asset_pipeline_boundary` 报告 `expected_source_file_count = 25`、`expected_guard_file_count = 22`、`worker_diagnostic_count = 7`、`expected_worker_diagnostic_count = 7`、`artifact_store_roundtrip_count = 4`、`expected_artifact_store_roundtrip_count = 4`、`watcher_acceptance_reference_count = 1`、`expected_watcher_acceptance_count = 7`、`artifact_acceptance_reference_count = 3`、`test_anchor_count = 28`、`behavior_test_anchor_count = 24`、`missing_behavior_test_anchors = []`、`missing_doc_anchors = []`、`missing_cargo_gate_anchors = []`、`retired_worker_new_references = []`、`retired_worker_request_sender_references = []`、`old_watch_debounce_references = []`、`mirror_docs_guard_present = true` 与 `risks = []`；`runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts` 保持这些数字跨计划、模块文档与架构评审同步。22 个 guard/test owner 包含当前 folder-backed facade-query、artifact-scene、worker-policy、mirror-doc 与 Cargo-gate children；broader `asset::` / `worker_pool` Cargo filters 仍 pending，Runtime 04 保持 `in_progress`。

Runtime 09 当前静态镜像由 `ui_architecture_boundary` 持有。2026-07-10 文本 owner 新增的 `ui/surface/{text_geometry,text_shape}.rs` 是 shaped caret/range geometry 与共享 shaping 的直接 surface leaves；当前 surface entry map 为 23 项。该同步只更新结构事实和 runtime index 路由，不关闭 `ui/input/naming_boundary/layout/template` Cargo gate。

2026-07-10 Runtime 08 当前 child-owner 同步：`ecs_kernel_data_boundary` 报告 `expected_source_file_count = 69`、`expected_test_file_count = 10`、`archetype_anchors = 15/15`、`storage_anchors = 9/9`、`component_storage_private_reexport_anchors = 9/9`、`component_identity_anchors = 18/18`、`entity_lifecycle_anchors = 10/10`、`observer_anchors = 8/8`、`deferred_command_anchors = 11/11`、`event_message_anchors = 12/12`、`resource_identity_anchors = 12/12`、`change_tick_anchors = 6/6`、`runtime_08_guard_anchors = 21/21`、`behavior_test_anchor_count = 16`、`missing_behavior_test_anchors = []`、`doc_anchors = 13/13`、`pending_cargo_gate_anchors = 6/6`、`mirror_docs_guard_present = true` 与 `risks = []`；`runtime_08_ecs_kernel_data_mirror_docs_match_structure_audit_counts` 保持计划、runtime index、ECS 模块文档、M0 review 与 interface-convergence 镜像一致。10 个 test owner 显式包含 `ecs_kernel_data/inventory.rs` 与 `cargo_gates/early/runtime_08.rs`；该口径取代历史 8-route-owner 镜像，但不关闭 pending `entity/observer/command/messages/change_tick/ecs` Cargo gates。

- 迁入记录：[`../../_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md`](../../_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md)

### 2.2 问题清单（本计划的工作对象）

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

| 问题 | 当前边界 | 当前处理 | Owner |
|------|----------|----------|-------|
| P1 | runtime root graphics alias 已硬切换 | 保持旧 root re-export 为零并等待 Cargo gate | 02 |
| P2 | `core/{runtime,framework,manager,math,resource}` spine 已落地 | 继续收窄 root surface 与 generated leaf 边界 | 02 |
| P3 | schedule/time/frame-loop 单一权威已收口 | 当前 Runtime 四组过滤门与 `zircon_app` 全包门槛已闭合 | 03 |
| P4 | plugin native surface 与 lifecycle 已具备 owner | 等待 plugin validation lane | 06 |
| P5 | 性能热路径、权威 FPS、trace 与 ECS/extract 诊断已完成 | 共享工作区全包编译由活动 owner 后续总体验证 | 07 |
| P6 | scene/editor authoring 边界已静态收口 | full `scene::` 1642/1642 已闭环 | 05 |
| P7 | asset pipeline owner 与 artifact 路径已对齐 | 等待 asset/worker Cargo gate | 04 |
| P8 | generated 仅保留 leaf binding/DTO/table | 等待 workspace 级生成物验证 | 02 |
| P9 | 非网络 `server` 与旧命名持续清零 | 命名守卫与 scene Cargo gate 已闭环 | 05 |
| P10 | 物理、导出与编辑器依赖已归属插件/editor | 等待依赖治理 Cargo gate | 01 |
| P11 | ECS kernel/data owner 与行为锚已对齐 | 等待 ECS Cargo gate | 08 |
| P12 | UI 子系统 owner 已分层 | 等待 UI active lane 与 Cargo gate | 09 |
| P13 | dynamic ABI 与 UI handoff 合同已定义 | 等待 Runtime 09 owner handoff 和 Cargo gate | 10 |
| P14 | JobSystem task model 已归一 | 等待 JobSystem Cargo filters | 11 |
| P15 | input/action mapping 已对齐 UI handled 语义 | 等待 input Cargo gate | 12 |
| P16 | script binding/reflection 已避免 native ECS 旁路 | 等待 script Cargo filters | 13 |
| P17 | animation/navigation/log 等模块族已有 owner | 等待 module-family Cargo filters | 14 |

- 迁入记录：[`../../_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md`](../../_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md)

### 2.3 参考引擎证据锚点（2026-06-13 扩充：全路径实测存在；实现型切片动工前必读对应行，见 §7.9）

| 维度                 | Bevy                                                                                                                                  | Fyrox                                                                                       | Unreal                                                                                                                                                    | Godot / 其他                                                                                                                                              |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 帧循环权威           | `dev/bevy/crates/bevy_app/src/main_schedule.rs`（MainScheduleOrder）                                                                | `dev/Fyrox/fyrox-impl/src/engine/executor.rs`（fixed-step 累积）                          | Tick group（PrePhysics→PostUpdateWork）                                                                                                                  | `dev/godot/core/os/main_loop.{h,cpp}`、`dev/godot/main/main.cpp`                                                                                      |
| 固定步与时钟         | `dev/bevy/crates/bevy_time/src/{time.rs,fixed.rs,virt.rs,real.rs,timer.rs}`                                                         | 同上 executor.rs 的 lag 循环                                                                | —                                                                                                                                                        | `dev/godot/main/main_timer_sync.{h,cpp}`（固定步+插值同步的工程实现）                                                                                   |
| ECS 存储             | `dev/bevy/crates/bevy_ecs/src/storage/{table/,sparse_set.rs,blob_array.rs}`                                                         | `dev/Fyrox`（图式 scene graph，对照非 ECS 取舍）                                          | —                                                                                                                                                        | —                                                                                                                                                        |
| 实体分配/观察者/事件 | `dev/bevy/crates/bevy_ecs/src/entity/mod.rs`、`observer/{mod.rs,runner.rs,centralized_storage.rs}`、`event/{mod.rs,trigger.rs}` | —                                                                                          | —                                                                                                                                                        | —                                                                                                                                                        |
| 调度执行器           | `dev/bevy/crates/bevy_ecs/src/schedule/{schedule.rs,executor/,graph/,auto_insert_apply_deferred.rs}`                                | —                                                                                          | —                                                                                                                                                        | —                                                                                                                                                        |
| 任务/JobSystem       | `dev/bevy/crates/bevy_tasks/src/{task_pool.rs,usages.rs,slice.rs,iter/}`                                                            | —                                                                                          | `dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/{Task.h,Pipe.h,TaskConcurrencyLimiter.h}`、`Async/{ParallelFor.h,Async.h,LocalWorkQueue.h}` | `dev/godot/core/object/worker_thread_pool.{h,cpp}`（组任务/优先级/yield）；Unity 仅语义锚（源码不在 dev/）                                              |
| 渲染世界分离         | `dev/bevy/crates/bevy_render/src/extract_plugin.rs`                                                                                 | `fyrox-impl/src/renderer/`                                                                | RHI/RenderCore/Renderer 三层                                                                                                                              | —                                                                                                                                                        |
| 资产                 | `dev/bevy/crates/bevy_asset/src/{loader.rs,handle.rs,server/,processor/,meta.rs}`                                                   | `dev/Fyrox/fyrox-resource/src/{manager.rs,loader.rs,state.rs,event.rs}`                   | AssetRegistry                                                                                                                                             | `dev/godot/core/io/{resource_loader,resource_importer,resource_uid}.{h,cpp}`（线程化加载/去重/稳定 ID）；`dev/Piccolo/engine/source/runtime/resource` |
| UI                   | `dev/bevy/crates/bevy_ui/src/layout/{mod.rs,ui_surface.rs}`（taffy 桥接）                                                           | `dev/Fyrox/fyrox-ui/src/{control.rs,canvas.rs,button.rs,...}`（retained 控件树+消息路由） | Slate/UMG（概念对照）                                                                                                                                     | `dev/godot/scene/gui/`（Control/Container 布局族）                                                                                                      |
| 插件/热重载          | Plugin/PluginGroup（静态）                                                                                                            | `fyrox-impl/src/plugin/{mod.rs,dylib.rs}`（DynamicPlugin + 状态序列化重载）               | 模块系统                                                                                                                                                  | `dev/godot/core/extension/{gdextension.{h,cpp},gdextension_function_loader.{h,cpp}}`（C ABI 函数表装载/协商）                                           |
| 动态扩展 ABI         | —                                                                                                                                    | 同上 dylib.rs                                                                               | —                                                                                                                                                        | `dev/godot/core/extension/extension_api_dump.cpp`（API 面 dump/版本化）                                                                                 |
| 诊断/Profiling       | `dev/bevy/crates/bevy_diagnostic/src/{diagnostic.rs,frame_time_diagnostics_plugin.rs,frame_count.rs}`                               | —                                                                                          | —                                                                                                                                                        | `dev/godot/main/performance.{h,cpp}`（性能监视器单点）；`dev/tracy`（profiler 本体源码）                                                              |
| 物理                 | 核心无物理（生态 rapier/avian）                                                                                                       | `dev/Fyrox/fyrox-impl/Cargo.toml:30-31`（rapier 外挂）                                    | Chaos 内置                                                                                                                                                | `dev/godot/modules/{godot_physics_3d,jolt_physics}`（自研+Jolt 双后端）                                                                                 |
| 模块尺度             | crate-per-subsystem（约 40+ crate）                                                                                                   | crate-per-layer（core/resource/graph/ui/impl/editor）                                       | `Engine/Source/Runtime` 约 189 模块                                                                                                                     | `dev/Piccolo/engine/source/runtime/{core,function,platform,resource}`（小型引擎单 runtime 分层全参照）                                                  |

`zircon_runtime` 取"单 crate + 内部 spine"形状，介于 Fyrox（多 crate 分层）与 Unreal（巨型模块树）之间;在 cdylib 热重载约束下单 crate 是合理选择，但要求内部模块边界承担 Bevy 中 crate 边界的职责——这正是 P1/P2/P4 要修的内容。

## 3. 子计划地图与执行顺序

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

子计划的切片级状态、Cargo gate、守卫计数与历史阶段记录已迁入编号产出目录；此处只保留执行依赖与当前剩余 gate。

| 计划 | 文档 | 执行依赖 | 当前状态 / 剩余 gate |
|------|------|----------|----------------------|
| 01 | `01-tech-stack-and-dependency-governance.md` | 无（基础治理） | completed：五项 dependency/runtime/plugin Cargo gate 已闭合 |
| 02 | `02-core-spine-and-root-surface.md` | Runtime 01 | in_progress：core/root Cargo gate pending |
| 03 | `03-schedule-and-frame-loop-alignment.md` | Runtime 02 | completed：Runtime filters 77/77、4/4、165/0/10 ignored、15/15；`zircon_app` 135/0/1 ignored + PBR viewer 15/15 |
| 04 | `04-asset-pipeline-alignment.md` | Runtime 02、Runtime 03 | in_progress：asset/worker Cargo gate pending |
| 05 | `05-scene-editor-boundary-closeout.md` | Runtime 02、Runtime 04 | completed：scene 1642/1642、structure 1304/1304、review 298/298 |
| 06 | `06-plugin-surface-and-lifecycle.md` | Runtime 02 | in_progress：plugin validation active lane |
| 07 | `07-runtime-performance-hotpath.md` | Runtime 03、Runtime 08、Runtime 11 | completed：双次 Vampire FPS、trace、ECS/extract 计数与权威热点清单完成；共享工作区全包编译 blocker 已归活动 owner |
| 08 | `08-ecs-kernel-data-alignment.md` | Runtime 02 | in_progress：ECS Cargo gate pending |
| 09 | `09-ui-subsystem-architecture.md` | Runtime 02、Runtime 03 | in_progress：UI owner active lane 与 Cargo gate pending |
| 10 | `10-dynamic-api-and-interface-convergence.md` | Runtime 02、Runtime 05、Runtime 09 | in_progress：Runtime 09 owner handoff 与 Cargo gate pending |
| 11 | `11-job-system-task-model.md` | Runtime 02 | in_progress：explicit Runtime worker join source implemented；managed Cargo、profiling 与 remaining owners pending |
| 12 | `12-input-stack-and-action-mapping.md` | Runtime 03、Runtime 09 | in_progress：input Cargo gate pending |
| 13 | `13-script-binding-and-reflection.md` | Runtime 02、Runtime 06、Runtime 08 | in_progress：script Cargo filters pending |
| 14 | `14-runtime-module-family-closeout.md` | Runtime 02、Runtime 03 | in_progress：module-family Cargo filters pending |
| 15 | `15-code-structure-and-module-conventions.md` | Runtime 01 至 Runtime 14 | in_progress：shader-prewarm inventory 224/431/159 owner split static/metadata green；active lane 清单与 Cargo gate pending |

2026-08-24 Runtime 02 root public-surface 静态同步：`text` 与 `operation` 按外部消费的 runtime-module-entry 归类，`runtime_diagnostics` 保持 feature-independent 私有根支持模块，不能被 optional `dynamic-api` 吸收。`core_spine_root_generated_boundary` 当前报告 core root entries 6/6、core public modules 5/5、retired core root entries 0、runtime root public modules 21/21、public `pub use` sites 2/2、crate-visible graphics alias debt 0/0、root-surface M1 与 generated-code M1 均为 `classified-and-clear`、generated export templates 10/10、generated behavior/allowed adapters/migration debt 6/6/0/0、guard tests 13/6/7、`guard_test_anchor_count = 26`、`missing_guard_test_anchors = []`、`mirror_docs_guard_present = true`、`risks = []`；仍仅为静态结构证据，Runtime 02 Cargo gate 保持 pending。

迁出明细：[`../../_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md`](../../_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md)

### 3.1 已知但暂不立项的缺口

| 缺口 | 当前证据 | Owner / 触发条件 |
|------|----------|------------------|
| 网络复制 runtime 侧 | 当前只有 net plugin transport，runtime replication contract 尚未立项 | net plugin owner；出现 authoritative replication 需求时触发 Runtime 新子计划 |
| 音频 runtime 服务面 | 当前由 sound plugin 与 cpal backend 承担 | sound plugin owner；跨场景 runtime audio service 需求稳定后立项 |
| FFI panic 安全动态验收 | 2026-08-29 current-source 统一 inventory 已覆盖 Runtime C ABI 42/42（Dynamic API 27、native loader 15）与 export/JNI 模板 13/13；受管 Cargo 和产品动态回归仍 pending | Runtime 10 + Runtime 15 owner；新增 ABI 入口必须先进入 owner 分类和 panic guard，再更新 inventory |
| 输入录制/回放 | runtime helper 已存在，尚无 editor UI 与资产格式 | Runtime 12 backlog；产品化需求稳定后立项 |
| 脚本调试器/断点面 | 当前无 debugger protocol | Runtime 13 backlog；ZrVM 调试协议稳定后立项 |
| 存档/会话持久化语义 | DynamicScene session archive core 已存在，玩法 schema 未定义 | Runtime 04/05 owner；玩法存档需求稳定后立项 |
| 本地化/i18n | 当前无 runtime localization service | Runtime 09 backlog；text/UI owner 稳定后立项 |

阶段划分:

- 当前阶段只维护上述 owner 与触发条件；具体实现一旦立项，必须新建对应子计划并写入编号产出目录。

## 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

| 范围 | 记录位置 |
|------|----------|
| Runtime 01-15 当前状态 | §3 子计划地图与各父计划 frontmatter |
| 具体切片、验证命令与证据 | 对应 `01/` 至 `15/` 编号产出目录 |

## 4. 全局边界约束（各子计划必须遵守）

继承 `Runtime 吸收层与 Editor_Scene 边界收束计划.md` 与 render 总计划 §5:

1. 不新增 crate;公共架构保持 `zircon_app`/`zircon_runtime`/`zircon_editor` 三件套 + `zircon_runtime_interface` ABI 层 + 内部 `core/{runtime,framework,manager,math,resource}` spine。
2. 硬切换:新 owner 路径落地的同一变更内迁移调用方并删除旧路径，不留 re-export、alias、shim。
3. 非网络语义的 `server` 命名是 blocker（`target-server` 为真实 headless 服务宿主语义，合法）。
4. 动态边界（dynamic_api、native 插件、VM 插件）只传 ABI-safe 值与序列化负载。
5. generated 产物只许 leaf binding/DTO/table，不许持有业务规则、调度或状态突变。
6. 渲染骨架（RDG、MeshDrawCommand、GPUScene、可见性、光照、时域、后处理、permutation）一律归 `docs/plans/zircon_runtime/render/01-08`，本目录子计划不得重复或冲突。
7. **UI 子系统(09)是编辑器 UI 链路的引擎实现层,不另立设计语言/约束语义**:`zircon_runtime/src/ui/**` 实现 `docs/plans/zircon_editor/editor_layout` 规范层定义的契约(13 约束/18 输入·命中/19 焦点导航/20 USS 级联样式/17 文本),契约 DTO 住 `zircon_runtime_interface::ui`(单源);`editor_ui/` 为运行时能力层,GPU 提交/上屏归 render(14 2D/UI stack,遵 `editor_layout/21` 提交契约)。语义以 `editor_layout/NN` 为准,缺口回流对应规范子计划。详见子计划 09「与 editor_layout / editor_ui / render 的关系」与 `editor_layout/index §6.1`。

## 5. 全局验收与测试基线

按 milestone-first 政策:实现切片期间只做轻量检查，每个里程碑末进入测试阶段。

- 切片期:`cargo check -p zircon_runtime --lib --locked`（必要时 `-p zircon_app` / `-p zircon_editor`）。
- 里程碑测试阶段:`cargo test -p zircon_runtime --lib --locked`（按子计划模块过滤词收窄）;涉及装配时加 `cargo test -p zircon_app --locked`。
- 插件接缝:`cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked`。
- 结构守卫:各子计划列出的源断言/结构测试（如 app 不得直依插件实现 crate、interface 不得出现 wgpu）。
- 文档:每个里程碑完成后按源码镜像路径更新 `docs/zircon_runtime/**`，并刷新本目录子计划状态标记。

## 6. 协调与活动会话避让

- `20260604-1232-runtime-architecture-review`（活跃）:正在执行旧渐进式计划的切片（root surface 债 4、大文件债 5、hotspots 38 的审计口径以该会话为准）。本目录 02/05 子计划的执行必须先对齐该会话的最新 touched_modules，避免双写。
- `20260611-0416-rendering-10fps-analysis`（活跃）:graphics 性能修复进行中，明示"不回退 worktree 改动、只做聚焦编辑"。07 子计划执行前必须复读该笔记。
- `20260603-2304-plugin-ecosystem-continuation`（活跃）:06 子计划执行前同上。

## 7. 工程化落地公约（2026-06-12 细化定稿，约束全部子计划执行）

各子计划已细化到切片级；执行任何切片时遵守以下公约，违反视为切片未完成：

1. **切片五要素**:每个切片的"目标文件 / 改动形态 / 调用方迁移 / 验收 / DoD"五项缺一即不开工;签名草案在动手前定稿，定稿差异回写计划文件。
2. **执行前检查清单是闸门**:各子计划"执行前检查清单"逐项过完才能动第一刀;行号与计数以重核结果为准，漂移时先回写"现状与证据"节再继续。
3. **状态节实时性**:每完成一个切片，立刻更新该计划"状态与产出记录"表（状态/日期/证据三列），禁止批量补记;基线数值在开工首日填写。
4. **milestone-first 测试节奏**:切片期只跑该计划列出的 `cargo check`;测试统一压到里程碑末的"测试阶段"命令清单，逐条可复制执行并留存输出摘要。
5. **硬切换提交粒度**:`git mv` / 删导出 / 改签名类切片，旧路径删除与调用方迁移必须同一提交闭合，禁止中间态（双签名、临时 re-export）跨提交存在。
6. **会话避让**:触及 `20260604-1232`（架构审查）、`20260611-0416`（10fps，禁止回退其改动）、`20260603-2304`（插件生态）三个活跃会话工作区前，先重读其笔记并按计划"风险与协调"节对齐;每次执行开新会话笔记（cross-session-coordination 规范）。
7. **证据纪律**:计划中标注"执行时核验:<命令>"的条目，核验输出粘入状态节;新增守卫测试一律带负例自检（参照 `authoring_boundary_guard_fails_on_representative_tokens` 模式）。
8. **共享基建复用**:结构扫描守卫（02-M4 generated、05-1.4 命名、01-1.4 manifest）共享同一套源码遍历 helper，后落地者复用先落地者，禁止三套扫描实现。
9. **参考锚点纪律（防凭空实现）**:实现型切片（新增类型/算法/调度/状态机语义）动工前，必须先读该计划"参考锚点"列出的 dev/ 源码与本 index §2.3 速查表对应行，并在状态节证据列注明"已读锚点: <路径>"；锚点与设计有出入时，差异判词写入计划再动手。计划未列锚点的领域：先查 §2.3 速查表，速查表也无对应行的，在计划中显式标注"无参考，自研判词: <理由>"后方可实现——禁止默默凭记忆复刻其他引擎行为。

## Runtime 15 review-guard row-data current routing

具体 owner、状态锚与验证记录已迁入 [`../../_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md`](../../_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md)；本索引只保留当前子计划路由与全局约束。

Current Runtime13 owner hard-cut sync 2026-07-14: `script_binding_boundary` reports `expected_source_file_count = 18`, `expected_test_file_count = 3`, `expected_guard_file_count = 9`, `missing_source_files = []`, `missing_guard_files = []`, `fixed_host_module_count = 6`, `fixed_host_function_count = 52`, `type_descriptor_count = 2`, `builtin_callback_count = 11`, `gameplay_callback_count = 39`, `macro_host_function_count = 2`, `host_capability_count = 11`, `guard_anchor_count = 9`, `native_ecs_abi_references = []`, `oversized_test_files = []`, `mirror_docs_guard_present = true`, and `risks = []`. The concrete ZrVM host-module source is owned and guarded by `zircon_plugin_zr_vm_language_runtime`, not by Runtime13.
2026-07-22 Runtime 06 native callback diagnostics public-surface mirror: the current tree retains the sole `plugin::native` public seat with `root_reexport_count = 0`, `native_namespace_reexport_count = 68`, native root re-export 0/0, native namespace re-export 68/68, M4 gate `classified-and-clear`, debt groups 0/0, native namespace symbol groups 5/5, unclassified native root symbols 0/0, unclassified native namespace symbols 0/0, root public native re-export locations 0/0, public native namespace re-export locations 1/1, app NativePlugin current call-site files: 7, native loader V1/V2 implementation files 0/0, `zircon_plugins` V1/V2 usage files 0/0, export_build_plan V1/V2 usage 0/0, unknown ABI rejection, hot reload failure injection, native loader test files 4/4, native test namespace import files 3/3, native test root import leaks 0/0, fallback lifecycle failure tests 4/4, `runtime_06_vm_lifecycle_fallback_failure_tests_are_folder_backed`, `runtime_06_native_loader_tests_use_isolated_plugin_native_namespace`, `mirror_docs_guard_present = true`, `risks = []`, and `runtime_06_plugin_surface_lifecycle_mirror_docs_match_structure_audit_counts`. This mirror does not close Runtime 06 validation gates.

2026-08-24 Runtime 06 M3.1 source hard cut: descriptor and entry remain V3, behavior remains V4, and byte transport is physical V3 only. The loader, SDK, native fixtures, editor-contribution fixture, and glTF importer retain no V2 descriptor/entry or transport symbol, V3-to-V2 alias, `abi_v2_only` fixture feature, or `NativeHostApiV3RegistrationScope`; V4 registration policy/scope is the sole public registration owner. The V3 descriptor `abi_unknown_version` fixture preserves explicit invalid-version rejection. This is static source evidence only: the existing public-surface classification/root-import drift and managed Cargo/native validation remain open.

2026-08-24 Runtime 06 native namespace static sync: `plugin::native::discovery` owns discovery/load commands and `plugin::native::host` owns the stable live-host handles; callers no longer use flat discovery or handle paths. `plugin_surface_lifecycle_boundary` reports `expected_source_file_count = 20`, `expected_doc_file_count = 5`, `root_reexport_count = 0`, `native_namespace_reexport_count = 68`, native root re-export 0/0, native namespace re-export 68/68, native namespace symbol groups 6/6, app NativePlugin current call-site files: 7, and `risks = []`. This is static source evidence only and does not close the managed Cargo/native validation lane.

Current Runtime 04 source-owner synchronization (2026-08-14): `asset_pipeline_boundary` reports `expected_source_file_count = 26`; `core/resource/manager/commit.rs` owns reload transition mutation state in the current tree. This replaces the prior public-facade-only inventory and does not close broader Cargo gates.
