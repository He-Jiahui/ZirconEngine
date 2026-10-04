---
title: Editor paint primitives shared-image and row-raster protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-paint-primitives-shared-image-and-row-raster-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor retained-host paint_primitives.rs + paint_primitives/**`
- 26/26 Rust files source-reviewed. Viewport data owns Arc pixels but paint downgrades to a slice and
  recopies `4WH` bytes per recorded capture; raw recording can hash/copy whole images; software rounded
  fill/border rebuild geometry per pixel with up to 16/32 edge samples; text markers measure before
  layout. GPU record-only clip/typed command behavior and shared template/atlas pixels are retained.
  M1 now preserves the viewport Arc and changes per-recording pixel copies `4WH -> 0` (focused GREEN
  2/2; owned contracts GREEN 79/79). M0/M2-M6 row-raster/layout/opacity/profile/power acceptance remain
  pending.

Do not add these files to `review.md` before M0-M6 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M6 to MVP viewport recording and primitive paint. Record copied/shared/hash/upload bytes,
rounded setup/row/edge/span work, text measure/layout/cache locks, alpha scans, CPU/RSS/latency/context
switches, WPR power and RenderDoc parity.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of raw-slice recording, duplicate image identifier/pixel ownership, impossible hybrid
paint modes and per-pixel rounded geometry after shared handles/prepared targets are authoritative.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own viewport generation-to-resource-handle transfer, damage-triggered primitive recording and one
prepared text/layout artifact across shell/native pane paint.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own canonical prepared image/text/rounded-geometry contracts and row-span software raster interfaces
consumed by Runtime UI and editor retained-host paint.

## `docs/plans/zircon_runtime/render/17-performance-and-profiling.md`

Own image upload/residency counters, typed rounded primitive batching and RenderDoc draw/upload/GPU
parity. CPU/WPR evidence remains owned by the MVP performance plan.

## Acceptance handoff

The handoff requires 26/26 post-change fingerprints, focused and managed Rust behavior tests, the
viewport/image/rounded/text/damage/backend matrix, same-executable WPR/power artifacts on D/E/F,
RenderDoc draw/GPU/pixel/text parity, milestone commit and quantified WeCom notification. Protected
ledgers remain unchanged until then.
