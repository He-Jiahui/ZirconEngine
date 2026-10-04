---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09c/2026-08-26-shading-model-token-hash-index.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/runtime/964-runtime09c-material-index-completion-list.md
implementation_files:
  - zircon_runtime/src/graphics/material/shading_models/registry.rs
  - zircon_runtime/src/graphics/material/shading_models/registry/hash_token_tests.rs
tests:
  - zircon_runtime/src/graphics/material/shading_models/registry/hash_token_tests.rs
---

# Runtime962 · shading-model token hash index

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime09c shading-model resolution | The registry keeps descriptor ownership and deterministic ID iteration in the existing `BTreeMap`, while normalized token resolution uses one `HashMap<String, ShadingModelId>`. Common tokens probe the borrowed trimmed key before allocating a lower-case fallback; registration still trims, normalizes, rejects duplicate IDs/tokens, validates plugin ranges and G-buffer channels, and preserves descriptor order. | The source contains the bounded hash-owned token index and borrowed common-token probe. The focused tests cover case-normalized custom lookup, duplicate-token rejection, duplicate-ID rejection, unchanged ordered descriptor IDs, plugin-range validation, and channel validation. The ignored Release model runs 17 alternating pairs over 256 long-prefix tokens and 4,096 hits; its acceptance gate is a HashMap P95 at least 30% below the ordered legacy path. | implemented_pending_validation |

## Deterministic work model

The plan workload changes 4,096 ordered token-index probes to 4,096 hash-index
probes without changing descriptor-table ordering or introducing direct-hit token
allocations. The release marker is
`RUNTIME09C_SHADING_TOKEN_HASH_INDEX_BENCH_V1`; exact managed Windows P50/P95
values remain coordinator-owned.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/graphics/material/shading_models/registry.rs` | `EB3BF8543B9FB653A45B74012DEB064908230A2494581D93CD03682730814463` |
| `zircon_runtime/src/graphics/material/shading_models/registry/hash_token_tests.rs` | `8B2B5275DBF16D7C93F2B571B794E523C05ABCB20D6E3CA479676C2934719F1C` |

## Validation handoff

The source and test files are included in the grouped Runtime graphics/lib-test
validation wave with the material-option index and material-property schema
rescan. No per-task managed invocation was started. This record remains pending
until the managed Release marker proves the 30% P95 gate and the package/product
acceptance lanes provide their authoritative receipts.

The broader Runtime09c parent work still owns shader compilation, pipeline
variants, PSO lifetime, persistence, and product GPU evidence; this record only
closes token-index resolution.
