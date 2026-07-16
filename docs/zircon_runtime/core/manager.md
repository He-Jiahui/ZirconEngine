---
related_code:
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/core/manager/service.rs
  - zircon_runtime/src/core/manager/service_names.rs
  - zircon_runtime/src/core/manager/tests.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/activation/batch.rs
  - zircon_runtime/src/core/runtime/handle/activation/service_lifecycle.rs
  - zircon_runtime/src/core/runtime/state/service_entry.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/reactivation.rs
  - zircon_runtime/src/core/runtime/tests/activation/structure/reactivation.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/access.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/surface.rs
  - zircon_runtime/src/tests/extensions/manager_handles.rs
  - zircon_runtime/src/core/framework/ai/manager.rs
  - zircon_plugins/ai/runtime/src/module.rs
  - zircon_plugins/ai/runtime/src/manager.rs
  - zircon_runtime/src/core/framework/sound/manager.rs
  - zircon_plugins/sound/runtime/src/module.rs
implementation_files:
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/core/manager/service.rs
  - zircon_runtime/src/core/manager/service_names.rs
  - zircon_runtime/src/core/manager/tests.rs
  - zircon_runtime/src/tests/extensions/manager_handles.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/activation/batch.rs
  - zircon_runtime/src/core/runtime/handle/activation/service_lifecycle.rs
  - zircon_runtime/src/core/runtime/state/service_entry.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/reactivation.rs
  - zircon_runtime/src/core/runtime/tests/activation/structure/reactivation.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/surface.rs
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - docs/engine-architecture/runtime-interface-convergence.md
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_runtime/runtime/15/fixed-2026-07-14-manager-service-reactivation-lifecycle.md
  - docs/plans/zircon_runtime/runtime/15/fixed-2026-07-14-ui-text-project-asset-manager-access-consumer-drift.md
  - docs/plans/zircon_runtime/runtime/15/fixed-2026-07-14-ui-text-manager-access-cross-frame-retention.md
  - docs/plans/zircon_runtime/frameworks/05/2026-07-14-ui-text-project-asset-manager-access-consumer-drift.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
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
  - python -m unittest tools.tests.test_frameworks_05_layer_direction.Frameworks05LayerDirectionTests.test_manager_services_use_versioned_handles_without_legacy_arc_holders -v
  - cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib core::runtime::tests::activation::behavior::reactivation --locked
  - cargo test -p zircon_runtime --lib core::runtime::tests::resolution::behavior::deactivation_invalidates_registered_manager_identity_before_reactivation --locked -- --exact --nocapture
  - rustc --edition 2021 --test zircon_runtime/src/core/runtime/tests/activation/structure/mod.rs; reactivation::reactivation_lifecycle_is_complete_and_folder_backed
  - managed Windows default-feature cargo build -p zircon_runtime --locked (job 9dac70c034fb4aa18155d370f77073e1; passed)
  - python -m unittest tools.tests.test_frameworks_05_manager_access_lifetime tools.tests.test_frameworks_05_layer_direction -v (24/24 passed)
  - managed Windows default-feature cargo build -p zircon_runtime --locked (job c2db4e7bfe0647678e6334648b6df811; passed after cross-frame lifetime correction)
doc_type: module-detail
---

# Core Manager Spine

`zircon_runtime::core::manager` is the runtime-internal access layer for stable manager service names, versioned manager identities, registration adapters, and use-point resolution. It sits between `core::runtime` service registration and higher-level runtime/plugin consumers. This keeps manager access descriptor-driven without reviving the old standalone manager crate shape or retaining a manager implementation across unload/reload generations.

## Boundary

The module owns four pieces of public contract:

- `service_names.rs` defines canonical `&'static str` manager service names.
- `service.rs` defines `ManagerServiceHandle<T> { index, generation, service }`, `RegisteredManagerService<T>`, and the generic use-point resolver contract.
- `resolver.rs` maps each neutral manager trait to a canonical service name and produces a typed versioned handle.
- `mod.rs` exposes only the curated handle, resolver, registration adapter, and canonical service names upper layers need; neutral traits remain public solely from `core/framework`.

Concrete behavior stays outside this module. For example, the Scene module owns `DefaultLevelManager`, the neutral trait has its single owner at `core::framework::scene::LevelManager`, and registration wraps the implementation in `RegisteredManagerService<dyn LevelManager>`. `core::manager` owns the identity vocabulary and resolver, not a duplicate trait alias or concrete implementation.

