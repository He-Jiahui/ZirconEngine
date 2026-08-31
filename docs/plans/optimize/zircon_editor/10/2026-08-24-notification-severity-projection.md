---
title: Editor10 Notification Severity Projection Optimization
category: zircon_editor
report_id: Editor10-severity-projection-2026-08-24
date: 2026-08-24
session_id: root-editor10-notification-projection-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor10 Notification Severity Projection Optimization

## Scope

This slice closes the Editor10 P0 severity-loss defect in the retained notification-center parser
and preserves the existing legacy `kind=done` fallback without treating notification kind as an
authoritative severity. It also removes lowercase-copy allocation from tone normalization and
avoids an intermediate string clone when table-backed rows select their tone.

It does not claim the parent plan's toast delivery queue, durable history, product center entry,
typed actions, accessibility, or cross-process recovery work is complete.

## Implementation

Pipe-backed rows now track whether `severity`, `level`, or `tone` supplied an explicit tone. A
`kind` field may provide the legacy fallback only until an explicit tone is seen; later `kind`
fields cannot overwrite it, and a later explicit field still wins. Table-backed rows use the same
precedence by selecting `severity`, `level`, and `tone` before `kind`.

Tone normalization compares the borrowed input with `eq_ignore_ascii_case` instead of allocating a
lowercase `String`. Table projection borrows the selected TOML string through normalization and
performs only the final owned-row conversion. The existing title fallback now distinguishes a
missing title from an explicitly empty title.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| Tone normalization, 400,000 calls | 400,000 lowercase `String` allocations | 0 lowercase allocations; <= 250 ms | 100% lowercase-allocation reduction |
| Table tone selection | cloned selected TOML string before normalization | borrowed selection through normalization | one intermediate clone removed per projected row |

Elapsed time and normalization throughput are accepted only from the Windows-native release
evidence ticket. Allocation counts are source-deterministic but do not replace managed execution.

## Validation

- Exact `rustfmt --check` and scoped `git diff --check`: passed.
- Notification-center regression batch plus release evidence: pending coordinator terminal
  evidence.
- Ticket `a0f43e6e7cd541bb8dd61f40f6247457` uses one direct release Cargo invocation with the
  `notification_center` filter and `--include-ignored`, so regressions and performance evidence
  share one compilation and external sources remain coordinator-pinned. Terminal evidence is not
  inferred from the queued receipt.

## Remaining Parent-plan Work

Error toasts can still expire while queued behind another toast, and the ordinary notification
center still lacks durable history and a product entry point. Those separate Editor10 P0 items
remain open.
