---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/82/2026-09-26-grapheme-aligned-edit-replace.md
  - docs/plans/optimize/zircon_runtime/82-runtime-text-editing-document-selection-caret-hit-test-ime-composition-clipboard-secure-text-product-integration-current-source-review.md
implementation_files:
  - zircon_runtime/src/ui/text/edit_state.rs
tests:
  - zircon_runtime/src/ui/text/edit_state.rs
---

# Runtime82 grapheme aligned edit replacement completion list

| Plan slice | Completed implementation | Acceptance boundary | Status |
| --- | --- | --- | --- |
| Reuse aligned offsets in local text edits | Selection insert, caret insert, Backspace, and Delete skip a redundant pair of grapheme scans; composition paths retain defensive range clamping. Mixed Unicode parity and the exact old helper comparator are in the local tests. | Grouped managed Runtime lib tests and ignored Release `RUNTIME82_UNICODE_SELECTION_REPLACE_SINGLE_CLAMP_BENCH_V1`; nonempty trailing grapheme replacements retain 31 paired raw samples and p50/p95/p99 for each scale, with p95 at 1,024 and 8,192 combining graphemes at most 75% of the former helper. | implemented_pending_validation |

Product-scale document editing and memory gates remain open until the
managed qualification batch supplies those results.
