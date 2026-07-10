# Interface Family

Use this reference to map roadmap vocabulary to the current Rust-facing contracts.

## Concept Mapping

| Concept name | Rust contract | Owning path | Notes |
| --- | --- | --- | --- |
| `IEntry` | `EngineEntry` | `zircon_app` | Owns profile, run mode, module owner set, and bootstrap flow. |
| `IModule` | `EngineModule` | `zircon_runtime::engine_module` | Owns a real module type that emits a `ModuleDescriptor`. |
| `IService` | `EngineService` | `zircon_runtime::engine_module` | Metadata-level contract for owner module, registry name, kind, startup mode, and dependencies. |
| `IDriver` | `EngineDriver` | `zircon_runtime::engine_module` | Marker contract for low-level services; use driver-only dependency direction. |
| `IManager` | `EngineManager` | `zircon_runtime::engine_module` | Marker contract for higher-level services; expose them through `zircon_runtime::core::manager` resolvers and `zircon_runtime::core::framework` contracts instead of concrete cross-crate objects. |
| `IPlugin` | `EnginePlugin` | `zircon_runtime::engine_module` | Reserved for the VM/plugin runtime path; do not equate it with normal modules or managers. |
| `IObject` | `RuntimeObject` | `zircon_runtime::scene` | Runtime semantic role only. Do not treat it as a global inheritance root. |
| `ISystem` | `RuntimeSystem` | `zircon_runtime::scene` | Schedule or manager-owned runtime system semantics. |
| `IEntity` | `EntityIdentity` | `zircon_runtime::scene` | ECS identity contract only. |
| `IComponent` | `ComponentData` | `zircon_runtime::scene` | ECS data contract only. |

## Dependency Rules

- Drivers may depend on drivers only.
- Managers may depend on drivers and managers.
- Plugins stay on the VM/plugin path and should not be used as a generic escape hatch for lifecycle bypasses.
- `EngineService` does not replace subsystem-specific framework or manager contracts. It describes shared runtime metadata around those services.

## ECS Exceptions

- `EntityIdentity` is about stable identity and hierarchy addressing, not behavior inheritance.
- `ComponentData` is about attachable runtime data, not an OO interface hierarchy.
- `RuntimeSystem` may be implemented by system-like runtime objects such as `LevelSystem`, but this does not justify moving scene execution back to scene-node object ownership.
