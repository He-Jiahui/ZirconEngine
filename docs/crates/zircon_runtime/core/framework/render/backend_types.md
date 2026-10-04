---
related_code:
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/backend_types/handles.rs
  - zircon_runtime/src/core/framework/render/backend_types/history.rs
  - zircon_runtime/src/core/framework/render/backend_types/camera_target.rs
  - zircon_runtime/src/core/framework/render/backend_types/graph_reports.rs
  - zircon_runtime/src/core/framework/render/backend_types/backend_status.rs
  - zircon_runtime/src/core/framework/render/backend_types/capability.rs
  - zircon_runtime/src/core/framework/render/backend_types/command.rs
  - zircon_runtime/src/core/framework/render/backend_types/quality.rs
  - zircon_runtime/src/core/framework/render/backend_types/tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_backend_types.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/backend_types/handles.rs
  - zircon_runtime/src/core/framework/render/backend_types/history.rs
  - zircon_runtime/src/core/framework/render/backend_types/camera_target.rs
  - zircon_runtime/src/core/framework/render/backend_types/graph_reports.rs
  - zircon_runtime/src/core/framework/render/backend_types/backend_status.rs
  - zircon_runtime/src/core/framework/render/backend_types/capability.rs
  - zircon_runtime/src/core/framework/render/backend_types/command.rs
  - zircon_runtime/src/core/framework/render/backend_types/quality.rs
plan_sources:
  - user: 2026-06-24 continue render architecture implementation and prioritize structure/review findings
  - docs/plans/zircon_runtime/render/index.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/core/framework/render/backend_types/tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_backend_types.rs::runtime_15_render_backend_types_are_child_owners
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/render/backend_types.rs zircon_runtime/src/core/framework/render/backend_types/*.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_backend_types.rs
doc_type: module-detail
---

# Render Backend Types

## Purpose

`zircon_runtime::core::framework::render::backend_types` is the framework-facing DTO surface between neutral render contracts and the concrete graphics runtime. It carries stable handles, history reports, camera target reports, render graph diagnostics, backend capabilities, command/query messages, quality settings, payload source labels, and the aggregate `RenderStats` snapshot.

The 2026-06-24 Render backend-types owner split keeps this surface readable by moving independent DTO families into child owners. The parent file now acts as the public re-export surface and owns only `RenderGpuSceneUploadPath` plus the large `RenderStats` aggregate. It no longer stores every handle, report, capability, quality profile, command, and test in one mixed file.

Status anchor: `render_backend_types_owner_split_static_passed_cargo_deferred_active_compile_lane`.

## Owner Layout

`backend_types.rs` remains the module facade. It declares child modules, re-exports the public DTOs consumed by `render/mod.rs`, and keeps `RenderStats` in one place because graphics submit/update paths still project many subsystem reports into that aggregate.

The child owners are split by responsibility:

- `handles.rs`: `RenderViewportHandle`, `RenderPipelineHandle`, and `FrameHistoryHandle`.
- `history.rs`: frame history invalidation/status and history copy reports.
- `camera_target.rs`: camera target resolution, graph-import, and writeback status reports.
- `graph_reports.rs`: transient pool, materialization, alias/profile/coverage/stage execution, scene velocity readback, and motion-vector camera status reports.
- `backend_status.rs`: backend info and graphics-debugger status.
- `capability.rs`: queue classes, capability kinds/classes, mismatch details, capability summary, and class reports.
- `command.rs`: render command/query DTOs, advanced payload source labels, and viewport descriptors.
- `quality.rs`: render feature quality settings and `RenderQualityProfile`.
- `tests.rs`: direct DTO behavior coverage moved out of the parent.

## Design Rationale

This split follows `docs/plans/engine-code-structure-convention.md` R1.4/R4.3: split by ownership, keep root files as facades, and move test bodies out of large production files. The original file had grown to 1969 lines and mixed several unrelated report families with inline tests. After the split the parent is 374 lines and every child owner is under the 800-line soft budget.

The split is behavior-neutral. Public names are re-exported from `backend_types.rs`, so downstream code continues to consume the same framework API through `zircon_runtime::core::framework::render::*`. The only intended change is source ownership.

## Test Coverage

`backend_types/tests.rs` keeps the moved unit coverage for history copy counts, camera target writeback marker semantics, graph execution neutral counts, quality profile TAA defaults, and capability class/gpu-driven/HZB support decisions.

`runtime_15_render_backend_types_are_child_owners` locks the structure: moved owners may not return to the parent, all parent/child files must stay under the line budget, and the render index, structure convention, review findings, and this module doc must carry the same status anchor.

During this slice, scoped `rustfmt --check`, source/line-count scans, docs anchor scans, and diff checks passed. Cargo/WGPU/RenderDoc validation was deferred because active compile lanes were already present, so this document does not claim a new Cargo or graphics-debugger pass.
