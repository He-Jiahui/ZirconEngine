---
title: Editor368 Direct Export Output Path Move
category: zircon_editor
report_id: Editor368-direct-export-output-path-move-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor368 Direct Export Output Path Move

The desktop export output picker now formats its success status while the selected path is still
borrowable, then moves the `PathBuf` directly into the per-profile override map. The old path cloned
the selected output solely to retain it for subsequent status formatting.

The stored path, status text, wizard invalidation, layout invalidation, cancel behavior, and error
behavior remain unchanged. Regression coverage requires status formatting before the direct move
and rejects restoration of the success-arm path clone.

The ignored Windows Release benchmark emits `EDITOR368_DIRECT_EXPORT_OUTPUT_PATH_MOVE_BENCH_V1`
over 17 alternating paired samples. Each sample models 8,192 successful selections with a long
nested output path. The legacy model performs 8,192 extra `PathBuf` clones per sample; the optimized
model performs zero. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.85`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor368 is prepared with Runtime440 under request
`runtime440-editor368-performance-batch-20260831ef-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
