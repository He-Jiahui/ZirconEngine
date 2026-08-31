---
title: Runtime456 Direct Meta File Name
category: zircon_runtime
report_id: Runtime456-direct-meta-file-name-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime456 Direct Meta File Name

Project asset scanning now appends `.zmeta` to each UTF-8 source file name through one
exact-capacity string. The former source-to-sidecar mapping invoked generic formatting for every
discovered asset before replacing the path file name.

Directory bytes, source file-name bytes, the `.zmeta` suffix, the non-UTF-8/no-file-name `asset`
fallback, spaces, and Unicode behavior remain unchanged. Regression coverage compares
representative and fallback paths with the former implementation.

The ignored Windows Release benchmark emits `RUNTIME456_DIRECT_META_FILE_NAME_BENCH_V1` over 17
alternating paired samples, each mapping 262,144 representative source paths. Both paths construct
the resulting `PathBuf`; the optimized path removes the additional generic formatting layer. The
gate requires `optimized_p95_ns <= legacy_p95_ns * 0.90` (at least 10% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime456 is prepared with Editor386 under request
`runtime456-editor386-performance-batch-20260831ex-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
