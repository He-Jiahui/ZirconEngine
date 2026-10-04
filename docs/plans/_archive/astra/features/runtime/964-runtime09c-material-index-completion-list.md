---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09c/2026-08-24-shader-prewarm-managed-env-test-contract.md
  - docs/plans/optimize/zircon_runtime/09c/2026-08-26-material-option-value-hash-index.md
  - docs/plans/optimize/zircon_runtime/09c/2026-08-26-material-property-schema-name-index.md
  - docs/plans/optimize/zircon_runtime/09c/2026-08-26-shading-model-token-hash-index.md
  - docs/plans/optimize/zircon_runtime/09c/2026-08-27-single-entry-texture-slot-sync.md
related_records:
  - docs/plans/astra/features/runtime/946-runtime09c-material-property-schema-rescan.md
  - docs/plans/astra/features/runtime/947-runtime09c-single-entry-texture-slot-sync.md
  - docs/plans/astra/features/runtime/962-runtime09c-shading-model-token-hash-index.md
  - docs/plans/astra/features/runtime/963-runtime09c-material-option-value-hash-index.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
---

# Runtime09c · material-index completion list

This list records the Runtime09c material lookup and synchronization slices that
are implemented in the current source. It does not promote the parent shader
prewarm, pipeline, persistence, allocator, or renderer-product gates.

| Plan slice | Astra record | Current status | Acceptance boundary |
| --- | --- | --- | --- |
| Material property schema-name index | [Runtime946](946-runtime09c-material-property-schema-rescan.md) | implemented_pending_validation | Managed grouped Release schema-rescan marker and product gates remain pending. |
| Single-entry texture-slot synchronization | [Runtime947](947-runtime09c-single-entry-texture-slot-sync.md) | implemented_pending_validation | Managed package/Release allocation and renderer-product evidence remain pending. |
| Shading-model token hash index | [Runtime962](962-runtime09c-shading-model-token-hash-index.md) | implemented_pending_validation | `RUNTIME09C_SHADING_TOKEN_HASH_INDEX_BENCH_V1` must clear the 30% P95 reduction gate. |
| Material option value hash index | [Runtime963](963-runtime09c-material-option-value-hash-index.md) | implemented_pending_validation | `RUNTIME09C_MATERIAL_OPTION_VALUE_HASH_INDEX_BENCH_V1` must clear the 80% P95 reduction gate. |

The three hash/index slices are intentionally validated in one grouped Runtime
invocation together with the schema-rescan contracts; the Editor package is
submitted in the same validation wave for current-source compile coverage. The
tooling migration remains out of scope.
