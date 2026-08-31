---
title: Editor48 Message Bus Zero-route Fast-path Optimization
category: zircon_editor
report_id: Editor48-message-bus-zero-route-fast-path-2026-08-24
date: 2026-08-24
session_id: root-editor48-no-route-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor48 Message Bus Zero-route Fast-path Optimization

## Scope

This slice removes delivery work for `Publish` messages whose topic has no subscribers. It does not
claim the parent plan's typed `NoRoute` disposition, subscription lease, shutdown, request,
retention-scope, product-consumer, or dirty-publication milestones are complete.

## Implementation

`EditorMessageBus::prepare_publish` now checks the topic index before reserving a delivery sequence,
building the target vector, estimating retained payload bytes, or allocating the shared delivery
owner. The empty dispatch report preserves the previous externally visible delivered/coalesced/
dropped/backpressured result.

The undelivered message is returned from the locked preparation phase alongside the report. The
thread-safe bus therefore releases its metadata mutex before destroying a potentially large custom
payload; the fast path does not trade allocations for longer lock hold time.

A regression places the sequence counter at `u64::MAX - 1`, performs an unrouted publish, then
proves that a routed publish can still reserve the final sequence and deliver exactly once.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| 100,000 zero-route publishes | >= 200,000 allocations: owned topic clone + shared delivery owner | <= 100,016 allocations; <= 500 ms | >= 49.992% allocation reduction |
| Delivery sequence reservations | 100,000 | 0 | 100% reduction |
| Retained-byte estimates and target/delivery materialization | 100,000 | 0 | route lookup terminates before payload accounting |

The ignored Windows-native release evidence prints `EDITOR_MESSAGE_NO_ROUTE_BENCH_V1` with exact
allocation operations, allocated bytes, elapsed nanoseconds, and the computed reduction. Exact
runtime values are accepted only from coordinator terminal evidence.

## Validation

- Exact `rustfmt --check`, scoped `git diff --check`, sequence-exhaustion regression, and ignored
  release allocation evidence are submitted as one multi-task coordinator batch with Runtime64.
- No local Cargo lane is launched and no compilation is monitored in real time.
- Final validation ticket, terminal marker values, and commit integration remain pending.

## Remaining Parent-plan Work

The report still cannot distinguish `NoRoute` from a successful empty delivery, and broadcast with
zero total subscribers still follows its existing sequence/materialization path. The parent plan's
subscription lifecycle, bounded plugin callback queue, product consumer, shutdown, and diagnostic
work remains open.
