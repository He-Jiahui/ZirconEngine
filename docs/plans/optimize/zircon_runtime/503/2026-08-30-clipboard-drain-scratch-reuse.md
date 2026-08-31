---
title: Runtime503 Clipboard Drain Scratch Reuse
category: zircon_runtime
report_id: Runtime503-clipboard-drain-scratch-reuse-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime503 Clipboard Drain Scratch Reuse

Runtime UI clipboard host-request draining previously created and consumed a temporary vector for
every surface. The drain now owns one scratch vector outside the surface loop, clears it, and uses
`drain(..)` so its allocation is retained across surfaces. Surface order, per-surface request order,
surface index conversion, and the destination payload are unchanged.

The source regression requires one loop-external scratch buffer and forbids consuming it. The
ignored Windows Release benchmark emits `RUNTIME503_CLIPBOARD_DRAIN_SCRATCH_REUSE_BENCH_V1` for
1,024 surfaces with 64 requests each and requires optimized growth events to be no more than 10%
of the legacy count.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, focused tests, ignored performance evidence, manifest-only commit/push, and one-shot
WeCom publication after the declared gates pass.

## Current batched validation handoff (2026-08-30)

Runtime503 is batched with Editor503 under request
`runtime503-clipboard-scratch-editor503-autosave-outcome-capacity-20260830cp-v1`. Receipt, ticket,
source manifest, and terminal evidence are recorded after coordinator acceptance.
