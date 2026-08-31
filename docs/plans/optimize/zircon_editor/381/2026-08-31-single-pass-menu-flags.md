---
title: Editor381 Single-Pass Menu Flags
category: zircon_editor
report_id: Editor381-single-pass-menu-flags-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor381 Single-Pass Menu Flags

Popup menu row styling now resolves the `loading` and `danger` flags in one traversal of the raw
menu flag segment. The former style projection reconstructed and traversed the split iterator once
for each flag. The combined classifier uses the distinct flag lengths to avoid unrelated
case-insensitive comparisons and stops when both states are known.

Explicit `item.loading` state, whitespace trimming, empty-flag filtering, ASCII case-insensitive
matching, exact whole-flag semantics, and danger styling remain unchanged. When explicit loading is
already true, the classifier only searches for danger. Regression coverage compares mixed-case,
explicit-loading, prefix-only, and absent-flag cases with the former two-query implementation.

The ignored Windows Release benchmark emits `EDITOR381_SINGLE_PASS_MENU_FLAGS_BENCH_V1` over 17
alternating paired samples. Each sample performs 8,192 style-flag checks over 128 flags with
`loading` and `danger` at the tail. The legacy path traverses the flag segment twice per check; the
optimized path traverses it once and length-classifies unrelated flags. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.60`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor381 is prepared with Runtime451 under request
`runtime451-editor381-performance-batch-20260831es-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
