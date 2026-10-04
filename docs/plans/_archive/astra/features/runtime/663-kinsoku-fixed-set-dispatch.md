---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/81/2026-08-26-kinsoku-match-dispatch.md
  - docs/plans/optimize/zircon_runtime/81-runtime-text-shaping-unicode-bidi-script-run-cluster-line-break-wrap-layout-product-integration-current-source-review.md
related_code:
  - zircon_runtime/src/text/layout/kinsoku.rs
  - zircon_runtime/src/text/layout/kinsoku/match_dispatch_tests.rs
tests:
  - zircon_runtime/src/text/layout/kinsoku/match_dispatch_tests.rs
---

# Runtime81 Kinsoku Fixed-Set Dispatch

The Kinsoku line-start predicate already used a direct match dispatch, but the
fixed JLREQ inseparable-pair and forbidden line-end sets still performed a
linear slice scan on every candidate. Both helpers now use compile-time match
dispatches. The five pair entries and seventeen line-end characters are
unchanged; chunk merging, mandatory breaks, protected suffix handling, and
fallback admission retain their previous behavior.

The three protected-boundary helpers also no longer count every Unicode scalar
in a chunk before inspecting its first or last scalar. They now consume at most
two scalars from the applicable iterator direction, preserving the existing
empty/single-scalar rejection and multi-byte punctuation behavior.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime81 | Replace fixed-set `.contains` scans and full-chunk boundary counts with bounded direct probes while preserving membership | implemented_pending_validation | Fixed-set and boundary-cardinality regressions pass by static inspection; merged Runtime/Editor source batch `125/125`, scoped Rustfmt, and diff checks pass. Managed Runtime Cargo and release p50/p95/p99 remain pending. |

## Coordinator dispatch

The concurrent Runtime/Editor batch `astra-runtime-editor-compile-batch-20260911-r2`
was recorded in `.codex/tmp/astra-runtime-editor-batch-20260911-r2.json`.
Admission rejected all three requests with `session_not_found` before ticket
creation, so no Cargo or performance result is inferred from that response.

A fresh Runtime81 registration request
`c6d0d3f450ae4a1688c699f858e61a57` for the current Codex thread was accepted
into coordinator admission on 2026-09-11. At the recorded observation point a
maintenance mutation blocker still held the gate, so it had not created a
Session row or validation ticket. No Cargo process, test result, or performance
sample is inferred, and no validation-ticket status was queried.

The five concurrent current-source submissions in
`.codex/tmp/astra-runtime-editor-batch-20260911-r3.json` were all rejected at
admission with `validation_copy_overlay_not_owned`: the source paths were not
attributed to the current Session. That response was terminal before a copy,
Cargo process, ticket, or benchmark started. Per the asynchronous-validation
rule, this record does not retry or poll the coordinator.

## Acceptance boundary

This is a bounded line-breaking hot-path optimization. It does not claim
versioned JLREQ tailoring, full Unicode line-break conformance, or product
performance qualification. The managed batch must execute the new regression
and the existing Kinsoku workload before this row can be promoted.
