---
title: Hub 06 Bounded Background Queue Admission
category: zircon_hub
report_id: Hub06-bounded-background-queue-2026-08-25
date: 2026-08-25
session_id: root-hub06-three-task-performance-batch-r2-20260831
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Hub 06 Bounded Background Queue Admission

## Scope

This batch advances Hub04 `ZHUB-CTL-P1-14` and the Hub06 `C04`/`Q04` gates. It adds the first
explicit item budget to the existing single-worker background action queue. It does not claim that
the parent plan's byte, lane, tenant, target, fairness, cancellation, deadline, persistence, or
restart-reconciliation contracts are complete.

## Change

While the worker was active, every Build, Package, Install, or Open Editor request was cloned into
an unbounded `VecDeque`. The queue now admits at most 64 waiting requests and checks capacity before
cloning the request. The 65th waiting request returns the typed
`HubError::BackgroundActionQueueFull` without changing the running task or FIFO order.

For a 10,000-admission overload, worst-case retained requests and retained request clones change
from `10,000 -> 64`, a deterministic `99.360%` reduction. The ignored release benchmark uses a
1 KiB JSON payload and 21 alternating sample pairs. It requires the bounded path's P95 admission
time to be at most 35% of the legacy unbounded clone path; measured timings remain pending the
managed Windows-native validation ticket.

## Validation

- TDD red state: two source-contract cases errored because the queue module did not exist, and the
  typed-error case failed against the unbounded implementation.
- Source performance contract after implementation: 3/3 passed; the complete Hub06 three-task
  contract batch passes 15/15.
- `rustfmt --check` and scoped whitespace validation: passed.
- Rust behavior tests cover capacity rejection without growth and FIFO preservation below capacity.
- The real queue helper benchmark is named with the shared `hub06_` filter and emits
  `HUB06_BACKGROUND_QUEUE_BENCH_V1` with raw alternating samples and nearest-rank percentiles.
- Focused Hub binary behavior and all three ignored release performance gates are pending one
  asynchronous Windows-native coordinator ticket.
- No local Cargo lane or Cargo dry-run was launched, polled, or terminated.

## Async Validation

This record shares one immutable-source coordinator ticket with Asset and Learn Top-K ranking. The
managed Windows Rust 1.94.1 command is `cargo test --manifest-path zircon_hub/Cargo.toml --bin
zircon_hub --locked --release --jobs 1 -- hub06_ --include-ignored --nocapture --test-threads=1`.
The common `hub06_` filter runs the capacity/FIFO regressions, both exact-prefix catalog
regressions, and all three release-only benchmarks in one Cargo invocation. Commit and automatic
WeCom publication remain pending until the ticket proves the performance gates and returns the
structured metric rows.

## Remaining Parent-plan Work

Hub still requires byte and per-target budgets, multi-lane fairness, immutable target admission,
cancel/deadline support, durable operation records, process ownership, restart reconciliation, and
the broader Hub06 product-control-plane acceptance matrix.
