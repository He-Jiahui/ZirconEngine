# Workspace Map

- `Cargo.toml`: workspace members, shared dependency versions, resolver, and default members.
- `.github/workflows/ci.yml`: canonical CI commands and Linux system dependencies for the workspace.
- `zircon_app/src/lib.rs`: process entry contracts, profile selection, and host bootstrap flow.
- `zircon_runtime/src/lib.rs`: runtime absorption surface for built-in high-level subsystems.
- `zircon_runtime/src/core/mod.rs`: runtime-internal kernel surface that now owns the old core/framework/manager/math/resource roles.
- `zircon_runtime/src/core/runtime/mod.rs`: lifecycle management, service registry, and shared runtime primitives.
- `zircon_runtime/src/core/manager/mod.rs`: stable service names, resolvers, and handles that route access into framework-backed managers.
- `zircon_runtime/src/core/framework/mod.rs`: shared framework contracts and neutral DTOs for runtime/editor-facing subsystems.
- `zircon_runtime/src/core/math/mod.rs`: canonical public math namespace.
- `zircon_runtime/src/core/resource/mod.rs`: canonical resource foundation and markers.
- `zircon_editor/src/lib.rs`: editor host, authoring state, and editor-facing scene or UI surfaces.

Use this map to decide whether a change is crate-local or whether it crosses shared runtime boundaries and therefore needs broader validation. Treat any remaining non-network `*server*` naming as migration debt, not as the preferred workspace map. If a task starts from a directory outside this converged ownership map, migrate it into the compliant destination before continuing the task.
