---
title: Editor host presenter retained-artifact and backend-parity protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-host-presenter-retained-artifact-backend-parity-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor retained-host host_contract/presenter/**`
- 32/32 Rust files source-reviewed. Correct GPU cache/retry and softbuffer dirty-row foundations are
  retained. M0 actual-damage attribution is applied and statically GREEN, but Rust/dynamic counter
  acceptance remains pending. M0-M5 still require requested/prepared/submitted counters, one
  generation-owned prepared artifact, transient diagnostics without presentation clones/self-
  invalidation, backend-parity resize transactions, measured softbuffer pixel path, multi-region
  propagation and current-source WPR/power/RenderDoc acceptance.

Do not add these files to `review.md` before M0-M5 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M5 to MVP editor paint/present/fallback. Keep requested, prepared, submitted and presented
damage/pixels separate; record artifact builds/range visits, DTO clones, overlay iterations, resize
scene builds, raster/copy/format bytes, CPU/RSS/p95 latency/context switches/package energy and
same-build RenderDoc evidence.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of presentation-DTO presenter inputs, the default ordinary-full native-resize method,
diagnostics mutation of presentation data, per-present native size polling and single-rectangle damage
APIs after typed replacements are authoritative.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own `PreparedHostPresent`, retained command/range generations, target overlays and exact damage receipts
shared by main/native windows and both presenter backends.

## `docs/plans/zircon_editor/editor_layout/07-windowing-chrome-tabs-and-dockable-drawers.md`

Own the backend-neutral native resize transaction and final exact layout. GPU and softbuffer must not
have different scene-rebuild semantics.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own generation-owned UI render artifacts, transient diagnostic layers, cached invalid ranges and the
bounded damage-region contract consumed by presenters.

## `docs/plans/zircon_runtime/render/17-performance-and-profiling.md`

Own requested/prepared/submitted GPU counters, RHI draw-list/scissor/cache evidence, presenter capture
readback, softbuffer raster/copy/format counters and RenderDoc parity. WPR CPU/power remains owned by
Performance01.

## Acceptance handoff

The handoff requires 32/32 post-change fingerprints, focused and managed Rust tests, backend/cache/
damage/resize/overlay/image/retry matrices, same-executable WPR and power artifacts on D/E/F,
current-source RenderDoc GPU parity, milestone commit and quantified WeCom notification. Protected
ledgers remain unchanged until then.
