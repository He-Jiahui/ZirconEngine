---
title: Runtime589 UI V2 Cache Source Move
category: zircon_runtime
report_id: Runtime589-ui-v2-cache-source-move-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime589 UI V2 Cache Source Move

UI v2 file-cache miss construction now consumes its collected source set. The former path cloned
the complete root document and every source `PathBuf`, then discarded the owned inputs after
building the prototype store. The new path moves the root document and paths, while retaining only
the imported token and stylesheet copies required to assemble the merged root document.

Source order, root-first prototype insertion, resource aliases, source-key construction, imported
documents, and compilation all retain their previous contracts. The focused source guard requires
the ownership path and rejects restoration of the root-document or path clones.

The ignored Windows Release benchmark emits `RUNTIME589_UI_V2_CACHE_SOURCE_MOVE_BENCH_V1` over 21
alternating sample pairs, 256 sources, and an 8,192-node root document per sample. The legacy model
clones one root document and 256 paths; the optimized model moves them. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.60`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime589 is prepared with Editor589 under request
`runtime589-editor589-cache-viewport-performance-20260901hm-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
