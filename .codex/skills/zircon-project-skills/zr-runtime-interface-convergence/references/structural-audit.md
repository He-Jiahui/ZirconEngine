# Structural Audit

This reference captures the current repository-level diagnosis behind the convergence skill.

## Confirmed Foundations

- `zircon_runtime::core::runtime` owns the descriptor-backed runtime spine: `CoreRuntime`, `ModuleDescriptor`, `DriverDescriptor`, `ManagerDescriptor`, `PluginDescriptor`, lifecycle state, and dependency resolution.
- `zircon_runtime::engine_module` exposes the shared module-level abstraction family: `EngineModule`, `EngineService`, `EngineDriver`, `EngineManager`, and `EnginePlugin`.
- `zircon_app` now has an explicit `EngineEntry` contract and a `BuiltinEngineEntry` implementation that boots from module owner objects instead of only free descriptor functions.
- `zircon_runtime::core::framework` owns the shared subsystem contracts and neutral DTO surfaces that used to drift across higher-level crates.
- `zircon_runtime::core::manager` owns manager-facing handles, resolvers, activation-facing access helpers, and stable service names.
- `zircon_runtime` now serves as the runtime absorption layer for built-in high-level subsystems, while `zircon_editor` remains the editor host and authoring-state owner.

## Still Open Gaps

- The runtime script/plugin path still hosts plugins through `VmPluginManager`, but the Core-level plugin lifecycle is not yet broadly exercised through real `PluginDescriptor` registrations and `resolve_plugin` consumers.
- `zircon_app` still owns the entry-side bootstrap contract, but optional runtime extension module assembly should stay under `zircon_runtime` instead of re-expanding the app crate dependency fan-out.
- Any remaining non-network `*server*` type names, handles, or docs should be treated as naming drift that must converge into `zircon_runtime::core::manager`, `zircon_runtime::core::framework`, `zircon_runtime`, or `zircon_editor` terminology instead of being preserved.

## Current Module-Convergence Pressure

Production module owners should not rely on `stub_module_descriptor`; any future hit outside test fixtures is a hard-cutover regression. The current pressure points are runtime-owned subsystem surfaces that still need narrow owner files, descriptor wiring, and focused boundary tests instead of root-level expansion:

- `zircon_runtime::platform`
- `zircon_runtime::ui`
- `zircon_runtime::graphics`
- `zircon_runtime::scene`
- `zircon_plugins::{navigation,net,particles,sound,texture}`
- plugin-backed runtime modules under `zircon_plugins/*/runtime`

## Immediate Refactor Pressure

- `zircon_editor` still contains large boundary hotspots such as `editing/ui_asset/session.rs` and `host/manager/ui_asset_sessions.rs`.
- `zircon_runtime::graphics` still contains large and actively changing subsystems that should keep splitting by behavior family instead of accreting at the same surface.
- `zircon_app` should stay a host/profile shell and avoid reclaiming optional runtime module ownership that already belongs in `zircon_runtime`.
- `zircon_runtime::scene` and `zircon_editor::scene` must keep editor-only authoring state out of the runtime world authority model.
- Any crate that exposes `module_descriptor()` but does not clearly separate owner type, descriptor wiring, and runtime behavior should be treated as `needs-refactor` until the tree is easier to navigate.

## Audit Standard

Before calling a crate or subsystem owner `converged`, confirm all of the following:

- it has a real `EngineModule` owner,
- it does not rely on `stub_module_descriptor`,
- its touched surface is not already a boundary hotspot,
- it does not reintroduce non-network `server` naming,
- and the change does not rely on pretending the plugin path is more complete than it currently is.
