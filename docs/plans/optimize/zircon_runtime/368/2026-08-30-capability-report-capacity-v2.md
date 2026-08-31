---
title: Runtime368 Capability Report Capacity v2
category: zircon_runtime
report_id: Runtime368-capability-report-capacity-2026-08-30-v2
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime368 Capability Report Capacity v2

This v2 record corrects the benchmark's baseline result type so both old and optimized paths
construct `RenderCapabilityMismatchDetail` values. The production capacity optimization and the
class/order behavior remain unchanged from v1.

The ignored Windows Release benchmark emits `RUNTIME368_CAPABILITY_REPORT_CAPACITY_BENCH_V1` and
requires `candidate_p95_ns <= baseline_p95_ns * 0.70`. The coordinator owns the combined
Runtime/Editor compile, six-test batch, exact p95 evidence, record finalization, manifest-only
commit/push, and one-shot WeCom publication. The v1 ticket remains immutable stale evidence.
