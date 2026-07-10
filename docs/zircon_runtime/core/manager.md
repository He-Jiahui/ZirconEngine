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
  - zircon_runtime/src/core/framework/sound/manager.rs
  - zircon_plugins/sound/runtime/src/module.rs
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
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
tests:
  - zircon_runtime/src/core/manager/tests.rs
  - zircon_runtime/src/tests/extensions/manager_handles.rs
  - rustfmt --edition 2021 --check zircon_runtime\src\core\manager\tests.rs zircon_runtime\src\tests\extensions\manager_handles.rs
  - rustfmt --edition 2021 --check zircon_runtime\src\core\manager\resolver.rs zircon_runtime\src\core\manager\tests.rs (2026-06-11 M5 manager resolver direct projection: passed)
  - manager resolver direct-projection source guard for `let holder = core.resolve_manager::<$holder>($service_name)?;`, `Ok(holder.shared())`, and no old `.map(|holder| holder.shared())` closure adapter (2026-06-11 M5 manager resolver direct projection: passed)
  - conflict-marker, trailing-whitespace, and git diff --check scans over zircon_runtime/src/core/manager/resolver.rs, zircon_runtime/src/core/manager/tests.rs, docs/zircon_runtime/core/manager.md, and .codex/sessions/20260604-1232-runtime-architecture-review.md (2026-06-11 M5 manager resolver direct projection: passed with expected LF-to-CRLF warnings only for tracked files)
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json (2026-06-11 M5 manager resolver direct projection: passed; plugin runtime gaps empty, unclassified public modules 2, unclassified public uses 0, unclassified hotspots 0, root surface migration debt 4, large-file migration debt 5)
  - cargo validation for M5 manager resolver direct projection (2026-06-11: deferred because active shared Cargo/rustc lanes were running; no new Cargo command was started and no Cargo pass/fail is claimed)
  - python -m unittest tools.tests.test_frameworks_03_contract_feature_boundary
  - cargo +nightly check -p zircon_runtime --lib --no-default-features --features ai-contracts --locked --offline --jobs 1
  - cargo +nightly check -p zircon_runtime --lib --no-default-features --features sound-contracts --locked --offline --jobs 1
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

Optional manager contracts are exposed only when their owning contract feature is enabled. Frameworks 03 now applies that hard boundary to `ai-contracts`, `net-contracts`, and `sound-contracts`: each trait re-export, service name, typed holder, and resolver is gated together at declaration/assembly points. There is no placeholder handle, fallback resolver, or compatibility re-export when a contract is absent.

## Performance Contract

Manager service names are static constants, so callers do not allocate or format registry names on the hot path. `CoreHandle::resolve_manager(...)` still locks the runtime service table and returns an `Arc` clone of the cached service holder. That is acceptable at setup, module activation, and coarse workflow boundaries, but repeated per-entity or per-frame inner-loop lookup should cache the returned `Arc<dyn ManagerTrait>` in the caller's runtime state.

The resolver layer intentionally returns trait-object `Arc`s instead of concrete plugin managers. This keeps plugin implementations replaceable and prevents app/editor code from depending on plugin-private manager structs.

M5 follow-up: resolver helpers now project the typed holder into its trait-object `Arc` through a direct `let holder = core.resolve_manager::<$holder>(...)?; Ok(holder.shared())` branch inside the macro. This keeps the existing typed downcast/error path in `CoreHandle::resolve_manager(...)`, while avoiding a per-resolver `.map(|holder| holder.shared())` closure adapter after the service holder has been resolved.

## Current Coverage

`core::manager::tests` locks the canonical service-name strings for resource, input, config, event, rendering, render-framework, level, AI, net, physics, animation, and sound managers. AI, Net, and Sound assertions compile only with their owning contract features, matching the public manager surface.

`tests/extensions/manager_handles.rs` protects the cross-crate contract for first-party manager-backed plugins. It checks that plugin modules keep framework-backed default managers and that `core::manager` exports the matching handles, resolver helpers, and canonical service names. The guard now includes AI alongside physics, animation, net, and sound.

## Follow-Up

M5 performance work should inspect call sites that invoke resolver helpers from repeated update loops. Those call sites should resolve once during setup or activation, store the trait-object `Arc`, and use the cached manager handle during tick/render/extract work.
