---
title: Editor615 Single Probe Locale Admission
category: zircon_editor
report_id: Editor615-single-probe-locale-admission-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor615 Single Probe Locale Admission

Editor localization bundle construction now uses `BTreeMap::entry` to combine normalized-locale
duplicate detection and insertion. The previous path performed `contains_key` and then `insert`,
traversing the ordered map twice for every valid locale.

Canonical locale ordering remains owned by the same `BTreeMap`. The occupied entry reports the
normalized locale, and entry selection still happens before translation validation, preserving the
existing rule that a repeated locale is diagnosed before an empty repeated bundle. Focused
behavior coverage locks that precedence for `zh-CN` and `zh-cn` aliases.

The ignored Windows Release benchmark emits `EDITOR615_SINGLE_PROBE_LOCALE_ADMISSION_BENCH_V1`
over 17 alternating sample pairs with 32,768 long locale identity keys. It compares contains plus
insert against one entry lookup. The gate requires entry-admission P95 to be at most 70% of the
double-probe P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor615 is prepared with Runtime615 under request
`runtime615-editor615-chunk-locale-performance-20260901ie-v1`. Receipt, validation ticket, measured
P95, pushed SHA, and notification result are recorded only after coordinator completion.
