---
title: Editor602 Selective Command Projection
category: zircon_editor
report_id: Editor602-selective-command-projection-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor602 Selective Command Projection

Command-registry projection no longer clones an entire Store-owned `ContributionBatch`. It keeps
views, capabilities, menus, native bindings, and validation catalogs borrowed, clones only command
descriptors that will be published, and clones each operation factory only when its matching
command is registered. This removes deep copies of unrelated views and other contribution maps on
every registry rebuild.

View command generation, capability composition, explicit command conflict detection, native
binding validation, asset write targets, and all final contribution binding checks retain their
previous contracts. A focused test projects a capability-gated view and verifies its generated
event and normalized capability set. A source guard prevents restoring the whole-batch clone.

The ignored Windows Release benchmark emits `EDITOR602_SELECTIVE_COMMAND_PROJECTION_BENCH_V1` over
17 alternating sample pairs with 4,096 retained views, 8 commands, and 32 capabilities. The gate
requires selective-copy P95 to be at most 10% of the legacy whole-batch clone.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor602 is prepared with Runtime602 under request
`runtime602-editor602-sprite-command-performance-20260901ht-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
