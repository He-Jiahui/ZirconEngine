---
title: Runtime436 Existing Disclosure State Key Update
category: zircon_runtime
report_id: Runtime436-existing-disclosure-state-key-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime436 Existing Disclosure State Key Update

Disclosure toggle reduction now probes the fixed `expanded` property by borrowed string and
overwrites the existing value in place. Repeated expand/collapse input therefore avoids allocating
a replacement `String` on every state update.

The first write still owns and inserts the property key. Every toggle still updates the expanded
flag, clears any reference source, publishes the same boolean value, and returns the same success
result. The regression test covers repeated true/false replacement and statically guards the
borrow-before-own production contract.

The ignored Windows Release benchmark emits
`RUNTIME436_EXISTING_DISCLOSURE_STATE_KEY_BENCH_V1` over 17 alternating paired samples, each
updating a pre-existing disclosure value 65,536 times. The legacy path allocates 65,536 keys per
sample and the optimized path allocates none. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime436 is prepared with Editor364 under request
`runtime436-editor364-performance-batch-20260831eb-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
