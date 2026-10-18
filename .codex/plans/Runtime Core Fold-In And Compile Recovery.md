# Runtime Core Fold-In And Compile Recovery

**Summary**
- First update only the skill/rule files that directly govern this task so they reflect the new repository truth: repo root keeps only `zircon_app`, `zircon_editor`, and `zircon_runtime`.
- Treat `framework` and `manager` as internal parts of `zircon_runtime::core`; remove their root-level runtime surfaces and hard-cut all code to `zircon_runtime::core::{framework,manager}`.
- Recover the workspace to a green `cargo build --workspace --locked`, then run test passes to catch post-build fallout.

**Implementation Changes**
- Skills first:
  - Rewrite the active Zircon workflow/architecture skills so they describe a 3-package root and a runtime-internal `core/framework/manager` layout.
  - Remove validation/examples that still tell Codex to build or reason about deleted top-level `zircon_core`, `zircon_framework`, or `zircon_manager`.
  - Update the boundary-discipline example that still points at removed legacy editor UI paths.
- Runtime layout:
  - Move the current runtime `framework/` and `manager/` trees under `core/`.
  - Make `core::math` the only math namespace and remove the unused root-level math surface.
  - Keep `core::resource` as the canonical resource foundation beside runtime kernel, framework contracts, and manager access.
- Contract completion:
  - Add the missing `core::framework::animation` and `core::framework::physics` contracts.
  - Extend `core::framework::scene` with the path DTOs already used by asset/editor code: `EntityPath` and `ComponentPropertyPath`.
  - Extend `core::manager` with `AnimationManagerHandle`, `PhysicsManagerHandle`, `resolve_animation_manager`, `resolve_physics_manager`, `ANIMATION_MANAGER_NAME`, and `PHYSICS_MANAGER_NAME`.
  - Extend `core::resource` with the physics-material and animation asset marker/kind coverage required by the asset loading pipeline.
  - Fix wrong-boundary imports so framework/render code consumes scene contract types from framework ownership, not from runtime scene authority.
- Import sweep:
  - Rewrite all app/editor/runtime imports from `zircon_runtime::framework` and `zircon_runtime::manager` to `zircon_runtime::core::framework` and `zircon_runtime::core::manager`.
  - Remove any remaining source-level assumptions that external `core/framework/manager` packages still exist.

**Public API Changes**
- Remove root runtime surfaces `zircon_runtime::framework::*` and `zircon_runtime::manager::*`.
- Keep only `zircon_runtime::core::framework::*` and `zircon_runtime::core::manager::*`.
- Keep only `zircon_runtime::core::math::*` for public math types.
- Add or restore these contract families where already required by current code:
  - animation: `AnimationTrackPath`, `AnimationParameterValue`, `AnimationPlaybackSettings`, `AnimationManager`
  - physics: `PhysicsSettings`, `PhysicsSimulationMode`, `PhysicsCombineRule`, `PhysicsMaterialMetadata`, `PhysicsManager`
  - scene paths: `EntityPath`, `ComponentPropertyPath`
  - resource markers: physics-material and animation asset markers

**Test Plan**
- Required build gate: `cargo build --workspace --locked`
- Required compile regression: `cargo test --workspace --locked --no-run`
- Required targeted runtime/editor regression:
  - `cargo test -p zircon_runtime --locked`
  - `cargo test -p zircon_editor --locked`
- Focus checks:
  - animation track path parsing still works in runtime asset tests and editor binding dispatch tests
  - physics and animation managers resolve through `core::manager`
  - no remaining code imports framework/manager from the runtime root
  - updated skills no longer describe deleted top-level `zircon_core / zircon_framework / zircon_manager`

**Assumptions**
- The latest user direction overrides the older six-crate skill language: root packages are now only `zircon_app`, `zircon_editor`, and `zircon_runtime`.
- No compatibility shim is kept for root-level `zircon_runtime::framework` or `zircon_runtime::manager`.
- This pass updates only the skill files and references that directly affect current architecture/build decisions, not the full historical docs set.
