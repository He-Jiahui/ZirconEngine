---
related_code:
  - zircon_runtime/src/core/runtime/diagnostics/mod.rs
  - zircon_runtime/src/runtime_diagnostics/mod.rs
  - zircon_runtime/src/runtime_diagnostics/collect.rs
  - zircon_runtime/src/core/runtime/diagnostics/physics.rs
  - zircon_runtime/src/core/runtime/diagnostics/physics_backend.rs
  - zircon_runtime/src/runtime_diagnostics/physics_collection_enabled.rs
  - zircon_runtime/src/runtime_diagnostics/physics_collection_disabled.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
implementation_files:
  - zircon_runtime/src/core/runtime/diagnostics/physics_backend.rs
  - zircon_runtime/src/runtime_diagnostics/collect.rs
  - zircon_runtime/src/runtime_diagnostics/physics_collection_enabled.rs
  - zircon_runtime/src/runtime_diagnostics/physics_collection_disabled.rs
plan_sources:
  - user: 2026-07-10 frameworks foundation architecture hard cutover
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/runtime_diagnostics/physics_collection_enabled.rs
  - tools/tests/test_frameworks_03_contract_feature_boundary.py
  - tests/acceptance/frameworks-03-physics-contract-feature-boundary.md
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs
doc_type: module-detail
---

# Runtime Diagnostics

## Purpose

Runtime diagnostics exposes read-only, tooling-facing snapshots without making Editor or core diagnostic DTOs depend on optional subsystem contracts or the manager facade. Core diagnostics owns DTOs, stores, profiling, and pure projection. The top-level `runtime_diagnostics` facade owns manager resolution and collection.

## Physics Projection

`RuntimePhysicsBackendDiagnostics` is the neutral projection of the optional Physics backend status. It preserves requested and active backend names, state, detail, simulation mode, and feature-gate information using always-on standard-library field types. It deliberately does not re-export or embed `PhysicsBackendStatus`, `PhysicsBackendState`, or `PhysicsSimulationMode`.

The feature-enabled facade collector resolves `PhysicsManager`, reads its settings and backend status, and converts contract enums to stable snake-case diagnostic strings. The feature-disabled collector reports that Physics contracts were not compiled. Both physical owners live under `runtime_diagnostics/`; `core/runtime/diagnostics` never imports `core::manager`.

This split keeps diagnostic consumers source-compatible across Runtime profiles without pretending that an unavailable Physics contract is a live backend. The Editor renders the neutral fields and does not import Physics framework types.

## Current Integration State

The neutral DTO remains mounted from `core/runtime/diagnostics/mod.rs`. The facade `runtime_diagnostics/mod.rs` mounts the enabled/disabled collectors and exports the only `collect_runtime_diagnostics` and `collect_runtime_devtools_snapshot` functions. All Runtime and Editor callers use this path; the old `core::diagnostics::collect_*` exports and physical collector files are deleted, with no compatibility re-export.

The Runtime projection intentionally emits stable snake-case strings such as `ready`. The separate Editor UI 08 handoff `runtime-diagnostics-physics-state-format` concerns presentation capitalization and accidental `Debug` quoting in a pane builder; it does not reopen the Runtime DTO or optional Physics dependency boundary.

## Validation

The historical Frameworks 03 boundary and Physics behavior gates remain recorded by their owner. The 2026-07-18 Frameworks01 hard-cut guard passes 2/2 and proves the new physical/public boundary. Current-source managed Cargo remains pending and is not replaced by the static result.
