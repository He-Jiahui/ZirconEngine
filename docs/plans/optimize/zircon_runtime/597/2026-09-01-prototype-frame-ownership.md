---
title: Runtime597 Prototype Frame Ownership
category: zircon_runtime
report_id: Runtime597-prototype-frame-ownership-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime597 Prototype Frame Ownership

Flat UI prototype expansion now consumes the working `UiNodePrototype` after its single source
lookup clone. Native and component paths prepare child expansion tasks before moving the node and
task fields into their finalize frame. The finalize frames read child mounts from the owned node,
removing the former duplicate child-mount vectors as well as frame-level copies of the node,
tokens, params, typed binding params, control scope, and native widget type.

The child-task vector is built in reverse child order and appended after its finalize frame, so the
existing LIFO depth-first expansion and result zip order remain unchanged. Slot-fill propagation,
component scope derivation, parameter resolution, binding resolution, and component decoration
retain their previous contracts. A focused compile test covers native child order and attributes,
while source guards require both frame types to keep the owned layout.

The ignored Windows Release benchmark emits `RUNTIME597_PROTOTYPE_FRAME_OWNERSHIP_BENCH_V1` over
21 alternating sample pairs, 512-entry token/param/binding and node maps, one child mount, and 96
frame preparations per sample. The legacy model clones task/node/frame storage; the optimized model
moves frame-owned storage after preparing the required child task. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.55`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime597 is prepared with Editor597 under request
`runtime597-editor597-prototype-pane-performance-20260901hp-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
