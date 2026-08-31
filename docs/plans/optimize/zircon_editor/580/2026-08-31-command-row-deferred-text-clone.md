---
title: Editor580 Command Row Deferred Text Clone
category: zircon_editor
report_id: Editor580-command-row-deferred-text-clone-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor580 Command Row Deferred Text Clone

Command-palette row label and detail painters now borrow their source text through clipping and
empty-text rejection, taking ownership only when a visible paint command is emitted. Offscreen rows
previously allocated both strings before culling. Visible command contents, styles, geometry,
ordering, and clipping behavior remain unchanged.

Regression coverage verifies that offscreen borrowed text still emits no commands. The ignored
Windows Release benchmark emits `EDITOR580_COMMAND_ROW_DEFERRED_TEXT_CLONE_BENCH_V1` over 21
alternating sample pairs and 65,536 offscreen label/detail projections per sample with 1,728-byte
text. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.50`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both batch gates pass.

## Current Batched Validation Handoff (2026-08-31)

Editor580 is prepared with Runtime580 under request
`runtime580-editor580-cookie-command-row-performance-20260831gy-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
