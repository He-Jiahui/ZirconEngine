---
title: Runtime host intent outbox transaction protected plan routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-runtime-host-intent-outbox-transaction-architecture-review.md
---

# Protected plan updates

This record requests owner-plan changes without overwriting shared `review.md`, `pending.md`, or
numbered plans.

## `docs/plans/performance/pending.md`

Keep one concise module entry:

`runtime input + runtime UI + dynamic host-request ABI + App host apply` - current source reviewed;
outbox frame-watermark/runtime-UI bridge M0, borrowed page M1, and batch-level rumble expiry M1a are
statically applied; managed behavior tests, single typed drain, continuation receipt, coalescing,
WPR/power and real-window acceptance remain pending.

Do not add the module to `review.md` on static evidence or the M0 correctness repair alone.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Update PERF-MVP-425 with the current three P0 facts: product tick clears the core outbox, runtime UI
IME owns an undrained second outbox, and a 257+ request page can strand without continuation. Make
M0 correctness gates precede batching/coalescing. Record M0/M1 as statically applied but not
dynamically accepted while the managed validation owner remains archived.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Route the final shape through one Unreal-aligned retained host-intent owner: explicit cursor/IME
lifecycle, one publication transaction, request-driven platform update, and no frame-reset consumer
or disconnected surface queue.

## `docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md`

Own the versioned typed batch, continuation receipt/wake, prepare/commit/rollback parity, borrowed V1
page envelope, exactly-once output release, and idle backlog behavior. Preserve `has_more` or an
equivalent remaining-row receipt through App decode instead of discarding the batch wrapper. Update the existing
`failure-2026-07-19-app-entry-host-request-and-wake-boundary.md`; do not create a duplicate failure.

## `docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md`

Own one `HostIntentOutbox` manager transaction and the lossless-edge/latest-state/bounded-command
classification. Preserve frame snapshot semantics with an explicit watermark instead of clearing
pending host output.

## Runtime UI owner plan

Route the runtime UI input owner to publish each surface's IME requests into the shared dynamic
session transaction in stable surface order. Later converge onto the common outbox rather than
keeping a permanent bridge or App cache.

## App entry owner plan

Own a separate coalesced host-work cadence and bounded apply slice that do not imply simulation tick
or redraw. `RuntimeFrameDemand::Immediate` and an unbounded drain loop are explicitly rejected as
continuation substitutes. Record OS-call and event-loop p95/p99 data; one drain call per pump is not
accepted while the ABI can report pending pages. M1a has already moved rumble expiry from `(N + R)`
full scans to one non-empty-batch scan; retain that batch invariant.

## Acceptance handoff

Return focused behavior tests, source-operation deltas, `1/256/257/1K/10K` counters, idle
continuation evidence, and same-build WPR/allocation/power results to PERF-MVP-425. Only a fully
accepted dynamic milestone may be committed and sent to WeCom with quantified data.
