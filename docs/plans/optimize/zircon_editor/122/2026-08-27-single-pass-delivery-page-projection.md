---
title: Editor122 Single-Pass Delivery Page Projection
category: zircon_editor
report_id: Editor122-single-pass-delivery-page-projection-2026-08-27
date: 2026-08-27
session_id: root-editor122-single-pass-delivery-page-projection-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor122 Single-Pass Delivery Page Projection

## Scope

The listener control path already releases the listener registry and retention-store locks before
projecting a delivery page. It previously cloned every shared event record into an owned
`EditorEventListenerDelivery`, collected a DTO vector, and immediately copied those owned fields
again into `serde_json::Value` rows.

`QueryDeliveriesPage` now projects each shared retained record directly into its final JSON row.
The public delivery DTO remains available for compatibility, but the JSON control response no
longer materializes it. Listener identity, delivery cursor, event identity, sequence, source,
operation metadata, arguments, group, result, page cursor, and `has_more` are unchanged. The
existing source guard continues to require JSON projection after the listener lock scope ends.

## Performance Evidence

The isolated optimized Rust model mirrors the ten owned fields of a maximum 256-record listener
page, including 96-byte operation arguments and representative operation strings. It compares the
old shared-record-to-DTO-to-owned-row path with one shared-record-to-owned-row projection. It runs
31 alternating sample pairs and 256 rounds per sample and was compiled with
`rustc +1.94.1 -O` on Windows.

| Metric | DTO then owned row | Direct owned row | Change |
|---|---:|---:|---:|
| Allocator calls per page | 3,586 | 1,793 | -50.000% |
| Cumulative requested bytes per page | 231,936 | 115,968 | -50.000% |
| P50 for 256 rounds | 106,455,700 ns | 51,217,300 ns | -51.889% |
| P95 for 256 rounds | 202,747,300 ns | 91,552,600 ns | -54.844% |

Model source:
`.codex/state/session-coordinator/editor122-single-pass-delivery-page-projection-model.rs`.

The model measures the eliminated owned DTO stage and representative payload copies. It is not a
replacement for the managed `serde_json` behavior test or end-to-end WebView latency evidence.

## Contracts And Validation

- `tools/tests/test_editor122_single_pass_delivery_page_projection_performance_contract.py` locks
  direct shared-record JSON projection, the complete response field set, absence of the DTO stage,
  and the existing lock-scope ordering guard.
- TDD RED failed all three source-contract checks against the old two-stage path; the implemented
  contract passes 3/3.
- Python bytecode compilation, scoped `rustfmt +1.94.1 --edition 2021 --check`, and scoped
  `git diff --check` pass.
- The post-implementation release model passes the allocation and P50/P95 reduction gates.
- Cargo type checking and the focused delivery-page cursor/JSON behavior test remain pending in a
  managed asynchronous coordinator batch; no direct Cargo command was run.

## Remaining Parent-Plan Work

Editor122 still owns typed envelope compatibility, listener lifecycle, replay, dirty-state
semantics, retention policy, shutdown, observability, UI integration, and product qualification.
This slice only removes the redundant owned DTO stage from listener delivery-page JSON projection.
