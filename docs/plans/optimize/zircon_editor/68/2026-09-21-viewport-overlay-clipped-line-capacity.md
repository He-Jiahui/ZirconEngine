---
title: Editor Viewport Overlay Clipped-Line Capacity
category: zircon_editor
report_id: Editor881-viewport-overlay-clipped-line-capacity-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor881 Viewport Overlay Clipped-Line Capacity

## Finding

Viewport handle-overlay rasterization clips every authored screen line before
computing raster bounds. The retained clipped lines were built with
`filter_map(...).collect::<Vec<_>>()`; the filtered iterator exposes no useful
lower bound, so a dense visible overlay could grow its temporary destination
geometrically. Reserving eagerly would regress the common all-invalid or
fully-offscreen path by allocating before the first retained line exists.

## Optimization

- Route clipping through one shared `clipped_raster_lines` owner.
- Keep the destination at zero capacity until the first valid clipped line is
  produced, then reserve the authored input-line upper bound once.
- Push valid lines directly in source order without a second projection.
- Preserve finite/alpha rejection, viewport clipping, width/color conversion,
  raster bounds, pixel blending, resource-key hashing, and transparent-output
  early return semantics.

## TDD and deterministic evidence

The Editor881 source/model contract was observed RED at `1/5` and GREEN at
`5/5`. Lower regressions prove 64 rejected lines leave an empty vector at zero
capacity and that mixed rejected/visible input retains visible source order
after reserving the authored upper bound.

For 4,096 retained clipped lines, the retired zero-lower-bound growth model
performs 11 geometric capacity changes and the lazy upper-bound model performs
zero. The ignored 101-pair Release marker
`EDITOR881_VIEWPORT_OVERLAY_CLIPPED_LINE_CAPACITY_BENCH_V1` emits alternating
p50/p95/p99 samples, locks both growth counts, checks output parity, and
requires reserved p95 to remain within 10% of the filtered collection.

## Local validation boundary

- Exact-file Rustfmt and scoped `git diff --check` pass.
- Editor879/880/881 plus the adjacent Asset Browser contract batch passes
  `33/33`.
- Lower Rust execution was submitted with the Editor879 compile repair in
  asynchronous v11 (PID `14240`); no per-task Cargo run is launched.
- Local evidence does not establish Windows compilation, allocator behavior,
  or viewport-overlay product p50/p95/p99 latency.
- Tooling production remains deferred for the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/host_contract/data/viewport_image/overlay.rs` | `60912616923B36178C3C22D74068D8363A9B627B2AC468D0DBE82785CD698859` |
| `zircon_editor/src/ui/retained_host/host_contract/data/viewport_image/overlay/capacity_tests.rs` | `70E0B82BFE86C8168ADF62EF73138449E10AC0B1CEB00AFE4CE153501DFF1D58` |
| `tools/tests/test_editor881_viewport_overlay_clipped_line_capacity_performance_contract.py` | `D0CFD4840E61BA21DF22610EA2796EC0FF1E19DD4C30754DCD01BE6C22F1A17E` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
a combined current-source Windows lane compiles Editor, executes the lower
regressions and ignored Release marker, and supplies allocator plus real
viewport-overlay product p50/p95/p99 evidence. The deterministic growth model
is not product acceptance.
