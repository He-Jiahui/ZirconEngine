---
title: UI surface node-pool bounded residency
date: 2026-08-23
plan: docs/plans/optimize/zircon_tooling/25-memory-allocation-domain-budget-oom-pressure-fragmentation-pooling-cache-residency-observability-review.md
status: candidate_validation_pending
scope: surface-local detached UI node reuse
---

# UI Surface Node-Pool Bounded Residency

`UiSurfaceNodePool` retains at most 256 template-identity buckets with four nodes in each bucket,
so detached reusable nodes are bounded to 1,024. A full bucket or identity table rejects new
recycles instead of retaining another node. `UiSurfaceNodePoolReport` now distinguishes those
capacity rejections from nodes that have no poolable template identity and records resident node,
bucket, and maximum-node snapshots after every detach or reuse operation.

`UiSurface::trim_retained_node_pool` gives an idle or memory-pressure owner an explicit release
path for detached reusable nodes without touching the live UI tree. Its report records trimmed
node and bucket counts, then the post-trim zero-residency snapshot. At the fixed limit, one trim
releases at most 1,024 pool-owned node entries.

This is an entry-count bound, not a byte or RSS claim: `UiTreeNode` contains variable metadata and
cache payloads, so byte accounting requires the plan's later allocation-trace work. The regression
fills all 1,024 admitted slots, then detaches one additional node and verifies that the pool remains
bounded while reporting the rejection.

The Rust regressions cover the 1,024-entry full-pool rejection and the explicit trim to zero
resident nodes. Managed Tooling25 batch validation is pending; no local Cargo command or product
memory profile was run.
