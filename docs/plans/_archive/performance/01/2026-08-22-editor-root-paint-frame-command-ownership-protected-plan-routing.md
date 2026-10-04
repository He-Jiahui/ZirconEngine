---
title: Editor root paint-frame and command ownership protected routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-root-paint-frame-command-ownership-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor retained-host root composition + paint_frame/** + paint_recording/**`
- 26/26 Rust files source-reviewed. Frame interaction deep-clones an existing `Arc`; every recording
  creates fresh command Vec capacity and owned text/resource Strings; no stable command range survives
  damage recordings. Primitive damage rejection and retained CPU region repaint are correct foundations.
  M1 changes underlying state clones from active `1 -> 0` and fallback `2 -> 1` (focused GREEN 2/2;
  owned contracts GREEN 74/74). M0/M2-M6 arena/handle/range/raster/profile/power acceptance remain
  pending.

Do not add these files to `review.md` before M0-M6 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M6 to MVP frame recording/root composition. Record state clone bytes, Vec growth/capacity,
command/range rebuilt/reused bytes, owned/shared text-resource bytes, RGBA allocation/write bytes,
fallback sorts, CPU/allocation/RSS/latency/context switches, WPR power/energy and RenderDoc parity.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of flat immediate command rebuilding, duplicate String command ownership and duplicate
geometry implementations after retained window paint storage/ranges become authoritative.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own editor scene generations, shared paint snapshots, stable command range order and resize/damage
buffer ownership.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own reusable prepared render-list arenas, shared text/image resource handles and canonical geometry/
invalidation contracts consumed by editor recording.

## Acceptance handoff

The handoff requires 26/26 post-change fingerprints, focused and managed Rust behavior tests, the
command/text/image/damage/interaction/backend matrix, same-executable WPR/power artifacts on D/E/F,
RenderDoc draw/GPU/pixel/text parity, milestone commit and quantified WeCom notification. Protected
ledgers remain unchanged until then.
