---
title: Editor372 Preallocated Package Path Sanitize
category: zircon_editor
report_id: Editor372-preallocated-package-path-sanitize-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor372 Preallocated Package Path Sanitize

Native dynamic package path sanitization now reserves the input byte length before emitting the
sanitized component. Every accepted ASCII character emits one byte and every other Unicode scalar
emits one underscore byte, so the output can never exceed that known upper bound. The former
`chars().map(...).collect()` path began from the iterator's conservative lower size hint and could
grow the string repeatedly for long ASCII package identities.

Accepted alphanumeric, dash, and underscore characters, replacement characters, and the empty-input
fallback remain unchanged. Regression coverage checks representative output and requires the
explicit capacity reservation and push loop.

The ignored Windows Release benchmark emits `EDITOR372_PREALLOCATED_PACKAGE_PATH_SANITIZE_BENCH_V1`
over 17 alternating paired samples. Each sample sanitizes a long ASCII package path 2,048 times.
The legacy model starts from the character iterator lower bound; the optimized model reserves the
full input byte upper bound. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.85`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor372 is prepared with Runtime444 under request
`runtime444-editor372-performance-batch-20260831ej-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