Optional manager accessors are exposed only when their owning contract feature is enabled. Frameworks 03 applies that hard boundary to `ai-contracts`, `net-contracts`, and `sound-contracts`: each neutral trait owner, service name, typed handle accessor, and registration path is gated together at declaration/assembly points. There is no placeholder handle, fallback resolver, compatibility re-export, or retired Arc-holder type when a contract is absent.

## Performance Contract

Manager service names are static constants. Long-lived cross-domain state stores only the lightweight typed handle; it does not retain `Arc<dyn ManagerTrait>` or `Arc<ConcreteManager>`. At the use point, `resolve_manager_service(...)` validates the handle's index and generation against the registered slot, resolves the registration adapter, and returns a transient trait-object `Arc` for the immediate operation.

Module deactivation invalidates the slot generation. A handle captured before deactivation therefore fails with `CoreError::StaleServiceHandle` after unload and remains invalid after reactivation; callers must request the current handle. This prevents stale manager retention while keeping plugin implementations replaceable and preventing App/Editor code from depending on plugin-private manager structs.

Reactivation is a module-owned transition over the complete immutable `ModuleEntry::service_names` list, not only its Immediate startup subset. Before lifecycle build begins, every slot owned by an `Unloaded` module is validated as empty and restored to `Registered` without changing its index or generation. Immediate services are then resolved by the normal startup path; lazy services remain `Registered` with no instance until use-point resolution. `activate_registered_modules` applies the same validation and transition atomically to every reactivated module before any batch build hook runs.

If reactivation fails, the module returns to its previous `Unloaded` state and all of its service slots return to `Unloaded`. Discarding a newly constructed instance advances the slot generation again, so an identity that observed that instance cannot survive rollback. A lazy slot that was never constructed keeps the generation established by the preceding unload; availability changes alone do not manufacture a new service generation. Both prepare and rollback notify service-resolution waiters after releasing the registry lock.

Use-point resolution is appropriate at event, tick, or orchestration boundaries. A hot inner loop should resolve once at the start of that bounded operation and reuse the returned `Arc` only within the operation; it must not persist that `Arc` as cross-frame manager state.

The screen-space UI renderer and its long-lived text system apply the same rule at two real use points. Construction resolves `ProjectAssetManagerAccess` only while loading the initial default-font asset; each frame's text `prepare` resolves again for that frame's font and glyph operations. The text system stores the versioned access object between frames, never the concrete `Arc<ProjectAssetManager>`. Unavailable or stale identity errors map to `GraphicsError::Asset` and propagate through both ordinary scene rendering and render-graph execution. No Arc adapter, named resolver, fallback owner, or silent text skip survives this cut.

## Current Coverage

`core::manager::tests` locks the canonical service-name strings and the versioned handle assembly surface. AI, Net, and Sound assertions compile only with their owning contract features, matching the public manager surface.

`tests/extensions/manager_handles.rs` protects the cross-crate contract for first-party manager-backed plugins and rejects the retired named Arc-holder/resolver vocabulary. The Frameworks05 layer-direction guard additionally verifies the index/generation/service shape and rejects long-lived manager trait-object storage in selected production owners.

`core::runtime::tests::activation::behavior::reactivation` covers single-module and batch reactivation with Immediate plus lazy managers, stable indices, unload generation invalidation, lazy construction, and finish-error rollback. The activation structure guard requires the transition logic to remain in `handle/activation/service_lifecycle.rs`, keeps the orchestration/test owners within file budgets, and rejects resolver exceptions or compatibility vocabulary in the lifecycle owner.

The Frameworks05 current-source lifetime guard locks access storage, construction-time and per-frame resolution, absence of concrete manager storage, and both renderer error exits. The focused Frameworks05 suite passed 24/24, and managed Windows job `c2db4e7bfe0647678e6334648b6df811` passed the default-feature Runtime compile gate after the deeper lifetime correction. The earlier constructor-only job remains historical evidence and is not reused as acceptance evidence for the review finding.

## Hard-Cut Invariants

- Neutral manager traits have one owner under `core/framework`; `core/manager` does not recreate or alias them.
- Cross-domain stored state uses `ManagerServiceHandle<T>`, never the retired `*ManagerHandle` Arc-holder structs or `resolve_*_manager` helpers.
- Registration uses `RegisteredManagerService<T>` and resolution validates index plus generation before projecting the trait object.
- Module reactivation transitions the complete service owner list; restoring only startup services or weakening `Unloaded` availability is forbidden.
- Failed reactivation restores the module and all service slots to `Unloaded`; only discarded instances advance generation during rollback.
- No compatibility module, duplicate public truth, or fallback to concrete manager resolution is retained.
