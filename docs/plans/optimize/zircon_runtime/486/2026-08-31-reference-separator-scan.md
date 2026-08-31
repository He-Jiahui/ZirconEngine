---
title: Runtime486 Reference Separator Scan
category: zircon_runtime
report_id: Runtime486-reference-separator-scan-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime486 Reference Separator Scan

Runtime capability-status Bevy reference validation now detects backslash and colon bytes in one
pass instead of issuing two independent substring scans. Repository prefix, segment validation,
diagnostics, and Unicode path behavior remain unchanged.

Regression coverage verifies both rejected separators, valid forward-slash paths, and non-ASCII
content. The ignored Windows Release benchmark emits
`RUNTIME486_REFERENCE_SEPARATOR_SCAN_BENCH_V1` over 17 alternating paired samples, a 4,097-byte
reference with a trailing colon, and 16,384 checks per sample. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.80` (at least 20% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime486 is prepared with Editor416 under request
`runtime486-editor416-performance-batch-20260831gd-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
