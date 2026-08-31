---
related_code:
  - zircon_runtime/src/core/framework/camera_controller/mod.rs
  - zircon_runtime/src/input/camera_controller/mod.rs
  - zircon_runtime/src/dynamic_api/camera_controller.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_navigation.rs
implementation_files:
  - zircon_runtime/src/core/framework/camera_controller/controller_output.rs
  - zircon_runtime/src/input/camera_controller/free/controller.rs
  - zircon_runtime/src/input/camera_controller/orbit/controller.rs
  - zircon_runtime/src/input/camera_controller/pan/controller.rs
plan_sources:
  - user: 2026-08-24 converge Frameworks architecture without compatibility paths
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
tests:
  - tools/tests/test_frameworks_01_camera_controller_owner_boundary.py
  - zircon_runtime/src/tests/camera_controller.rs
doc_type: module-detail
---

# Runtime Camera Controllers

## Contract

`zircon_runtime::core::framework::camera_controller` owns neutral input, settings, state,
cursor intent, and transform-output DTOs. Callers map platform, editor, remote, or scripted
events into those DTOs and remain responsible for applying the returned transform through
their own scene or viewport authority.

The contract layer must not acquire raw window/input access, editor selection state, scene
mutation authority, or new camera-update algorithms.

## Ownership

`zircon_runtime::input::camera_controller` is the canonical implementation owner for the
Free, Orbit, and Pan controllers. It consumes normalized framework DTOs and math types, but
does not own platform event collection or scene writes. The old controller files and exports
under `core::framework` are removed; there is no compatibility re-export.

This split follows the repository M1 rule that `zr_contracts` receives only trait/DTO state.
Unreal and Fyrox keep viewport navigation behavior with their editor owners, while Bevy uses a
separate `bevy_camera_controller` implementation crate rather than placing behavior in its
input contracts. Zircon uses the Runtime input subsystem for behavior shared by runtime and
authoring consumers, while Editor retains viewport-specific projection and selection policy.

## Validation

`test_frameworks_01_camera_controller_owner_boundary.py` pins the physical owners, rejects
old controller imports (including grouped, glob, and qualified forms), and requires the contract
modules to export DTOs only. Runtime behavior tests continue to cover free movement, damping,
cursor intent, orbit, and pan/zoom/rotation semantics through the new implementation path.
