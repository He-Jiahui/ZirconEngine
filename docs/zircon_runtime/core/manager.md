---
related_code:
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/core/manager/service_names.rs
  - zircon_runtime/src/core/manager/tests.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/state/service_entry.rs
  - zircon_runtime/src/tests/extensions/manager_handles.rs
  - zircon_runtime/src/core/framework/ai/manager.rs
  - zircon_plugins/ai/runtime/src/module.rs
  - zircon_plugins/ai/runtime/src/manager.rs
implementation_files:
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/core/manager/service_names.rs
  - zircon_runtime/src/core/manager/tests.rs
  - zircon_runtime/src/tests/extensions/manager_handles.rs
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - docs/engine-architecture/runtime-interface-convergence.md
tests:
  - zircon_runtime/src/core/manager/tests.rs
  - zircon_runtime/src/tests/extensions/manager_handles.rs
  - rustfmt --edition 2021 --check zircon_runtime\src\core\manager\tests.rs zircon_runtime\src\tests\extensions\manager_handles.rs
doc_type: module-detail
---

# Core Manager Spine

`zircon_runtime::core::manager` is the runtime-internal access layer for stable manager service names, typed service holders, and resolver helpers. It sits between `core::runtime` service registration and higher-level runtime/plugin consumers. This keeps manager access descriptor-driven and avoids reviving the old standalone manager crate shape.

## Boundary

The module owns three pieces of public contract:

- `service_names.rs` defines canonical `&'static str` manager service names.
- `resolver.rs` defines typed holder structs such as `RenderingManagerHandle`, `AiManagerHandle`, `NetManagerHandle`, and `SoundManagerHandle`.
- `mod.rs` re-exports only the stable manager traits, handles, resolver functions, and canonical names that upper layers need.

Concrete behavior stays outside this module. For example, the AI runtime plugin owns `DefaultAiManager` and registers it as an implementation service, while `core::manager` owns only `AI_MANAGER_NAME`, `AiManagerHandle`, and `resolve_ai_manager(...)`.

## Performance Contract

Manager service names are static constants, so callers do not allocate or format registry names on the hot path. `CoreHandle::resolve_manager(...)` still locks the runtime service table and returns an `Arc` clone of the cached service holder. That is acceptable at setup, module activation, and coarse workflow boundaries, but repeated per-entity or per-frame inner-loop lookup should cache the returned `Arc<dyn ManagerTrait>` in the caller's runtime state.

The resolver layer intentionally returns trait-object `Arc`s instead of concrete plugin managers. This keeps plugin implementations replaceable and prevents app/editor code from depending on plugin-private manager structs.

## Current Coverage

`core::manager::tests` locks the canonical service-name strings for resource, input, config, event, rendering, render-framework, level, AI, net, physics, animation, and sound managers.

`tests/extensions/manager_handles.rs` protects the cross-crate contract for first-party manager-backed plugins. It checks that plugin modules keep framework-backed default managers and that `core::manager` exports the matching handles, resolver helpers, and canonical service names. The guard now includes AI alongside physics, animation, net, and sound.

## Follow-Up

M5 performance work should inspect call sites that invoke resolver helpers from repeated update loops. Those call sites should resolve once during setup or activation, store the trait-object `Arc`, and use the cached manager handle during tick/render/extract work.
