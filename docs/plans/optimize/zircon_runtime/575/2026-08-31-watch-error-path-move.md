---
title: Runtime575 Watch Error Path Move
category: zircon_runtime
report_id: Runtime575-watch-error-path-move-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime575 Watch Error Path Move

Watcher error conversion now formats the notify error before moving its owned path vector into
`AssetWatchError`. The previous conversion deep-cloned the vector and every owned `PathBuf` even
though the notify error was consumed immediately afterward. The optimized conversion performs
zero path clones while preserving the assets root, ordered paths, and formatted message.

Regression coverage verifies the complete converted payload and guards the production path move.
The ignored Windows Release benchmark emits `RUNTIME575_WATCH_ERROR_PATH_MOVE_BENCH_V1` across
31 alternating sample pairs, each converting 1,024 errors with four paths. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.90` (at least 10% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both batch gates pass.

## Current Batched Validation Handoff (2026-08-31)

Runtime575 is prepared with Editor575 under request
`runtime575-editor575-performance-batch-20260831gt-v1`. Receipt, validation ticket, measured P95,
pushed SHA, and notification result are recorded only after coordinator completion.
