---
title: Editor403 Borrowed Showcase Attribute Key Update
category: zircon_editor
report_id: Editor403-borrowed-showcase-attribute-key-update-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor403 Borrowed Showcase Attribute Key Update

Component-showcase category projection now looks up the fixed `selected` and `selection_state`
attribute keys by borrowed text and replaces existing values in place. The first projection of a
missing attribute still allocates and inserts its owned key. Category routing, selection state
values, node filtering, and projection order are unchanged.

This removes two temporary key allocations per categorized node on steady-state projection
refreshes. Regression coverage checks missing-key insertion and same-key replacement. The ignored
Windows Release benchmark emits `EDITOR403_BORROWED_SHOWCASE_ATTRIBUTE_KEY_BENCH_V1` over 17
alternating paired samples and 65,536 existing-key updates per sample. Both sides include the
required TOML value-string allocation; only the legacy side performs 65,536 temporary key copies.
The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.75` (at least 25% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor403 is prepared with Runtime473 under request
`runtime473-editor403-performance-batch-20260831fq-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
