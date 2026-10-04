---
title: Runtime UI surface frame patch-range capacity
category: zircon_runtime
report_id: Runtime833-surface-frame-patch-range-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime833 · surface frame patch-range capacity

## Scope

`UiSurface::mark_surface_frame_rebuild_dirty` receives an optional `BTreeSet` of
exactly the nodes whose render commands may be patched. The range projection
previously used an empty `Vec` collector, so a dense partial-render update paid
geometric growth even though the node-set length was already an upper bound.

## Implementation

- Reserve `node_ids.len()` once for the temporary patch-range vector.
- Extend the existing `filter_map` directly into that buffer, preserving the
  `BTreeSet` iteration order, missing-command filtering, range endpoints, and
  subsequent merge semantics.
- Keep `None` full-snapshot fallback and empty-set zero-capacity behavior
  unchanged.
- Add the lower source regression and ignored
  `RUNTIME833_SURFACE_FRAME_PATCH_RANGE_CAPACITY_BENCH_V1` Release marker.

This is a local publication-buffer bound; it does not create a second render
authority or change the persistent frame/command patch contract.

## TDD and deterministic model

The Python source/model contract was intentionally RED before the production
reservation and GREEN after it (`2/2`). For a dense 4,096-node patch set, the
zero-capacity model changes `12` geometric growth events to `0`; the source
bound remains a safe upper bound because command lookup may filter nodes out.
These counts are allocation-shape evidence only, not allocator, CPU, RSS, or
product percentile evidence.

## Local validation

- `tools/tests/test_runtime_surface_frame_patch_range_capacity_performance_contract.py`:
  `2/2`.
- Scoped `rustfmt --edition 2021 --check` passes for
  `zircon_runtime/src/ui/surface/surface/frame_publication.rs`.
- The batched Runtime/Editor smoke set covers 169 source/model contract files
  and passes `626/626` tests in `5.379s` with zero failures, errors, load
  errors, or skips.
- Managed Windows Cargo/Release compilation, lower Rust execution, allocator
  evidence, and surface-frame product p50/p95/p99 remain pending in the
  asynchronous owner-attributed batch. Tooling production remains deferred
  for the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/surface/surface/frame_publication.rs` | `644AF5A70E7B37B032D60474A4EE11ACE7D41406E816ED7E801E3C9D7B82D748` |
| `tools/tests/test_runtime_surface_frame_patch_range_capacity_performance_contract.py` | `5E5C4D1676460FAADDB130DABCD4623DE3B14F8085642183072F12A744C0A30D` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release batch compiles the current Runtime/Editor
tree, executes the lower regression and ignored marker, and supplies allocator
plus surface-frame product p50/p95/p99 evidence. Local source/model evidence
does not claim final performance acceptance.
