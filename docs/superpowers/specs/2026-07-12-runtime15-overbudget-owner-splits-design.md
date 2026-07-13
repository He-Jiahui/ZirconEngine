# Runtime 15 Overbudget Owner Splits Design

**Date:** 2026-07-12

**Status:** Approved for implementation

**Scope:** Close the current Runtime 02 / Runtime 15 priority structure gates without changing runtime behavior, public APIs, or file-budget thresholds.

## Context

The current-source Runtime structure-convention harness passes 1297 of 1304 tests. The failures reduce to six unique overbudget owners plus one guard drift introduced by the active Physics 03 folder-backed split:

- `graphics/scene/scene_renderer/environment/ibl_bake_wgpu_command_plan.rs`: 802 lines, global production budget 800;
- `graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs`: 810 lines, global production budget 800;
- `graphics/scene/scene_renderer/ui/render.rs`: 802 lines, global production budget 800;
- `graphics/text/layout/rich.rs`: 831 lines, global production budget 800;
- `graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/compute_workload.rs`: 690 lines, owner budget 680;
- `graphics/scene/scene_renderer/advanced_lighting/froxel/integrate/tests.rs`: 861 lines, global test budget 800;
- the Physics property-access structure guard still expects collider-shape anchors in `physics.rs` after the implementation moved them to `entries/collider_shape.rs`.

`asset/project/manager/scan_and_import.rs` was previously part of the failure set. The Frameworks owner has already split it into focused child modules and reduced the parent to 544 lines, so this design preserves that work and makes no further change there.

The code-review-findings harness independently passes 80 of 80 current-source tests. This slice must preserve that result.

Current-worktree owner activity after the initial design review has already moved the six named files below their budgets. Five of the seven previously failing exact structure tests now pass. The remaining two failures are guard-routing drift: the Physics guard does not yet read `collider_shape.rs`, and the render-graph execution-record guard does not yet read `compute_workload/tests.rs` after the test split. Implementation therefore adopts the existing owner splits and changes only the two stale guards unless a later current-source validation exposes another concrete issue.

## Goals

1. Bring every named owner below its existing budget through folder-backed responsibility splits.
2. Preserve all current dirty-worktree behavior and all existing public and crate-visible paths.
3. Keep root modules focused on declarations, wiring, and orchestration.
4. Update the Physics structure guard to validate the new parent/child ownership boundary rather than requiring implementation anchors in the parent.
5. Produce current-source structure, focused behavior, compilation, and plan-status evidence before recording completion.

## Non-goals

- No file-budget increase, exemption, allowlist, whitespace-only reduction, or assertion weakening.
- No runtime feature addition or behavior change.
- No compatibility facade, duplicate implementation, or re-export-only migration layer.
- No overwrite, reset, or rollback of work owned by Text 07, Render 18, Frameworks 03, Physics 03, Shader 06, or other active sessions.
- No broad formatting or cleanup outside the exact split paths.

## Design

### 1. IBL command-plan tests

Convert the inline `#[cfg(test)] mod tests { ... }` in `ibl_bake_wgpu_command_plan.rs` to `#[cfg(test)] mod tests;` and move the complete test body to:

`environment/ibl_bake_wgpu_command_plan/tests.rs`

The production owner remains unchanged. Test names, assertions, helper functions, and visibility remain identical. The parent falls well below 800 lines.

### 2. Render-pass GPU context setup

Add:

`graph_execution/render_pass_execution_context/gpu/context.rs`

Move the coherent context setup group from `gpu.rs` into an `impl RenderPassGpuExecutionContext` in the child module:

- production and test constructors;
- immutable context accessors;
- resource resolver and streamer accessors;
- `with_*` dependency-injection builders;
- render-region and attachment-policy accessors.

`gpu.rs` retains the context struct, its `Debug` implementation, module wiring, stage-recording orchestration, and small root helpers. The existing type and method paths do not change because the child adds inherent methods to the same parent type.

### 3. Screen-space UI prepared data

Add:

