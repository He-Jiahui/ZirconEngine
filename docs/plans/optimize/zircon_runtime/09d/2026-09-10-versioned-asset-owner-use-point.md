---
title: Runtime09D Versioned Asset Owner Use-Point Resolution
category: zircon_runtime
report_id: Runtime09D-versioned-asset-owner-use-point-2026-09-10
date: 2026-09-10
session_id: root-astra-optimize-20260909
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09D Versioned Asset Owner Use-Point Resolution

## Scope

This slice closes the long-lived concrete-manager violation in the semantic residency
executor owner. It does not claim semantic artifact streaming, product frame scheduling,
RHI upload integration, fence retirement, or the 09D M1-M11 dynamic acceptance gates.

## Implementation

- `RenderAssetSemanticExecutorOwner` now retains `ProjectAssetManagerAccess` rather than an
  `Arc<ProjectAssetManager>`.
- Construction resolves the access handle once to capture the project-generation token and
  loader snapshot. Admission and maintenance resolve the current manager again at their use
  points, so manager replacement and runtime teardown cannot leave a stale strong owner.
- Resolution failure closes the executor before returning the typed owner error; generation
  mismatch keeps the existing superseded close path.
- The owner remains independent of WGPU ownership and continues to return neutral upload
  plans to the RHI owner.

## Regression and local evidence

- `test_asset_manager_consumers_use_versioned_handles_at_use_points` now rejects production
  `Arc<ProjectAssetManager>` storage and verifies the versioned access contract.
- Focused Frameworks01/05 plus the updated UI access-boundary contract: `31/31` tests passed.
- The UI text contract now tracks the `new_with_font_collection` constructor,
  `GraphicsError` preparation boundary, prepared-upload return type, and borrowed
  renderer call sites after the retained-frame refactor.
- Scoped `rustfmt --edition 2021 --check` passed for the owner and related test source.
- Scoped `git diff --check` passed (the repository reports existing CRLF warnings only).
- The bounded Runtime/Editor source-contract batch covering owner resolution, retained UI
  image/text paths, and Editor12 plugin-manager boundaries passed `70/70`; this is contract and
  operation-count evidence only.
- Managed Windows Cargo/WGPU execution and release p50/p95/p99 measurements remain pending;
  no streaming or device-performance qualification is inferred from source checks.

## Remaining parent work

09D still owns semantic manifest/load-plan execution, product completion/retirement driving,
full resource demand, budget accounting, device lifecycle, and the cold/warm/traverse/
teleport/reload/device-loss/OOM/fault/soak matrix.
