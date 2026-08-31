---
title: Inline command arena explicit idle trim
date: 2026-08-23
plan: docs/plans/optimize/zircon_tooling/25-memory-allocation-domain-budget-oom-pressure-fragmentation-pooling-cache-residency-observability-review.md
status: candidate_validation_pending
scope: deferred command packed inline arena retained backing storage
---

# Inline Command Arena Explicit Idle Trim

`CommandQueue::apply` and `reset` continue to retain packed inline blocks for normal
frame-to-frame reuse. An explicit `trim_retained_inline_storage` operation now releases the
backing allocation only when a queue has no live commands; it also trims reclaimed worker arenas.
`World` and `WorkerCommandBuffer` expose the same maintenance action for an idle or memory-pressure
owner to call without changing normal command dispatch behavior.

`CommandQueueMetrics` records the number of non-empty trim operations and the logical packed-block
capacity returned to the allocator. This capacity is not process RSS: allocator page retention and
other process mappings require the plan's later product workload capture.

The regression verifies that an active queue cannot be trimmed, that an idle one-block prewarm
releases 65,536 bytes, and that a following push allocates its block storage again. It also verifies
the World and worker-buffer maintenance facades.

`merge_worker_buffers` additionally has a direct fast path for a destination that initially owns no
worker arenas. The incoming buffers are sorted and duplicate keys are rejected before moving any
payload, so their distinct arenas can be appended directly without probing the destination arena
vector for each buffer. At 64 workers this removes the previous `0 + ... + 63 = 2,016` linear key
comparisons from that fresh-barrier path. A destination with retained arenas continues to use the
existing lookup path, preserving repeated-merge behavior and arena reclamation semantics.

The first managed Tooling25 runtime batch is pending for the prior artifact, node-pool, and arena
work. The direct worker-arena merge path is included in the next combined batch; no local Cargo
command or product memory profile was run. Its 2,016-comparison reduction is an algorithmic count,
not a measured CPU-time, throughput, p95, or RSS result.
