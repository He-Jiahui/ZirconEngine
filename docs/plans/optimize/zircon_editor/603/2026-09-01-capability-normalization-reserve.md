---
title: Editor603 Capability Normalization Reserve
category: zircon_editor
report_id: Editor603-capability-normalization-reserve-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor603 Capability Normalization Reserve

`EditorCommandDescriptor::with_required_capabilities` now reserves the incoming iterator's lower
bound before extension and uses unstable sorting before deduplication. Capability order remains the
same canonical lexical order, duplicates remain suppressed across chained builder calls, the public
descriptor shape is unchanged, and serialized command metadata remains deterministic.

A focused behavior test covers unsorted duplicates across two chained calls. A source guard requires
the size-hint reservation and unstable sort and prevents restoration of stable sorting. The ignored
Windows Release benchmark emits `EDITOR603_CAPABILITY_NORMALIZATION_BENCH_V1` over 17 alternating
sample pairs with two chained 4,096-capability inputs containing long, non-ordered names. The gate
requires reserved unstable-sort P95 to be at most 90% of the legacy stable-sort P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor603 is prepared with Runtime603 under request
`runtime603-editor603-hook-capability-performance-20260901hu-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
