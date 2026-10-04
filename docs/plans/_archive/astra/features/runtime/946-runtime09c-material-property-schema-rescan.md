---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09c/2026-08-26-material-property-schema-name-index.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/asset/assets/material/property_values.rs
  - zircon_runtime/src/asset/assets/material/property_values/schema_index_tests.rs
tests:
  - zircon_runtime/src/asset/assets/material/property_values/schema_index_tests.rs
---

# Runtime946 · material property schema rescan elision

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime09c material projection | The declared shader-schema pass remains the single typed-property owner; the material-only fallback now uses one `BTreeMap::entry` lookup to preserve an already-projected declared string without a second string allocation, and no longer rescans the full shader schema for every override. Invalid typed values, unknown string fallback values, sorted output, and unknown non-string filtering remain unchanged. | The current-source graphics-enabled Release batch passes `10/10` (7 ordinary contracts plus 3 ignored markers). Its exact grouped markers report shading ordered P95 `6,399,200ns` versus hash P95 `1,777,400ns` (`72.22%` reduction), option scan P95 `20,344,000ns` versus hash P95 `1,143,100ns`, and schema baseline P95 `32,794,200ns` versus optimized P95 `4,604,000ns` (`85.96%` reduction). Managed Release, allocator, and renderer-product gates remain pending. | implemented_pending_validation |

## Deterministic boundary

For the 4,096-property / 4,096-override fixture, the legacy fallback performs
`16,777,216` schema-name comparisons while the optimized fallback performs zero
schema comparisons and retains only the bounded output-map membership probes.
No temporary schema index or ordering change is introduced.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/assets/material/property_values.rs` | `6CA3762196D86B6EAF4AA5301E88DBDD620A010CB5CD04FDF2D27340EB226CB4` |
| `zircon_runtime/src/asset/assets/material/property_values/schema_index_tests.rs` | `CB88D5A2EFA5EFD873E3CE2B832E5A76568745C3681A93D1ED24CA8340A5017B` |

## Validation handoff

The corrected material/option subset completed in
`F:\\codex-targets\\zircon-engine\\runtime09c-batch-repair-20260926`: `7/7`
tests passed, including the repaired marker at `86.57%` P95 reduction
(`70,448,100ns` legacy versus `9,462,300ns` optimized). The earlier failed
marker (`71.45%`) and the first repaired run (`86.24%`) are retained as RED/GREEN
history. The graphics-enabled grouped Release target completed the full
Runtime09c selector in `F:\\codex-targets\\zircon-engine\\runtime09c-graphics-batch-20260926`
(local PTY `86670`): `10/10` tests passed, with all three ignored performance
markers included. A direct rerun of the produced test binary preserved the
same grouped boundary and provides the unwrapped marker receipt below.
This record stays pending until the managed batch supplies authoritative
Release allocation/P50/P95 and renderer-product evidence.

The source contract was then made rustfmt-stable by matching the entry and lazy
insert operations as separate structural fragments rather than requiring one
source line. The replacement material/option Release batch used the corrected
contract (local PTY `80387`) and completed `7/7`; its result is now the current
local receipt, while managed acceptance remains pending.

The first graphics-enabled batch also exposed two compile errors in an existing
mesh-pipeline cache unit test that constructed `ShaderSourceValidationKey`
through private fields. The test now uses its existing `new` constructor; the
graphics-enabled Runtime09c batch was restarted with that test-only repair in
`F:\\codex-targets\\zircon-engine\\runtime09c-graphics-batch-20260926` (local
PTY `86670`).

The completed grouped binary receipt is:

- `RUNTIME09C_SHADING_TOKEN_HASH_INDEX_BENCH_V1`: ordered P95 `6,399,200ns`,
  hash P95 `1,777,400ns`, `descriptor_order_changes=0`, and
  `direct_hit_allocations=0` (`72.22%` P95 reduction).
- `RUNTIME09C_MATERIAL_OPTION_VALUE_HASH_INDEX_BENCH_V1`: scan P95
  `20,344,000ns`, hash P95 `1,143,100ns`, `comparisons_before=2,101,248`,
  `comparisons_after=0`, and `owned_key_allocations=0`.
- `RUNTIME09C_MATERIAL_PROPERTY_SCHEMA_RESCAN_BENCH_V1`: baseline P95
  `32,794,200ns`, optimized P95 `4,604,000ns`, `85.96%` reduction,
  `schema_comparisons_before=16,777,216`, and `schema_comparisons_after=0`.

The direct grouped rerun exited successfully in `0.54s` with `10 passed; 0
failed; 0 ignored; 0 measured` after the three ignored markers were explicitly
included. The `85.96%` value is timing-noise variation from the earlier local
`86.57%` core-min and `86.28%` graphics Cargo measurements; both remain above
the unchanged `80%` threshold. These are local Release receipts only; managed
allocator and renderer-product acceptance remains pending.

After this local batch and the current-source Editor library compile returned,
the grouped managed Windows package validation was submitted together for
Runtime and Editor (`validate-matrix.ps1 -Package zircon_runtime -LibTests
-TestThreads 1`, execution PTY `24063`; `-Package zircon_editor`, execution PTY
`44882`). The submissions are intentionally asynchronous and this record does
not infer a compiler or test result from their launch; the managed Release,
allocator, and renderer-product gates remain pending.
