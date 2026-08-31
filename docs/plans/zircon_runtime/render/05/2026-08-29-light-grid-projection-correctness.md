---
title: Light Grid Projection Correctness
doc_type: implementation-record
status: source_complete_isolated_tests_passed_managed_validation_pending
owner: Runtime Render 05 / 09E P0-3
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_grid_builder.rs
  - zircon_runtime/src/core/framework/render/view_matrix_pair.rs
references:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/LightGridInjection.cpp
  - dev/UnrealEngine/Engine/Shaders/Private/LightGridInjection.usf
  - docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
  - docs/plans/optimize/zircon_runtime/95-runtime-direct-lighting-photometry-light-grid-clustered-forward-plus-shadow-atlas-cascade-point-spot-rect-cookie-ies-submission-product-integration-current-source-review.md
---

# Light Grid Projection Correctness

## Decision

The CPU light-grid builder now consumes the canonical projection owner already shared by render,
visibility and picking; `ortho_size` is the orthographic half-height. A perspective sphere that
crosses the near plane is no longer rejected from its center clip position. Camera-inside spheres
receive a conservative full-viewport tile rect; other near-crossing spheres project from the near
plane while retaining the existing sphere radius expansion. Spheres wholly behind the camera remain
rejected by the existing depth-range test.

This is a correctness repair for RDL-P1-020/RDL-P1-021 and does not change tile budgets, z-bin
layout, light ordering, light-type approximation, allocation strategy or GPU scheduling.

`ViewProjectionMatrixPair::projection_from_camera` is the sole crate-local projection constructor;
the light-grid view descriptor stores its result rather than reimplementing camera matrix formulas.

## Reference Review

Before implementation, the current builder, camera projection owners and the Unreal reference were
re-read. Unreal's light-grid path publishes view-space position/radius and tests each light against
cell bounds rather than rejecting a light solely from its center clip coordinate. The corresponding
Zircon change keeps the existing CPU sphere model but makes the screen bound conservative at the
near-plane singularity. Existing engine owners confirm `ortho_size` is consumed directly as half-height
in `view_matrix_pair`, editor projection and picking.

## Evidence

- RED isolation before the fix: orthographic matrix Y scale was `0.2` for `ortho_size = 10`, and a
  camera-inside point light produced no tile mask.
- Post-fix E-drive rustc harness assertions pass for canonical orthographic scale, camera-inside
  full-tile coverage, behind-camera-center near crossing, and fully-behind-camera rejection.
- Added inline regressions cover the same four cases plus shared projection-owner equality in the
  production module.
- The actual production module compiled and ran the five focused tests in an E-drive rustc harness;
  scoped rustfmt and `git diff --check` also pass. The
  managed Cargo/WGPU/product lane remains pending behind the shared validation coordinator.

Status token:
`runtime_render05_light_grid_projection_correctness_isolated_tests_passed_managed_validation_pending`.
