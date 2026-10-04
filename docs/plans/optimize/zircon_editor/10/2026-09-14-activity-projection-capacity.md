---
title: Editor10 Activity Projection Exact Capacity
category: zircon_editor
report_id: Editor745-activity-projection-capacity-2026-09-14
date: 2026-09-14
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor10 Activity Projection Exact Capacity

## Scope

The retained Activity surface materializes toast, progress, log, and visible decision-option
views from already bounded snapshots. Each projection previously started with an empty `Vec` and
let iterator growth discover the final size. The four paths now reserve the authoritative input
bound before materializing values, then extend in the existing source order. No filtering,
localization, expiry, severity, decision selection, or record identity semantics changed.

## Implementation

- `activity_toast_views` and `activity_progress_views` reserve `snapshots.len()`.
- `activity_log_views` reserves `records.len()` before cloning the retained records into view
  wrappers.
- `activity_decision_options` reserves the first presented decision's option count before
  formatting labels and constructing selection IDs.
- The lower Rust regression is wired as `activity/capacity_tests.rs`; the ignored release marker
  is `EDITOR745_ACTIVITY_PROJECTION_CAPACITY_BENCH_V1`.

## Deterministic work model

For 4,096 projected values, the legacy empty vector requires 11 geometric growth events in the
model (0, 4, 8, ... 4,096). Exact reservation requires zero growth events. The model is a
structural allocation proxy only: it excludes allocator internals, localization cost, CPU time,
RSS, and input-to-present latency. The ignored benchmark reports paired samples but does not make
a product percentile claim.

## Validation

- TDD RED/GREEN source contract: `4/4`.
- The merged Activity/notification contract slice passes `31/31` in one invocation, including
  the new capacity contract and the existing ordering/localization/retained-surface checks.
- New lower Rust capacity/order model and ignored release marker are present.
- Standalone `rustc --edition 2021 -O --test` execution passes the two lower tests and runs the
  ignored marker: `legacy_growth_events=11`, `optimized_growth_events=0`, paired synthetic
  `legacy_p95_ns=65800` versus `optimized_p95_ns=38100` (57.9% of legacy, below the informational
  local 70% guard). Timing is reported without a flaky assertion; this is a local harness
  measurement, not a managed Cargo or product latency result.
- New Python contract compiles with `py_compile`.
- All four touched Activity Rust files pass `rustfmt --edition 2021 --check`.
- The post-Editor745 merged Runtime + Editor performance-contract batch passes `1836/1836`
  in `9.621s`; the full non-tooling discovery passes `3585/3585` in `478.170s` in one
  process. Non-strict Wiki validation reports `272/272` pages with zero errors and one
  pre-existing warning. These are local source/model receipts, not managed Cargo or Release
  evidence.
- A later current-source rerun of the merged performance-contract loader also passes
  `1836/1836` in `14.186s` in one process.
- The latest current-source non-tooling Runtime + Editor discovery then passes
  `3585/3585` in `775.037s` in one process. This fresh batched regression
  receipt remains local evidence and does not replace the managed Cargo/Release
  or product percentile gate.

## Complexity boundary

Each projection remains linear in its bounded input. The change removes geometric output-vector
growth on the common path and preserves spare capacity only when a future presentation policy
filters values. It does not claim that the full Activity window, notification store, or product
UI latency budget is solved.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/activity/view.rs` | `F7839CF1079D6BF7B88DC4BE6BF2686DF6AE2C28E57DEA6086B1BF8964365349` |
| `zircon_editor/src/ui/activity/decision/view.rs` | `8549AA45C5011296C5780D7C546E93A5E3FAAEF07C152BF5A2B4153A03762CBF` |
| `zircon_editor/src/ui/activity/mod.rs` | `488051A5B7BB557D5E6E8BBE08E6E38FE699402C78E611C3689B075F680007F5` |
| `zircon_editor/src/ui/activity/capacity_tests.rs` | `E1CC87521106087A9D099200CE007A5BA88B63FCDF99AE5D208159FB44245A5E` |
| `tools/tests/test_editor_activity_projection_capacity_performance_contract.py` | `5A2165D71BBA2643D717D0C0E9099976398651DAEAD2BF2A7094F99802B5908F` |

## Managed gate

This slice joins the existing batched Runtime/Editor Windows validation lane. Do not mark it
product-validated until the owner-attributed Cargo/Release run proves compilation, Activity
ordering, allocation behavior, and product p50/p95/p99 latency. No standalone Cargo invocation or
coordinator polling was performed; tooling production work remains deferred.
