---
related_code:
  - zircon_runtime/src/core/diagnostics/mod.rs
  - zircon_runtime/src/core/diagnostics/store.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/capability.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/history.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/graph.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/anti_alias.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/particle.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/virtual_geometry.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/hybrid_gi.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/advanced_provider.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/solari.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/core/diagnostics/snapshot.rs
  - zircon_runtime/src/core/diagnostics/render.rs
  - zircon_runtime/src/core/diagnostics/profiling/mod.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/tests/render_framework_graph_stats.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/diagnostics.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/core/runtime/state/runtime_inner.rs
  - zircon_runtime/src/diagnostic_log/diagnostics.rs
implementation_files:
  - zircon_runtime/src/core/diagnostics/store.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/capability.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/history.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/graph.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/anti_alias.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/particle.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/virtual_geometry.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/hybrid_gi.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/advanced_provider.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/solari.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/core/diagnostics/render.rs
  - zircon_runtime/src/core/diagnostics/profiling/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/diagnostics.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/core/runtime/state/runtime_inner.rs
  - zircon_runtime/src/diagnostic_log/diagnostics.rs
plan_sources:
  - user: 2026-05-22 continue M10 render diagnostics and profiling bridge checklist
  - user: 2026-06-02 PLEASE IMPLEMENT THIS PLAN - ZirconEngine WGPU 渲染主链闭环计划
  - user: 2026-05-16 continue Bevy-style runtime Time diagnostics integration
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
  - docs/zircon_runtime/graphics/render-product-submit.md
  - dev/bevy/crates/bevy_render/src/diagnostic/mod.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/internal.rs
  - dev/bevy/docs/profiling.md
  - .codex/plans/ZirconEngine Bevy 参照基础设施收束计划.md
  - dev/bevy/crates/bevy_diagnostic/src/frame_time_diagnostics_plugin.rs
  - dev/bevy/crates/bevy_diagnostic/src/log_diagnostics_plugin.rs
tests:
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs
  - zircon_runtime/src/tests/runtime_diagnostics/support.rs
  - zircon_runtime/src/tests/time.rs
  - zircon_runtime/src/tests/prelude.rs
  - zircon_runtime/src/graphics/tests/render_profiling.rs
  - zircon_runtime/src/graphics/tests/render_framework_graph_stats.rs::render_framework_stats_report_transient_allocation_bytes
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs::tests::execution_record_tracks_compute_dispatch_metadata
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs::headless_wgpu_server_falls_back_async_compute_passes_to_graphics
  - cargo test -p zircon_runtime --lib runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins --locked
  - cargo test -p zircon_runtime --lib execution_record_tracks_compute_dispatch_metadata --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib headless_wgpu_server_falls_back_async_compute_passes_to_graphics --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib time --locked
  - cargo check -p zircon_runtime --profile profiling --features profiling --locked
doc_type: module-detail
---

# Core Runtime Diagnostics

`zircon_runtime::core::diagnostics` provides the read-only diagnostic snapshot surface for runtime tooling. The store contracts already existed as plain data structures; the current Bevy-parity slice makes `CoreRuntime` own one `DiagnosticStore` so frame and system metrics can accumulate in the same runtime instance that owns lifecycle, time, task pools, and services.

## Reference Evidence

Bevy's `FrameTimeDiagnosticsPlugin` in `dev/bevy/crates/bevy_diagnostic/src/frame_time_diagnostics_plugin.rs` registers `frame_time`, `fps`, and `frame_count` diagnostics from `Time<Real>` plus `FrameCount`. Bevy's `LogDiagnosticsPlugin` in `dev/bevy/crates/bevy_diagnostic/src/log_diagnostics_plugin.rs` consumes the diagnostics store as a reporting layer rather than owning frame timing itself.

Zircon mirrors the ownership split: `CoreRuntime` records diagnostic measurements, while log/dev tooling can read snapshots later through `collect_runtime_diagnostics`.

## Ownership Boundary

- `DiagnosticStore` owns bounded series history, current values, smoothing, min/max, units, and subsystem tags.
- `CoreRuntimeInner` owns one `DiagnosticStore` per runtime instance.
- `CoreRuntime` and `CoreHandle` expose `record_diagnostic`, `diagnostic_store`, and `diagnostic_store_snapshot`.
- `CoreHandle::advance_time_by(...)` records Bevy-style time measurements after advancing runtime clocks.
- `collect_runtime_diagnostics` starts with the runtime-owned store and then overlays derived render, physics, animation, and profiling diagnostics.
- `diagnostic_log::format_diagnostic_store_snapshot(...)` and `write_diagnostic_store_snapshot(...)` turn store snapshots into process-log lines for dev-profile diagnostics.