`scene_renderer/ui/render/prepared.rs`

Move the three related prepared-render data declarations into the child:

- `PreparedScreenSpaceUi`;
- `ScreenSpaceUiDraw`;
- `ScreenSpaceUiTextBatch`.

The root imports or privately re-exports the declarations so current sibling paths such as `super::ScreenSpaceUiTextBatch` continue to resolve. Rendering orchestration and Text 07's current batch-building behavior remain in place. This is the smallest responsibility split that clears the budget without colliding with active rich-text logic.

### 4. Rich-layout tests

Convert the inline test module in `graphics/text/layout/rich.rs` to a folder-backed test owner:

`graphics/text/layout/rich/tests.rs`

Only tests and their local helpers move. Rich layout, wrapping, inline metrics, and style resolution stay in the production owner. Test names and assertions remain unchanged.

### 5. Compute-workload tests

Convert the inline test module in `render_graph_execution_record/compute_workload.rs` to:

`render_graph_execution_record/compute_workload/tests.rs`

Only tests and test-local fixtures move. Dispatch records, workload context, expected dispatch calculation, and audit records remain in the production owner. The parent falls below its stricter 680-line owner budget.

Update the render-graph execution-record structure guard so it validates production anchors in `compute_workload.rs` and the moved test anchors in `compute_workload/tests.rs`. All existing anchors remain required; only their authoritative child source changes.

### 6. Froxel light-scatter product tests

Add:

`advanced_lighting/froxel/integrate/tests/light_scatter_product.rs`

Move the complete light-scatter/integrate GPU product group into the child:

- the live WGPU light-grid and shadow-atlas product test;
- the ignored PNG/report export test;
- `VolumetricChainResult`;
- the GPU chain, resource creation, readback, image, and adapter helpers used only by that product.

The parent `tests.rs` retains ABI/shader-contract tests and child-module declarations for `light_scatter_product` and the existing `temporal_product`. Both test owners remain below 800 lines.

### 7. Physics property-access guard ownership

Update the Runtime 15 structure guard for scene-world physics property entries so it validates both sides of the new boundary:

- `physics.rs` must declare and call the `collider_shape` child owner;
- `collider_shape.rs` must contain `Collider.shape.kind` projection and the required `ColliderShape` variants.

The guard must read both files and keep every existing semantic anchor. This is a guard-routing correction, not an expectation relaxation.

## Concurrent-work preservation

Before each edit, the implementation claims exact-path leases and re-reads the current file. If a path is leased, the split waits or narrows to a non-conflicting child path. All moves use the current worktree as the source; no content is reconstructed from `HEAD`, and no unrelated dirty file is staged.

For untracked owner files, the complete current content is treated as authoritative. Parent and newly created child files are staged together only after focused validation proves the module still compiles.

## Validation

Validation proceeds from narrow to broad:

1. run `rustfmt` only on touched Rust files;
2. run `git diff --check` on the exact paths;
3. compile a current-source standalone structure harness and run the seven previously failing exact tests, including the Physics and compute-workload child-source routing checks;
4. run the complete current structure-convention harness and require 1304/1304;
5. compile and run the current code-review-findings harness and require 80/80;
6. run focused IBL, UI/text, render-graph compute, froxel, and physics tests selected from the moved modules;
7. run a coordinator-managed Windows-native `cargo check -p zircon_runtime --lib --locked --jobs 1`;
8. run the required Runtime 02 downstream Cargo/editor/plugin gates when shared target capacity and active owners permit them.

Any failure is diagnosed at the lowest owning layer. No higher-level gate is claimed from static inspection alone.

## Documentation and status records

After validation, update the Runtime 02 evidence record with the new current-source results and update the Runtime 15/status mirrors only through their authorized plan-output workflow. A plan or milestone is marked complete only when its own listed gates have authoritative evidence.

The `close-session-goal-milestones` workflow is used only after the accepted milestone has no remaining required work. It must not stage unrelated dirty changes or close the full runtime-architecture goal while Runtime 03–15 work remains.
