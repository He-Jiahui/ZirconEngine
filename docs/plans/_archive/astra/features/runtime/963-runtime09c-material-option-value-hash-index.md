---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09c/2026-08-26-material-option-value-hash-index.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/runtime/964-runtime09c-material-index-completion-list.md
implementation_files:
  - zircon_runtime/src/core/framework/render/shader/material_property_layout.rs
  - zircon_runtime/src/core/framework/render/shader/material_property_layout/option_lookup_tests.rs
tests:
  - zircon_runtime/src/core/framework/render/shader/material_property_layout/option_lookup_tests.rs
---

# Runtime963 · material option value hash index

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime09c material option projection | Large option tables now build one bounded borrowed `HashMap<&str, &MaterialOptionRef>` per value-application call, eliminating repeated linear option scans. Tables below 16 options or with fewer than two supplied values retain the allocation-free linear fast path. Ordered `BTreeMap` value folding, first duplicate option ownership, default bits, unknown values, invalid types, serialization, and bit packing remain unchanged. | The focused tests cover ordered overwrite parity against a legacy scan, duplicate-name ownership, unknown values, the small-table threshold, and the borrowed hash-index source contract. The ignored Release model resolves 512 options across 16 calls and emits `RUNTIME09C_MATERIAL_OPTION_VALUE_HASH_INDEX_BENCH_V1`; acceptance requires indexed P95 at least 80% below repeated linear scans. | implemented_pending_validation |

## Deterministic work model

The plan workload changes 2,101,248 linear option-name comparisons to 8,192
borrowed-index insertions and 8,192 hash lookups, with zero owned key
allocations. Exact managed Windows P50/P95 values remain coordinator-owned.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/core/framework/render/shader/material_property_layout.rs` | `B3059B36C7944EB9227E67E017AE5BEF995B5213D0D49C62482D81B045BE8DCF` |
| `zircon_runtime/src/core/framework/render/shader/material_property_layout/option_lookup_tests.rs` | `04393BE75753081CDA0ADC2861F0589FC9C31D65A9FA2AAD1D978EEC69A02DC7` |

## Validation handoff

The source and test files are included in the grouped Runtime graphics/lib-test
validation wave with the shading-token index and material-property schema
rescan. No per-task managed invocation was started. This record remains pending
until the managed Release marker proves the 80% P95 gate and the package/product
acceptance lanes provide their authoritative receipts.

The broader Runtime09c parent work still owns shader compilation, material
variants, PSO lifetime, persistence, and product GPU evidence; this record only
closes option-value resolution.
