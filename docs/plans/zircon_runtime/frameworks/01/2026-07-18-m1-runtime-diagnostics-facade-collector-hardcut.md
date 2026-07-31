---
related_code:
  - zircon_runtime/src/core/runtime/diagnostics/mod.rs
  - zircon_runtime/src/core/runtime/diagnostics/devtools.rs
  - zircon_runtime/src/runtime_diagnostics/mod.rs
  - zircon_runtime/src/runtime_diagnostics/collect.rs
  - zircon_runtime/src/runtime_diagnostics/physics_collection_enabled.rs
  - zircon_runtime/src/runtime_diagnostics/physics_collection_disabled.rs
implementation_files:
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/core/runtime/diagnostics/mod.rs
  - zircon_runtime/src/core/runtime/diagnostics/devtools.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store.rs
  - zircon_runtime/src/runtime_diagnostics/mod.rs
  - zircon_runtime/src/runtime_diagnostics/collect.rs
  - zircon_runtime/src/runtime_diagnostics/physics_collection_enabled.rs
  - zircon_runtime/src/runtime_diagnostics/physics_collection_disabled.rs
  - zircon_runtime/src/dynamic_api/session/diagnostics.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs
  - zircon_runtime/src/tests/runtime_diagnostics/motion_vector.rs
  - zircon_editor/src/ui/host/editor_manager_runtime_diagnostics.rs
  - tools/tests/test_frameworks_01_runtime_diagnostics_boundary.py
plan_sources:
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - tools.tests.test_frameworks_01_runtime_diagnostics_boundary
  - runtime_diagnostics_reports_missing_runtime_contracts_without_panicking
  - runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins
  - devtools_snapshot_lists_modules_services_and_builtin_catalog
  - backend_contract_projects_to_stable_neutral_diagnostics
doc_type: milestone-detail
---

# Frameworks01 M1 Runtime Diagnostics Facade Collector Hard Cut

Plan: `docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
Milestone: M1 owner-DAG prerequisite
Status: implemented_static_passed_foreign_doc_and_cargo_pending
Date: 2026-07-18

## Delivered

| Slice | Status | Evidence |
|---|---|---|
| Facade collector owner | implemented | top-level `runtime_diagnostics` uniquely owns manager-resolving collection. |
| Core diagnostics boundary | implemented | core diagnostics contains DTO/store/profiling/render-stat/projector code and zero `core::manager` imports. |
| Physics profile adapters | implemented | enabled/disabled collectors moved with the facade owner. |
| Devtools split | implemented | core owns pure `project_runtime_devtools_snapshot`; facade owns collection orchestration. |
| Public hard cut | implemented | Runtime and Editor callers use `runtime_diagnostics::collect_*`; old core exports are deleted. |
| Static guard | passed | focused Frameworks01 guard plus the full Frameworks03 contract boundary suite are 23/23 GREEN. |
| Independent review | passed | final current-source review reports P0/P1/P2 = 0/0/0. |
| Live documentation | pending foreign owner | all owned live docs are migrated; `core/framework/render/material.md` remains Shader06-owned. |
| Managed Cargo | pending | current-source Runtime/Editor behavior and profile compilation remain FIFO pending. |

## Architecture Decision

`core/manager` is a facade access layer because it must see registered domain services. A future
`zr_diagnostics` crate may depend on kernel/contracts, but it must not depend back on that facade.
Therefore diagnostic DTOs, stores and pure projections remain under core diagnostics while service
resolution moves to the top-level runtime facade.

The old collector modules and `core::diagnostics::collect_*` exports are physically removed. This is
not an alias migration: every caller moved to the new canonical owner, and no forwarding wrapper or
duplicate collector survives.

## Remaining Scope

This closes the runtime-diagnostics-to-manager prerequisite edge only. It does not claim physical
`zr_diagnostics` extraction, the other Frameworks01 M1 owner-DAG prerequisites, or package acceptance.
The remaining stale collector path in `docs/zircon_runtime/core/framework/render/material.md` is
owned by active Session `shader06-current-source-closeout-audit-20260716`; this Session records the
handoff and does not overwrite that foreign worktree change.
