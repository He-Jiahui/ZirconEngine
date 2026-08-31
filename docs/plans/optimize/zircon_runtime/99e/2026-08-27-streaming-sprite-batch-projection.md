---
title: Runtime99E Streaming Sprite Batch Projection
category: zircon_runtime
report_id: Runtime99E-streaming-sprite-batch-projection-2026-08-27
date: 2026-08-27
session_id: root-runtime99e-linear-visibility-entity-projection-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime99E Streaming Sprite Batch Projection

## Scope

This slice reduces transient CPU allocation while expanding visible Sprite2D geometry into
adjacent-texture draw batches. It advances Runtime99E's CPU expansion and batching P1 work without
changing phase order, camera-layer filtering, alpha routing, texture batch boundaries, vertex
layout, tiled/sliced output, or the 1,000-slice safety cap.

## Change

- Consume phase-queue Sprite indices directly instead of collecting an intermediate index vector.
- Enumerate stretch, scale, tiled, and sliced image regions through a bounded visitor instead of a
  per-Sprite `Vec<SpriteImageSlice>`.
- Append generated vertices directly into the final adjacent-texture batch and count a Sprite only
  when it emits geometry.
- Keep the existing `build_sprite_vertices` compatibility entry for Transparent3D OIT and mixed
  mesh/Sprite submission, where independent vertex groups preserve interleaved draw order.
- Preserve the legacy empty-phase fallback until Runtime99E introduces an explicit phase-queue
  readiness generation.

The primary 2D path changes from O(N) transient per-Sprite owners plus final batch growth to O(B)
batch owners, where B is the number of adjacent texture runs. Geometry generation remains O(V).

## Deterministic Performance Evidence

Independent optimized Rust model on Rust 1.94.1, 32,768 stretch Sprites sharing one adjacent
texture batch, 21 samples per implementation, repeated in three processes:

| Metric | Per-Sprite projection then batch | Direct final-batch projection | Reduction |
|---|---:|---:|---:|
| allocations | 65,554 | 16 | 99.98% |
| allocated bytes | 23,592,744 | 14,155,560 | 40.00% |
| canonical P50 | 27,752,100 ns | 7,767,500 ns | 72.01% |
| canonical P95 | 32,495,100 ns | 12,306,200 ns | 62.13% |
| three-run worst P50 reduction | - | - | 72.01% |
| three-run worst P95 reduction | - | - | 60.45% |

The executable gate requires at least 99% fewer allocations, at least 40% fewer allocated bytes,
at least 40% lower P50, and at least 30% lower P95. The stable checksum is
`525291680169984`.

## Acceptance

- Existing stretch, tiled, sliced, UV crop, phase routing, and camera-layer Rust regressions remain
  in the focused batch.
- A new Rust regression proves adjacent Sprites write twelve vertices into one final batch.
- The Python source contract rejects phase-index and image-slice materialization in the production
  2D path and rejects the old build-then-batch renderer call.
- The independent model emits `RUNTIME99E_STREAMING_SPRITE_PROJECTION_MODEL_V1` and enforces the
  allocation, byte, P50, and P95 targets.
- Exact-file formatting, source contracts, focused Rust behavior, model execution, and scoped diff
  checks are submitted to the validation coordinator as one batch.

## Remaining Parent-Plan Work

The renderer still creates one GPU buffer and render pass per adjacent texture batch. Transparent3D
mixed submission still needs per-Sprite vertex ownership for ordering. Persistent ring-buffer or
instance storage, explicit empty-phase readiness, bounds-backed culling, material-aware batch keys,
and GPU-driven Canvas2D submission remain open Runtime99E work.
