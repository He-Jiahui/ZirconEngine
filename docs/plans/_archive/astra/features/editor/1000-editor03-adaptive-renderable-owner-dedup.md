---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/03/2026-08-27-adaptive-renderable-owner-dedup.md
related_records:
  - docs/plans/astra/features/editor/997-editor03-scene-history-selection-completion-list.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/scene/viewport/pointer/candidates/renderable_candidates.rs
  - zircon_editor/src/scene/viewport/pointer/candidates/mod.rs
tests:
  - zircon_editor/src/scene/viewport/pointer/candidates/renderable_candidates.rs
---

# Editor1000 · adaptive renderable-owner dedup

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor03 viewport selection candidates | Retain the adjacent-owner fast path for grouped render extracts; build a `HashSet` only after the first decreasing owner ID and reject later non-adjacent duplicates while preserving first-candidate geometry. | Grouped and interleaved regressions preserve checksum `6240534528` from the source plan, with grouped allocations unchanged and interleaved rows `65,536→8,192`, bytes `9,961,600→1,851,536`, P50 cycles `80,198,144→7,977,654`, and P95 `107,264,100→8,876,445`. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/scene/viewport/pointer/candidates/renderable_candidates.rs` | `ADD2C8D28A6BB31911974636484EF8C695250D37C9FB838650E7E3D21D0C1A8F` |
| `zircon_editor/src/scene/viewport/pointer/candidates/mod.rs` | `2DD62E287F2047991785BBFB5F253D4FFB18E4F3D1AA1D75FB060D264D548D1B` |

## Validation handoff

The grouped-owner no-allocation regression, interleaved dedup regression, and
Release model are included in the grouped Editor package validation. Managed
Cargo and product picking evidence remain pending.
