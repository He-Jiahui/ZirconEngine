---
related_code:
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/root_surface.rs
implementation_files:
  - zircon_runtime/src/tests/runtime_absorption/root_surface.rs
  - zircon_runtime/src/tests/runtime_absorption/mod.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_runtime/runtime/index.md
tests:
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/root_surface.rs zircon_runtime/src/tests/runtime_absorption/mod.rs
  - root_surface_guard_static_passed static checks passed 2026-06-13
  - pre_m3_type_alias_guard_static_passed_pending_render_owner static checks passed 2026-06-13
  - root_surface_interface_convergence_mirror_uses_current_audit_counts added 2026-06-14; Cargo pending active compile lanes
  - rustc --edition 2021 --test zircon_runtime/src/tests/runtime_absorption/root_surface.rs passed 4/4 on 2026-06-13
  - git diff --check -- Runtime 02 root-surface scoped files: passed 2026-06-13 with LF-to-CRLF warnings only
doc_type: module-detail
---

# Runtime Root Surface

## Current Public Root Surface

`zircon_runtime/src/lib.rs` currently exposes 20 public module declarations. This is a namespace surface, not a flattened type surface. The public modules are `core`, `diagnostic_log`, `dynamic_api`, `engine_module`, `prelude`, `animation`, `asset`, `scene`, `ui`, `graphics`, `render_graph`, `rhi`, `rhi_wgpu`, `builtin`, `foundation`, `input`, `navigation`, `platform`, `plugin`, and `script`.

The crate root has three public `pub use` sites:

- `crate::core::resource`
- `zircon_runtime_reflection_macros::{zircon_host_function, zircon_host_module, ZirconScriptType}`
- `builtin::{RuntimeModuleLoadReport, RuntimePluginId, RuntimeRequiredPluginMissing, RuntimeTargetMode}`

Subsystems such as `graphics`, `render_graph`, `rhi`, `rhi_wgpu`, `ui`, `input`, `scene`, `asset`, and `plugin` are exposed as namespaces. They must not add flattened root `pub use` surfaces without updating Runtime 02 and its guard.

Runtime 14 classifies `animation` and `navigation` as intentional crate-root runtime module-family seats. `animation` owns playback/evaluation and scene-hook application. `navigation` owns the built-in fallback runtime pathfinding implementation while advanced Recast/editor/baking behavior remains plugin-owned.

## Core Spine

`zircon_runtime/src/core/mod.rs` is limited to the decided spine: `runtime`, `framework`, `manager`, `math`, and `resource`. The former root fragments `config_store`, `event_bus`, `frame_clock`, `job_scheduler`, `lifecycle`, `modules`, `state`, and `tasks` remain retired as root modules. Curated core facade exports may still point into `core::runtime` or `core::framework`, but the old root module names must not return.

## M3 Alias Debt

`zircon_runtime/src/lib.rs` still contains crate-private graphics alias debt through `pub(crate) use graphics::...` plus `#[allow(unused_imports)]`. This is not a public API. Runtime 02 M3 must remove the alias block during a render owner window, because the call sites are in graphics/render paths that are currently being edited by the active render session.

Until that cutover happens, the guard keeps the debt explicit: graphics aliases may remain crate-private and documented, but they must not become public root re-exports.

## M3.2 Type Alias Debt

Runtime 02 M3.2 is the hard deletion slice for the crate-private type alias debt in `zircon_runtime/src/lib.rs`. The current debt includes representative graphics type symbols such as `RendererFeatureReferenceListKind`, `GraphicsError`, `SceneRenderer`, `WgpuRenderFramework`, `ViewportFrame`, `HybridGiRuntimeProvider`, `VirtualGeometryRuntimeProvider`, and `SolariRuntimeProvider`.

These names are crate-private type alias debt, not a public root API. The pre-M3.2 guard status is `pre_m3_type_alias_guard_static_passed_pending_render_owner`: the aliases remain in place only because their known call sites live under graphics/render paths that need the render owner window. The actual M3.2 cutover is still to delete the alias block and migrate callers to the real `graphics::...` owner paths.

## Guard

`zircon_runtime/src/tests/runtime_absorption/root_surface.rs` binds this document to Runtime 02 and the runtime index. It checks the crate root public module list, the three allowed public re-export sites, the absence of flattened subsystem `pub use` surfaces, the current private graphics alias debt markers, the M3.2 type alias debt status, and the core spine root modules.

`root_surface_interface_convergence_mirror_uses_current_audit_counts` also binds the interface convergence review to the current root-surface audit facts: 20 public modules, 3 public `pub use` locations, 80 crate-visible graphics re-export symbols, direct `rhi_wgpu` backend exposure, and M1 gate status `migration-debt-present`. It rejects the stale 17-module / 75-symbol mirror while the actual alias removal and backend public-surface cutover remain pending the render owner window.

`runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation` keeps the broader Runtime 02 validation lane visible after these static guards: core/root/generated/export_build_plan/app/editor/plugin checks, default lib-test reruns, and the render-owner graphics alias cutover must all have evidence before Runtime 02 can be promoted.
