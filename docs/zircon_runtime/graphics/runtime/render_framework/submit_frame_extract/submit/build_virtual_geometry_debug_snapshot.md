---
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/page.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/node_cull.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/execution.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/support.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_build_virtual_geometry_debug_snapshot.rs
implementation_files:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/page.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/node_cull.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/execution.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/support.rs
plan_sources:
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/index.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - user: 2026-06-24 implement WGPU-to-render pipeline design from render plans with structure/review priorities
tests:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_build_virtual_geometry_debug_snapshot.rs
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/index.md
doc_type: module-detail
---

# VG Debug Snapshot Submit Owner

## Purpose

This module builds the frame-owned `RenderVirtualGeometryDebugSnapshot` that bridges submit-time WGPU render evidence back into runtime diagnostics and product tests. The snapshot is consumed by runtime-frame construction, mesh-level virtual-geometry indirect planning, debug overlays, and later plan evidence for RenderDoc/WGPU verification.

VG debug snapshot owner split records `render_plan02_vg_debug_snapshot_owner_split_static_passed_cargo_deferred_active_compile_lane`. It keeps the root submit module short enough to inspect while preserving the same output shape and call site from `build_runtime_frame.rs`.

## Related Files

`build_virtual_geometry_debug_snapshot.rs` is now the orchestration root. It pulls source extract data from `FrameSubmissionContext`, asks child owners for cull input, page inspections, node/cluster cull replay, execution segments, selected clusters, visbuffer evidence, and hardware rasterization records, then assembles the final DTO.

`page.rs` owns page residency, pending request inspection, evictable page inspection, available slot projection, and the `RenderVirtualGeometryCullInputSnapshot` counters.

`node_cull.rs` owns the node/cluster cull replay model: global state, dispatch setup, launch worklist, instance seeds, instance/cluster/child work items, traversal records, and derived page request IDs.

`execution.rs` owns render-path execution evidence from visibility VG draw segments: page state classification, repeated draw accounting, indirect offsets, submission records, selected cluster expansion, visbuffer marks, visbuffer64 entries, and hardware rasterization records.

`support.rs` owns the shared saturating `usize` to `u32` conversion helper used by all three child owners.

## Behavior Model

The root returns `None` when the frame has no virtual-geometry extract. Otherwise it uses the already-prepared page upload plan, visibility context, and provider feedback from `FrameSubmissionContext`.

Page data is derived before node/cluster cull replay so the cull-input snapshot can report resident, pending, available, and evictable counts. Node/cluster cull replay uses the submit-time main-camera visibility record when present, preserving the same camera authority as runtime visibility.

Execution evidence is derived from `visibility_context().virtual_geometry_draw_segments`. Segments with missing pages are counted as missing and excluded from executable submission records. Resident and pending segments still produce execution segments, selected cluster evidence, and draw submission records so mesh-level indirect planning can inspect render-path intent separately from residency outcome.

## Design and Rationale

The old single file mixed page cache inspection, hierarchy traversal replay, execution classification, visbuffer evidence, and DTO assembly in one near-1000-line owner. That shape made Plan 02 work harder to review and violated the engine structure convention's R1.4 owner budget intent.

The split follows ownership boundaries rather than line-count slices. The root stays responsible for final DTO assembly because it is the only place that needs the complete output shape. Each child can now evolve with its own tests or RenderDoc evidence without reopening unrelated page, cull, or execution logic.

No compatibility wrapper was added. The existing module path remains mounted by `submit/mod.rs`, and the existing public-in-parent function name stays as the only call surface used by `build_runtime_frame.rs`.

## Test Coverage

`runtime_15_render_vg_debug_snapshot_is_child_owner_split` is the static structure guard for this slice. It verifies parent/child module ownership, moved-definition boundaries, line budgets, and documentation/status anchors.

As of status `render_plan02_vg_debug_snapshot_owner_split_static_passed_cargo_deferred_active_compile_lane`, scoped rustfmt, static owner scans, line-count scans, docs-anchor scans, touched-file whitespace scans, and scoped diff-check are the immediate implementation evidence. Cargo, WGPU product tests, and RenderDoc capture remain deferred while active compile lanes are present and belong to the later Plan 02 testing/evidence stage.

## Plan Sources

The split is driven by Plan 02 MD-M2/MD-M4 virtual-geometry execution evidence work, the render index structure rules, and the engine-wide code structure/review findings that prioritize R1.4 owner budgets and render hot-path maintainability.

## Open Issues

This slice does not claim new WGPU or RenderDoc acceptance evidence. The next validation window still needs focused Plan 02 VG/debug snapshot guard execution, product-path reruns, and mesh-level indirect/execution buffer capture evidence once compile lanes are available.
