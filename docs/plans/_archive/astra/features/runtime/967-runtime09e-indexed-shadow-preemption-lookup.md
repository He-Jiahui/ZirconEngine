---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09e/2026-08-26-indexed-shadow-preemption-lookup.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/runtime/972-runtime09d-to-09h1-completion-list.md
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator/tests.rs
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator/tests.rs
---

# Runtime967 · indexed shadow preemption lookup

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime09E shadow-atlas contention | Planned shadow slots are sorted before contention, one `planned_by_key` index is shared by incumbent lookup and retained projection, and challenger traversal stops at the first priority below the incumbent threshold. Tier and self-pair semantics remain unchanged. | Local source contracts pass `3/3`; the model reduces combined incumbent/challenger work from `25,167,872` visits/comparisons to `8,192` probes/visits (`99.9675%`). Two release preflights reduce P95 by `99.3638%` and `98.7835%`; the terminal marker requires at least 95%. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator.rs` | `E15BC3BC36901631A7F7DC8A9A2A248EEF476B5716598142F8D4C866410A4753` |
| `zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator/tests.rs` | `CF49AC078886E7AF0790ADDE047B1AA62ABAE098810CF51598124C7559A376C6` |

## Validation handoff

The filtered allocator tests and ignored release benchmark remain part of the
grouped Runtime package validation. The `RUNTIME09E_SHADOW_PREEMPTION_BENCH_V1`
terminal receipt, managed compilation, and product shadow-matrix evidence remain
pending.
