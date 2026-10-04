---
title: Editor07 Pending Edit Page Single-Scan Projection
category: zircon_editor
report_id: Editor07-pending-edit-page-single-scan-2026-09-13
date: 2026-09-13
related_to:
  - docs/plans/optimize/zircon_editor/07/2026-08-26-pending-edit-page-capacity.md
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor07 Pending Edit Page Single-Scan Projection

## Scope

The bounded pending-edit page already reserved its output vector from a
candidate upper bound, but the implementation still cloned the filtered
iterator and called `count()` to discover the exact number of matches. That
walked the complete retry-plus-pending queue before the page projection walked
it again, even though the product limit is capped at 128 entries.

## Implementation

`PendingEditQueue::page` now reads `candidates.size_hint().1`, clamps the safe
upper bound to the existing `1..=128` page limit, and materializes the page in
the retained iterator. The post-page `next_cursor` probe remains unchanged.
The iterator's ordering, cursor filtering, retry precedence, compact payload,
and empty-queue behavior are unchanged.

## Deterministic performance model

For a queue containing `N` entries, the old exact-capacity path performed a
full `N`-entry count walk plus the bounded page walk. The new path obtains the
upper bound in `O(1)` and visits at most 128 entries for projection and the
single next-cursor probe. The reservation may be conservative when a cursor
filters out most entries, but it is bounded and removes the queue-wide scan.

## Validation

- The existing Editor07 source regression now requires the upper-bound
  expression and rejects the clone/count path.
- A new source/pressure contract covers the one-pass iterator shape, retained
  next-cursor probe, and a `1,048,576`-entry queue model (`4/4`).
- Scoped Rustfmt, Python compilation, and the batched Runtime/Editor
  performance-contract loader remain the required local checks. Managed
  Windows Cargo/Release compilation and product allocation/page-latency gates
  remain pending under the existing external-worktree admission blocker.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/play/pending_edits/queue.rs` | `BC593406FDB7125BAE13C8FFAE4DA4DA0764E00CA84C69FBA8586D1FDC7DB939` |
| `zircon_editor/src/core/play/pending_edits/tests.rs` | `C0C4574FC2A97DB244A4ECE5B50251A8F5A1A2E5E63488F1153EBA34B5C7A7B6` |
| `tools/tests/test_editor07_pending_edit_page_single_scan_performance_contract.py` | `D362263AB38424A0C2A75990BEE84A28EBAE773F5CC15247226724D447A3E20E` |

## Acceptance boundary

This is implementation-complete source evidence only. Keep
`validation_status: managed_validation_pending` until the owner-attributed
batched managed run verifies compilation, page-order parity, allocation, and
pending-edit page p50/p95/p99 latency. Tooling changes remain out of scope.
