---
related_code:
  - zircon_runtime/src/engine_module/mod.rs
  - zircon_runtime/src/engine_module/engine_module.rs
  - zircon_runtime/src/engine_module/engine_service.rs
  - zircon_runtime/src/engine_module/service_factory.rs
  - zircon_runtime/src/engine_module/contexts.rs
  - zircon_runtime/src/engine_module/descriptors/names.rs
  - zircon_runtime/src/core/runtime/descriptors/module_descriptor.rs
  - zircon_runtime/src/core/runtime/descriptors/service_factory.rs
  - zircon_runtime/src/core/runtime/handle/registration/register_module.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/plugins/builder.rs
implementation_files:
  - zircon_runtime/src/engine_module/mod.rs
  - zircon_runtime/src/engine_module/engine_module.rs
  - zircon_runtime/src/engine_module/engine_service.rs
  - zircon_runtime/src/engine_module/service_factory.rs
  - zircon_runtime/src/engine_module/contexts.rs
  - zircon_runtime/src/engine_module/descriptors/names.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md
tests:
  - zircon_runtime/src/engine_module/tests.rs
  - zircon_runtime/src/engine_module/tests.rs::engine_module_declared_layer_does_not_own_runtime_lifecycle
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - "pending: cargo test -p zircon_runtime --lib engine_module --locked"
doc_type: module-detail
---

# Engine Module Relationship

`zircon_runtime::engine_module` is the declaration layer on top of `core::runtime`. It is not the lifecycle runtime itself.

The relationship is intentionally layered:

| Concept | Owner | Responsibility |
|---|---|---|
| `ModuleDescriptor`, `DriverDescriptor`, `ManagerDescriptor`, `PluginDescriptor` | `core::runtime` | Stored runtime contract consumed by registration, activation, dependency ordering, and service resolution. |
| `ServiceFactory`, `PluginFactory` | `core::runtime` | Runtime factory ABI used when services are instantiated. |
| `CoreRuntime`, `CoreHandle`, lifecycle states | `core::runtime` | Runtime state, registration validation, startup/shutdown, and lookup. |
| `EngineModule` | `engine_module` | High-level trait used by built-in modules, app plugin groups, and descriptor-backed app composition. |
| `EngineService` contracts | `engine_module` | Read-only contract view over driver/manager/plugin descriptor metadata. |
| `factory`, `plugin_factory`, `qualified_name`, `dependency_on` | `engine_module` | Ergonomic helpers that construct core descriptors without owning lifecycle semantics. |

## Call Surface

Runtime 14 M0.4 inspected current usage and found `EngineModule` used by built-in module assembly, `zircon_app` entry composition, app plugin groups, and module declarations such as animation, asset, foundation, graphics, input, navigation, platform, scene, script, and UI.

That usage means deleting the layer would push app-facing composition directly onto low-level descriptor structs. Moving it into `core::runtime` would make the core runtime depend on the higher-level "engine module" authoring vocabulary and would blur the existing descriptor/runtime split.

## Judgement

The Runtime 14 judgement is **declared layering**:

- Keep `engine_module` as a crate-root declaration family.
- Keep `core::runtime` as the sole owner of registration, lifecycle, service storage, dependency ordering, and activation.
- Do not add parallel lifecycle state or service registries to `engine_module`.
- When a helper starts needing runtime mutation or resolution, place that behavior in `core::runtime` and expose only a declaration-facing wrapper if call sites need one.

No hard cutover is required for Runtime 14 M0.4. Runtime 14 M1 adds `engine_module_declared_layer_does_not_own_runtime_lifecycle` to assert that the declaration files do not introduce registration, lifecycle, registry storage, or runtime state ownership.