The diagnostics store is not a global singleton. This keeps tests, runtime preview sessions, editor-host runtimes, and future export hosts isolated from each other.

## Render Diagnostics Bridge

M10.8 keeps render diagnostics on the same runtime-owned snapshot boundary. Bevy's `RenderDiagnosticsPlugin` records CPU/GPU pass elapsed time, pipeline statistics, and buffer-backed scalar diagnostics, then syncs finished rows into `DiagnosticsStore`. Zircon is not at that parity level yet: `RuntimeRenderDiagnostics` currently wraps a queried `RenderStats` snapshot, and `collect_runtime_diagnostics(...)` records submit/product counters into the store. The capability rows use `render.capability.*` bool/count paths for queue class count, surface/offscreen support, async queues, cache/storage/readback/indirect support, raytracing/resource-indexing capabilities, anti-alias feature support, max MSAA samples, VG/HGI backend gates, and the M8 `render.capability.neural_compute_supported` / `render.capability.sparse_texture_supported` slots. History rows use `render.history.*` paths for current/previous handle presence, previous-frame usability, aggregate invalidation, target/render dimensions (`target_width`, `target_height`, `render_width`, `render_height`), and one-hot invalidation reasons including `render_size_changed`. The render graph rows now include the legacy `render.last_graph_executed_pass_count` plus stable `render.graph.*` count paths for planned pass count, culled passes, queue fallbacks, resource lifetimes, sparse texture reservation lifetimes, planned resource accesses, planned dependencies, dense transient texture slots, sparse texture reservation slots, transient buffer slots, executed passes, executed resource accesses, executed dependencies, pass-level debug marker coverage, concrete compute dispatch count, aggregate compute dispatch group volume, compute storage-write resource count, and executed family counts for AA, VG, HGI, particles, transparent, and async-compute passes. The same graph bridge records `bytes` rows for dense transient texture/buffer reservations, total dense transient reservation pressure, and sparse virtual texture reservation footprint. Post-process graph rows use `render.post_process.graph.*` for node count, skipped node count, executed node count, and final composite presence; effect-stack rows remain under `render.post_process.effect_stack.*`; LUT renderer readiness rows now use `render.post_process.lut.request_count`, `ready_count`, and `fallback_count`. Advanced-slot rows now mirror AA fallback state under `render.anti_alias.*`, GPU particle counters under `render.particle.gpu.*`, VG counters/source/debug/readback rows under `render.virtual_geometry.*`, HGI probe/cache/scene/voxel rows under `render.hybrid_gi.*`, provider availability/report rows under `render.advanced_provider.*`, and Solari requested/status/degradation rows under `render.solari.*`.

That narrower bridge is still the correct consumer boundary. Runtime diagnostics panels, diagnostic log schedules, overlays, and editor tooling should consume `RuntimeDiagnosticsSnapshot` or `DiagnosticStoreSnapshot` instead of querying renderer-private state. The same bridge also mirrors material, light, mesh queue, sprite, UI, post-process effect-stack, and LUT texture fallback readiness rows. Promotion beyond the current bridge still requires adding stable diagnostic paths for pass-level CPU timing, backend-gated GPU timing, pipeline/cache status, present/capture failures, render-asset residency, and mesh allocator memory. Profiling artifacts can support this evidence, but they do not replace store-backed diagnostics.

RenderDoc markers are explicitly debugging evidence, not profiling evidence. Bevy's profiling docs route GPU performance investigation through Tracy RenderQueue or vendor profilers, while RenderDoc remains a capture/debug tool. Zircon records `render.graph.debug_marker_count` from `RenderStats.last_graph_executed_debug_markers` only to prove graph pass marker coverage; it is not a timing metric and should stay separate from future GPU timestamp or pipeline-statistics rows.

Compute dispatch diagnostics are likewise execution evidence, not timing or backend object exposure. `RenderGraphExecutionRecord` collects `RenderGraphComputeDispatchRecord` rows from the graph GPU context after SSAO or clustered-lighting executors launch their WGPU compute pass bodies. Runtime diagnostics only mirror three numeric aggregates: `render.graph.compute_dispatch_count`, `render.graph.compute_dispatch_group_count`, and `render.graph.compute_storage_write_resource_count`. This lets tools see whether graph-declared compute work actually ran, including graphics-queue fallback cases, while the framework surface remains neutral.

