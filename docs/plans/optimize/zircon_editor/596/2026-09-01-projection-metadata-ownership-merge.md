---
title: Editor596 Projection Metadata Ownership Merge
category: zircon_editor
report_id: Editor596-projection-metadata-ownership-merge-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor596 Projection Metadata Ownership Merge

Retained-host projection reconciliation now consumes each temporary projection-only host node in a
single pass. When a control already exists on the runtime surface, its attributes, style tokens,
and style overrides are transferred into the surface node with ownership-aware `BTreeMap::append`
instead of cloning every key and value before discarding the temporary node. Unmatched synthetic
nodes keep the existing parent remap and insertion path.

Control matching, projection-over-surface value precedence, synthetic node order, parent remapping,
focus-state projection, and nodes without a control identifier retain their previous contracts.
Focused tests cover duplicate-key precedence and require all three metadata maps to use the owned
merge path.

The ignored Windows Release benchmark emits `EDITOR596_PROJECTION_METADATA_MOVE_BENCH_V1` over 21
alternating sample pairs, three 512-entry metadata maps, and 32 merges per sample. The legacy model
clones each map key and value; the optimized model transfers owned map entries. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.60`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor596 is prepared with Runtime596 under request
`runtime596-editor596-component-projection-performance-20260901ho-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
