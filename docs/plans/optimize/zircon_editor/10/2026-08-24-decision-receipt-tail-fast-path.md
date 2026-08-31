---
title: Editor10 Decision Receipt Tail Fast Path
category: zircon_editor
report_id: Editor10-decision-receipt-tail-fast-path-2026-08-24
date: 2026-08-24
session_id: root-editor10-notification-projection-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor10 Decision Receipt Tail Fast Path

## Scope

This slice removes repeated retained-receipt scans when the product poll cursor is already current.
It does not claim the parent plan's durable decision workflow, unified notification authority,
owner leases, typed delivery receipts, accessibility, journal, or product-adoption milestones are
complete.

## Implementation

`DecisionNotificationCenter::receipts_since` now compares a valid, non-expired cursor with the
newest retained receipt before walking the bounded deque. An empty queue or a cursor at/after the
tail returns an unchanged empty batch immediately. This is the steady-state path used by the host
receipt pump after it has consumed the latest decision.

Foreign cursor rejection, expired cursor recovery, retained ordering, and the path that clones new
receipts are unchanged. The optimization adds no secondary index or mutation bookkeeping and keeps
the receipt capacity and eviction contract intact.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| 10,000 current-cursor polls with 256 retained receipts | 2,560,000 receipt comparisons | 10,000 tail comparisons; <= 3 s | 99.609375% comparison reduction |
| Returned receipts | 0 per poll | 0 per poll | unchanged |
| Cursor result | input current cursor | input current cursor | unchanged |

The ignored Windows-native release evidence prints `EDITOR_DECISION_RECEIPT_BENCH_V1` with the
retained count, poll count, legacy receipt checks, optimized tail checks, reduction basis points,
and elapsed nanoseconds. Exact elapsed time is accepted only from coordinator terminal evidence.

## Validation

- Exact `rustfmt --check`, scoped `git diff --check`, latest-cursor behavior, existing cursor-gap
  behavior, and ignored release evidence are prepared for a shared coordinator batch with the
  Runtime44 schedule optimization.
- No local Cargo lane is launched and no compilation is monitored in real time.
- Final validation ticket, terminal marker values, and commit integration remain pending.

## Remaining Parent-plan Work

Decision receipts remain process-local and bounded only by entry count. Workflow side effects,
owner generation, timeout/withdraw/revoke semantics, durable recovery, and generalized product
consumers beyond the Play pending-edit adapter remain open.
