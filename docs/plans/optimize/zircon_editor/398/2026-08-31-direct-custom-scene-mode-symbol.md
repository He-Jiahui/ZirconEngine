---
title: Editor398 Direct Custom Scene Mode Symbol
category: zircon_editor
report_id: Editor398-direct-custom-scene-mode-symbol-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor398 Direct Custom Scene Mode Symbol

Editor viewport binding now serializes custom scene mode identities into one exact-capacity String
and writes the `Custom:` prefix plus mode ID directly. Built-in symbols, custom round trips, empty
custom IDs, and reserved built-in IDs preserve their existing behavior.

Regression coverage checks exact custom bytes and parser semantics. The ignored Windows Release
benchmark emits `EDITOR398_DIRECT_CUSTOM_SCENE_MODE_SYMBOL_BENCH_V1` over 17 alternating paired
samples, each serializing 262,144 representative plugin mode IDs. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.80` (at least 20% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor398 is prepared with Runtime468 under request
`runtime468-editor398-performance-batch-20260831fl-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