Sparse texture diagnostics are resource-reservation evidence, not proof of a sparse residency implementation. `CompiledRenderGraphStats.sparse_texture_lifetime_count` and `CompiledRenderGraphTransientAllocationPlan.sparse_texture_slot_count` flow through `RenderStats.last_graph_sparse_texture_lifetime_count` / `last_graph_sparse_texture_slot_count` and into `DiagnosticStore` as `render.graph.sparse_texture_lifetime_count` / `render.graph.sparse_texture_slot_count`. These rows show that graph validation preserved sparse virtual texture reservations and kept them out of dense transient aliasing; page tables, tile uploads, residency eviction, and WGPU sparse objects remain future renderer/provider work.

Transient allocation byte diagnostics are planning evidence, not allocator ownership evidence. `CompiledRenderGraphTransientAllocationPlan` derives byte totals from RHI-neutral buffer sizes and texture descriptors, then `update_base_stats(...)` copies them into `RenderStats.last_graph_transient_texture_bytes_reserved`, `last_graph_transient_buffer_bytes_reserved`, `last_graph_transient_dense_bytes_reserved`, and `last_graph_sparse_texture_virtual_bytes`. `DiagnosticStore` mirrors those as `render.graph.transient_texture_bytes_reserved`, `render.graph.transient_buffer_bytes_reserved`, `render.graph.transient_dense_bytes_reserved`, and `render.graph.sparse_texture_virtual_bytes` with unit `bytes`.

## Time Diagnostics

Each nonzero time advance records:

- `time.frame_time` in milliseconds,
- `time.fps` in hertz,
- `time.frame_count` in frames,
- `time.fixed_steps` in fixed-step count for that outer update.

`time.frame_count` and `time.fixed_steps` are still recorded on zero-delta updates. `time.frame_time` and `time.fps` are skipped for zero deltas, matching Bevy's guard against dividing by zero.

## Test Coverage

`zircon_runtime/src/tests/time.rs` verifies that advancing runtime time records the expected frame time, FPS, frame count, and fixed-step measurements, and that `collect_runtime_diagnostics` includes those runtime-owned values.

`zircon_runtime/src/diagnostic_log/diagnostics.rs` verifies stable formatting for current, smoothed, min, and max diagnostic values. `zircon_runtime/src/tests/prelude.rs` continues to verify the public diagnostic store, snapshot, and diagnostic-log formatting helpers through the stable runtime prelude.

2026-05-26 M10W evidence:

- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo test -p zircon_runtime --locked runtime_diagnostics --jobs 1 --message-format short --color never`: PASS, 2 matching lib tests passed.
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo test -p zircon_runtime --locked diagnostic_store --jobs 1 --message-format short --color never`: PASS, 5 matching lib tests passed.
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never`: PASS with 7 existing warnings.

2026-06-02 render-main-chain LUT diagnostics evidence:

- `cargo test -p zircon_runtime --lib runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never`: PASS, proving `render.post_process.lut.request_count`, `ready_count`, and `fallback_count` are projected from `RenderStats` into `DiagnosticStore`.

2026-06-03 render-main-chain compute dispatch diagnostics evidence:

- `cargo test -p zircon_runtime --lib execution_record_tracks_compute_dispatch_metadata --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never`: PASS, proving `RenderGraphExecutionRecord` preserves compute dispatch metadata and aggregates dispatch group volume plus storage-write resources.
- `cargo test -p zircon_runtime --lib headless_wgpu_server_falls_back_async_compute_passes_to_graphics --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never`: PASS, proving a compute-declared graph pass that falls back to the graphics queue still reports concrete clustered-lighting dispatch evidence.
- `cargo test -p zircon_runtime --lib runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never`: PASS, proving the compute dispatch, dispatch group, and storage-write resource count rows are projected from `RenderStats` into `DiagnosticStore`.

2026-06-03 render-main-chain history-size diagnostics evidence:

- `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir F:\cargo-targets\zircon-render-main-chain-history --message-format short --color never`: PASS with existing warnings, proving the `RenderStats` / `FrameHistoryStatus` / `render.history.*` row additions type-check through the runtime lib.
- `cargo test -p zircon_runtime --lib render_framework_invalidates_history_when_dynamic_render_size_changes --locked --jobs 1 --target-dir F:\cargo-targets\zircon-render-main-chain-history --message-format short --color never`: BLOCKED before test execution by unrelated `zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/*` lib-test compile errors (`E0364` private re-exports and `E0282` inference errors).

2026-06-04 render-main-chain transient allocation byte diagnostics evidence:

- `cargo test -p zircon_runtime --lib render_framework_stats_report_transient_allocation_bytes --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture`: focused validation target for `RenderStats` transient byte projection.
- `cargo test -p zircon_runtime --lib runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture`: focused validation target for byte-unit `DiagnosticStore` rows.
