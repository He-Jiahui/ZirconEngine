---
title: Runtime463 Direct Plugin Capability
category: zircon_runtime
report_id: Runtime463-direct-plugin-capability-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime463 Direct Plugin Capability

Builtin content-tool catalog classification now constructs its `runtime.plugin.<package_id>`
capability identity with one exact-capacity string. The catalog projection keeps the same package
bytes and capability status while avoiding generic formatting for this fixed namespace grammar.

Regression coverage compares terrain, tilemap, prefab, and nested plugin identifiers with the former
formatter. The ignored Windows Release benchmark emits
`RUNTIME463_DIRECT_PLUGIN_CAPABILITY_BENCH_V1` over 17 alternating paired samples, each building
262,144 capability identities. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.75` (at least
25% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime463 is prepared with Editor393 under request
`runtime463-editor393-performance-batch-20260831fg-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
