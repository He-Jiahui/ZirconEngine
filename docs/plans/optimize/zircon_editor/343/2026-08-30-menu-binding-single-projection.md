---
title: Editor343 Menu Binding Single Projection
category: zircon_editor
report_id: Editor343-menu-binding-single-projection-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor343 Menu Binding Single Projection

Editor reflection snapshots now project each menu binding once, derive its native representation
from that owned projection, and move the binding symbol into the reflected action. The previous
path constructed one `UiEventBinding` to read the symbol, cloned that symbol, then reconstructed the
complete binding through `EditorUiBinding::native_binding`. Menu property storage also reserves its
known maximum of four entries, avoiding growth when operation path and shortcut metadata are both
present.

The ignored Windows Release benchmark emits `EDITOR343_MENU_BINDING_SINGLE_PROJECTION_BENCH_V1`
over 17 alternating paired samples with 8,192 projections per sample. Its Custom binding contains
16 owned string arguments so the gate measures the removed payload deep copy, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Editor343 is prepared with Runtime413 under request
`runtime413-editor343-performance-batch-20260830df-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
