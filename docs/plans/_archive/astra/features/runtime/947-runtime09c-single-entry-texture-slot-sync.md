---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09c/2026-08-27-single-entry-texture-slot-sync.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/asset/assets/material/material_asset/value_sync.rs
tests:
  - zircon_runtime/src/asset/assets/material/material_asset/value_sync.rs
---

# Runtime947 · single-entry texture-slot synchronization

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime09c material texture-slot synchronization | The occupied texture-slot path now takes one `BTreeMap::entry` traversal, reads retained fallback/transform/UV metadata from the occupied entry, and replaces the value in place. The vacant path inserts directly through the same entry; `None` removal semantics and metadata preservation remain unchanged. | The two in-source occupied/vacant behavior tests are present, and the standalone optimized model compares complete maps after every sample. It reduces ordered-map traversals from `262,144` to `65,536` per sample (`75.000%`), with P50 `68,817,800ns→51,006,400ns` (`25.882%`) and P95 `91,776,300ns→76,528,200ns` (`16.614%`), clearing the plan's `15%` P50/P95 gates. Managed package/Release allocation and product evidence remain pending. | implemented_pending_validation |

## Deterministic boundary

The optimized path retains the replacement texture reference, prior fallback,
transform, and UV channel for occupied slots. A vacant slot is created with the
same default metadata as before. A `None` texture still clears the reference and
removes the slot only when no retained metadata remains.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/assets/material/material_asset/value_sync.rs` | `A4480C91E651397586700A55A5F5735A5D7FC79B35B2DA0AC58BBC887A02C305` |

## Validation handoff

The standalone Rust model was compiled with `rustc --edition 2021
-C opt-level=3`; its source contracts, complete-map parity, 75% traversal
reduction, and 15% P50/P95 reduction gates pass. The focused Cargo tests remain
part of the grouped Runtime package validation rather than a per-task launch.
The fresh grouped current-source Runtime/Editor managed wave was submitted with
Runtime execution PTY `98618` and Editor execution PTY `86711`; this record does
not infer a compiler or test result from those asynchronous launches. Managed
Release allocation/P50/P95 and renderer-product gates remain pending.
