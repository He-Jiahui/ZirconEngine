---
title: Editor374 Direct Platform Assets Path Move
category: zircon_editor
report_id: Editor374-direct-platform-assets-path-move-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor374 Direct Platform Assets Path Move

Editor platform bundle validation now moves the already-computed `assets_root` out of the expected
layout instead of rebuilding the same path from `engine_root`. This removes one `PathBuf` allocation
and growing-root copy for every validation attempt.

Expected layout construction, engine-root validation, assets-directory validation, runtime-library
validation, target-specific launcher validation, and the returned layout remain unchanged.
Regression coverage requires the direct field move and rejects restoration of the duplicate join.

The ignored Windows Release benchmark emits `EDITOR374_DIRECT_PLATFORM_ASSETS_PATH_MOVE_BENCH_V1`
over 17 alternating paired samples. Each sample consumes 2,048 owned layouts beneath a long build
root; batch construction is excluded from timing. The legacy model allocates one replacement assets
path per layout and the optimized model directly moves the existing path. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.50`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor374 is prepared with Runtime446 under request
`runtime446-editor374-performance-batch-20260831el-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
