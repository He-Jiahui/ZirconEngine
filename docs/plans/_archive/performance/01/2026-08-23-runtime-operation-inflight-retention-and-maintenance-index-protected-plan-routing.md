---
title: Runtime operation in-flight retention and maintenance index protected plan routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-runtime-operation-inflight-retention-and-maintenance-index-architecture-review.md
---

# Protected plan updates

This record requests owner-plan changes without overwriting shared ledgers or numbered plans.

## Performance ledgers

Keep `zircon_runtime/src/operation/**` in `pending.md`: the reviewed 12-file source plus the new M0
behavior gate are statically covered, M0 is applied, and M1 scale counters are present, but the
behavior gate/counters have not run and M2-M5 dynamic gates remain. Do not add it to `review.md`.

Update PERF-MVP-435 in `01-mvp-performance-audit-and-optimization.md` with the new in-flight
tombstone eviction capacity leak, the current FIFO phase indexes, and the `4*T/frame` maintenance
scan model. The July 7-file synchronous snapshot is historical only.

## Optimize Runtime41

Refresh the currentness row from 11 files/2,639 lines to the current 12-file fingerprint. Remove the
obsolete `HashMap::iter().find_map` queue fact because FIFO `VecDeque` indexes now exist. Add the M0
rule that cancelled/expired metadata cannot be evicted while `prepare_in_flight`; then retain
Runtime41 ownership of descriptor, typed receipt, read-only snapshot, fairness, progress,
completion port, wake, shutdown, and real consumers.

## Runtime11 and Runtime10

Update `runtime/11/failure-2026-07-19-operation-service-synchronous-unbounded.md`: batch-scoped
completion ownership is not exact while pressure can evict an in-flight terminal task. Runtime11
owns prepare lease release, one completion authority, deadline indexes, and shutdown fence.

Update `runtime/10/failure-2026-07-29-operation-phase-detail-abi-owner-thread-apply.md`: preserve the
V2 allocation-free poll, but route encoded admission and typed/canonical result transport so byte
accounting does not reserialize JSON. Operation completion wake must not imply redraw.

## App and Editor owners

The App runtime-library and Editor session gateway currently duplicate submit encode, fixed-layout
poll validation, and harvest decode. Their package owner plans must converge on one runtime-host
adapter after the fixed app/runtime/editor hard cutover; do not add forwarding compatibility layers.

## Acceptance handoff

Return the blocking-worker M0 behavior result, `1/1K/100K` queue/permit/scan counters, owner callback
p50/p95/p99, WPR CPU/context-switch/energy evidence, retained/allocation bytes, reactive wake proof,
and shutdown census to PERF-MVP-435. Only a dynamically accepted milestone may be committed and
sent to WeCom.
