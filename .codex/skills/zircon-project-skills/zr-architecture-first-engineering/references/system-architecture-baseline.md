# System Architecture Baseline

Use this reference to align daily design decisions with the current repository-wide engine target. This file is a checklist and summary, not a replacement for the roadmap plans. Read the full plans whenever the task changes ownership boundaries, lifecycle rules, ECS authority, runtime or editor absorption, or VM plugin contracts.

## Authority Sources

- Primary roadmap: `.codex/plans/全系统重构方案.md`
- Binding convergence rules for runtime/editor architecture: `.codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md`
- Mainstream engine evidence: `../zr-language-feature-design/SKILL.md` plus the reference trees under `dev/`
- Execution and validation workflow: `../layered-milestone-development/SKILL.md` and `../../zircon-dev/SKILL.md`

## Current Structural Invariants

### Fixed Top-Level Packages

- `zircon_app`: process entry, profile selection, and main-loop host
- `zircon_runtime`: runtime absorption layer plus the internal `core/{runtime,framework,manager,math,resource}` spine
- `zircon_editor`: editor host and authoring-state owner
- `zircon_runtime::core::runtime::CoreRuntime` is the process-level authority for registration, activation, dependency ordering, config, events, scheduling, and shutdown.
- Treat this three-package root as fixed public architecture. Supporting modules may exist inside `zircon_runtime`, but they must not reappear as competing root packages.

### Runtime-Internal Supporting Rule

- Shared foundations live under `zircon_runtime::core`, not as separate root packages.
- `zircon_runtime::core::math`, `zircon_runtime::core::resource`, `zircon_runtime::rhi`, `zircon_runtime::rhi_wgpu`, and `zircon_runtime::render_graph` are supporting runtime infrastructure, not replacements for the fixed root packages.
- Optional extension subsystems such as `physics`, `sound`, `texture`, `net`, `navigation`, `particles`, and `animation` stay as `zircon_runtime` modules until deliberately converged further by plan.

### Naming and Cutover Rule

- Reserve `server` naming for true server-side or network semantics only.
- Treat any non-network `*server*` crate names, trait names, handles, or registry names as migration debt.
- Do not keep compatibility shims, alias crates, or legacy-path re-exports once a migration stage lands. Each stage must cut directly to the new structure.
- If active work is happening in a directory or crate that the convergence plan already declares non-compliant, migrate that work to the target directory or crate before continuing feature development there.

### Module and Service Model

- Real functionality belongs behind module descriptors and lifecycle-aware services.
- Driver and manager dependencies must be declared, not hidden in ad hoc object graphs.
- `zircon_runtime::core::manager` owns access surfaces, stable names, handles, and resolvers. It does not re-introduce framework traits or high-level DTO ownership.
- `zircon_runtime::core::framework` owns shared contracts and neutral data, but it must not implement concrete business behavior.

### Runtime and Editor Boundary Rule

- `zircon_runtime::scene` owns the runtime world with ECS plus hierarchy.
- `zircon_runtime::scene` may expose authoritative runtime state such as entity, hierarchy, transform, renderable, light, camera, serialization, and render extract data.
- `zircon_runtime::scene` must not own editor authoring state such as selection, viewport tools, gizmo state, camera overrides, or overlay generation.
- `zircon_editor::scene` owns selection, viewport tools, handle and gizmo routing, scene overlays, and authoring commands.
- Scene trees, editor views, and render snapshots are derived views, not the authority model.

### Render and App Host Rule

- `zircon_runtime::core::framework::render` owns neutral render contracts and shared packets only.
- Graphics and runtime/editor callers must meet through `zircon_runtime::core::framework::render` data, not through editor-owned concrete types.
- `zircon_app` is the process host. It creates `CoreRuntime`, registers built-in modules, chooses the active profile, and drives editor/runtime/headless loops.
- Runtime preview or editor host state that does not belong in the runtime world stays in app/runtime host controllers, not in world serialization.

### Plugin Rule

- Plugins are VM-oriented contracts, not direct Rust dynamic-library object sharing.
- Stable host handles, capability negotiation, and state migration define hot reload behavior.

## Architecture Questions to Answer Before Coding

### Boundary Ownership

- Which fixed role owns the new capability: app host, runtime-internal core kernel, runtime-internal manager access layer, runtime-internal framework contract layer, runtime absorption layer, editor authoring host, supporting runtime module, or VM plugin path?
- If the answer is not one of those, is the design inventing a fourth root package by accident?

### Lifecycle Shape

- Is the capability `Immediate` or `Lazy`?
- Which dependencies must be declared?
- What shutdown or hot-reload guarantees must remain valid?

### Public Surface

- Should consumers receive a descriptor, a typed facade, a stable handle, a manager, a system registration hook, or a command interface?
- Which layers are allowed to know about the concrete implementation?
- Is the public contract owned by `zircon_runtime::core::framework`, accessed through `zircon_runtime::core::manager`, or confined to `zircon_runtime` or `zircon_editor` internals?

### Data Model

- Does the change introduce ECS components, render-extract data, config objects, manifests, or serialized project artifacts?
- Is the data authoritative at runtime, or derived from another authority?
- Does any editor-only state leak back into runtime world serialization or runtime-owned ECS data?

### Validation

- Which layers need tests: lifecycle, dependency ordering, ECS propagation, render extraction, serialization, VM state migration, or error handling?
- Which sibling crates need stub or skeleton integration so the workspace architecture stays coherent?
- Does the validation prove that old path aliases and old naming are gone instead of only proving the new path was added?
- Does the execution order prove that ongoing work did not keep advancing inside a directory that should have been migrated first?

## Acceptable Leaf Exceptions

Direct implementation is acceptable only when all of these are true:

- The work stays entirely inside one existing abstraction.
- No new cross-crate dependency or public entry point appears.
- No lifecycle, manager/framework/runtime/editor ownership, ECS authority, or plugin contract changes.
- No legacy naming, alias path, or shim path must stay alive to make it work.
- Another feature would not need a new reusable boundary to share the same logic.

If any condition fails, stop and design the framework slot first.
