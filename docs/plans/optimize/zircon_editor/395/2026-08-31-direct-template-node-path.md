---
title: Editor395 Direct Template Node Path
category: zircon_editor
report_id: Editor395-direct-template-node-path-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor395 Direct Template Node Path

Retained-host template surface tree construction now appends each projected node identity to the
`template_nodes/` namespace in one exact-capacity string. Node IDs, metadata, frames, state flags,
input policy, and clip behavior remain unchanged while a formatter is removed from the per-node
surface rebuild path.

Regression coverage compares empty, ordinary, nested, and symbol-bearing node identities with the
former formatter. The ignored Windows Release benchmark emits
`EDITOR395_DIRECT_TEMPLATE_NODE_PATH_BENCH_V1` over 17 alternating paired samples, each building
262,144 paths. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.80` (at least 20% lower
P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor395 is prepared with Runtime465 under request
`runtime465-editor395-performance-batch-20260831fi-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
