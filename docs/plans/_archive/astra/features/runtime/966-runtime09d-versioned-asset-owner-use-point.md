---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09d/2026-09-10-versioned-asset-owner-use-point.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/runtime/972-runtime09d-to-09h1-completion-list.md
implementation_files:
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency/semantic_executor/owner.rs
tests:
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency/semantic_executor/owner.rs
---

# Runtime966 · versioned asset-owner use-point resolution

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime09D semantic residency owner | `RenderAssetSemanticExecutorOwner` retains `ProjectAssetManagerAccess` instead of a strong manager reference, captures the generation/loader snapshot at construction, and resolves the current manager at admission and maintenance use points. Resolution failure closes the executor with the typed owner error; generation mismatch keeps the existing superseded path. | The focused Frameworks01/05 and UI access-boundary contracts pass `31/31`; the bounded Runtime/Editor source-contract batch passes `70/70`; scoped Rustfmt and diff checks pass. This slice does not claim streaming or product frame qualification. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/graphics/scene/resources/render_asset_residency/semantic_executor/owner.rs` | `9C17B5B712328F1C9616F4487EBE23248E8583EE1546C4B4DDFB79DBF3F900E0` |

## Validation handoff

The source is included in the grouped Runtime/Editor lib-test validation wave.
Managed Cargo/WGPU execution and release P50/P95/P99 evidence remain pending;
no streaming, device, or product-performance result is inferred here.
