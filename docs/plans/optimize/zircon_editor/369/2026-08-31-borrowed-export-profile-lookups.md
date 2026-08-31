---
title: Editor369 Borrowed Export Profile Lookups
category: zircon_editor
report_id: Editor369-borrowed-export-profile-lookups-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor369 Borrowed Export Profile Lookups

Build/export target overlay projection now borrows the profile name from the target model while it
resolves the output path, completed report, and active job. The previous projection allocated a new
`String` for every target solely to perform those borrowed lookups.

Output diagnostics, report overlays, job overlays, lookup order, and target mutation behavior remain
unchanged. All borrowed lookups complete before the target model is updated. Regression coverage
requires the shared-string borrow and rejects restoration of the profile-name allocation.

The ignored Windows Release benchmark emits `EDITOR369_BORROWED_EXPORT_PROFILE_LOOKUP_BENCH_V1`
over 17 alternating paired samples. Each sample performs 16,384 report lookups with a long profile
fixture. The legacy model allocates and copies 16,384 profile strings per sample; the optimized
model allocates none. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor369 is prepared with Runtime441 under request
`runtime441-editor369-performance-batch-20260831eg-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
