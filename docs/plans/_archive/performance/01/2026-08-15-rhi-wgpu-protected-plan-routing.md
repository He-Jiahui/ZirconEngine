---
related_code:
  - zircon_runtime/crates/zr_rhi/src
  - zircon_runtime/crates/zr_rhi_wgpu/src
  - zircon_runtime/src/graphics/backend/render_backend
  - zircon_runtime/src/graphics/runtime/render_framework
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/pending.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
doc_type: implementation-evidence
status: routing_blocked_by_protected_plan_owner
created_at: 2026-08-15
---

# RHI/WGPU protected-plan routing evidence (2026-08-15)

## Coordinator decision

The 76/76-file RHI/WGPU review is complete in
`2026-08-15-rhi-wgpu-submission-gpu-lifetime-current-architecture-review.md`. It found a P0 system
split: the neutral `RenderDevice` is implemented only by a deterministic CPU test double, while the
product owns and submits WGPU devices/queues directly. The default editor submission is synchronous;
the optional private worker is a one-slot handoff; a normal presented scene has at least two source-
path submits; product readbacks retain indefinite waits; GPU lifetime, one poll owner and device-loss
generation are missing.

Performance01 attempted to authorize the required writes on 2026-08-15. The coordinator returned:

- `protected_plan_definition` for
  `docs/plans/performance/01-mvp-performance-audit-and-optimization.md` and
  `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`;
- `outside_registered_child` for `docs/plans/performance/pending.md` and the Plan02 child directory.

The current Session is correctly restricted to `docs/plans/performance/01/**`. It did not bypass
that boundary or overwrite the foreign-dirty global indexes. This record is ordinary routing
evidence, not a `failure-*` handoff: the handoff validator requires a cross-plan failure to live in
the fixing Plan02 child directory, which this Session cannot write.

## Required owner merges

### Performance main plan

Add `PERF-MVP-635` as P0 with these current-source facts and route it to Plan02 M3,
Render01/02/17 and Runtime11:

- RHI/WGPU 76/76 files, 23,731 physical lines, 249 tests, manifest
  `336dfe9df6fca33f03ef6f45ce11eb721ec3b8989f829781bbb6bf4ff7192a53`;
- product bypasses `RenderDevice`; default editor is synchronous; private worker is one-slot;
- normal presented viewport has main submit plus surface-blit submit; output writeback may add one;
- direct indefinite readbacks, multiple poll/submit/resource owners and no product-wide deferred
  destruction/device-generation policy;
- hard cut to one product RHI device generation, immutable packet, submission ticket, shared
  TaskGraph RHI affinity, resource registry, async readback and diagnostics generation.

Acceptance must include 1/32/256/1k passes, 1/2/8 queues/surfaces, 1/2/3 frames in flight,
0/1/64/1k resource retire/readback cases, direct product submit/poll owner count, WPR/xperf, GPU
timestamps, RenderDoc and energy. No module is accepted on static evidence alone.

### Plan02 M3

Add the following implementation gates after the RDG packet definition:

1. one `RhiDeviceGeneration` is the only product device/queue/poll/resource owner;
2. `CompiledFrameGraphPacket` lowers to immutable dense `RhiCommandPacket` and one bounded
   `RhiSubmissionTicket` chain on the Runtime11 affinity executor;
3. surface blit, output writeback and readback copies coalesce into the owning batch, with typed
   split reasons as the only exception;
4. generation-tagged resource handles retire behind last-use tickets; device loss stops admission,
   terminates in-flight tickets and follows an explicit terminal-or-recreate policy;
5. delete product direct WGPU authorities, the private submission thread and synchronous/pipelined
   semantic fork in the same milestone.

Update the Render01/02/17 and Runtime11 cross-plan row to include PERF-MVP-635 and these gates.

### Global pending index

Update the `zircon_runtime` remaining/RHI entry to say that `zr_rhi` + `zr_rhi_wgpu` are statically
reviewed 76/76 with the evidence report above. Keep them pending because current Cargo, product
WPR/xperf, GPU timestamp, RenderDoc and energy evidence are absent. Do not touch `review.md`.

### Other plan owners

- Render01: compiled RDG packet to RHI packet, versioned resources and physical lifetime owner.
- Render02: mesh/present command coalescing and removal of pass-name lookup/owned command DTOs.
- Render17: one diagnostics generation, one poll owner, submit/split/readback/resource-lifetime
  counters, GPU timestamp/RenderDoc/energy acceptance.
- Runtime11: named RHI affinity executor, dependency-aware translate/submit tasks, bounded tickets
  and shutdown deadlines; no private submission OS thread.
- Optimize09: treat local queue/readback micro-edits only as post-hard-cut refinements; do not
  optimize the deterministic test double as if it were the product backend.

## Completion condition

This routing record can be retired only when the protected plan owners have merged the task and
pending changes, then the implementation hard cut has current-source Cargo plus WPR/xperf/GPU
timestamp/RenderDoc/energy evidence. Until then the report remains
static-complete/dynamic-blocked and no commit or WeCom milestone message is permitted.

