---
title: Runtime631 Preallocated Reference Candidates
category: zircon_runtime
report_id: Runtime631-preallocated-reference-candidates-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime631 Preallocated Reference Candidates

Filesystem-backed reference resolution now reserves both ambiguity candidate vectors from the
project root count. Each root can contribute at most one candidate to either locator-to-hint or
hint-to-locator resolution, so the source slice length is a real output upper bound.

Root traversal order, source safety checks, ambiguity decisions, repair payloads, and the
single-candidate clone remain unchanged. Empty roots still reserve zero capacity.

The ignored Windows Release benchmark emits `RUNTIME631_PREALLOCATED_REFERENCE_CANDIDATE_BENCH_V1`
over 17 alternating sample pairs and 32,768 roots. The gate requires the exact-bound preallocated
path P95 to be at most 85% of the unreserved path P95.

No direct Cargo validation was run. The coordinator owns the batched Runtime/Editor Windows
Release regression and performance validation. Receipt, measured P95, commit, push, and WeCom
outcome are recorded only after coordinator completion.

## Managed Validation Attempt (2026-09-01)

Exact `rustfmt --check`, `git diff --check`, source-shape guards, and SHA-256 capture passed for
the four Runtime631/Editor631 source and regression paths. Request
`runtime631b-editor631b-reference-when-capacity-performance-20260901iv-v1` returned
`offline/descriptor_absent` before admission. No validation ticket exists from this attempt; no
test, performance, commit, push, or WeCom success is claimed.
