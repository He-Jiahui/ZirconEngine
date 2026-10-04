---
title: Editor frame/paint geometry and damage-region-set protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-frame-paint-geometry-damage-region-set-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor retained-host frame_geometry.rs/** + paint_geometry.rs/**`
- 9/9 Rust files source-reviewed. Redraw stores one Region and bounding-unions every disjoint change
  (two opposite 16x16 regions at 1080p amplify useful area 4,050x); logical `>0` and paint `>0.5`
  visibility share ambiguous names across at least 12 definitions/wrappers, allowing accepted damage
  that paints nothing. Pixel clamping/exact fixed-size math are retained. M0-M4 region-set/semantic
  convergence/profile/power acceptance remain pending; no unsafe ABI-only edit was applied.

Do not add these files to `review.md` before M0-M4 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M4 to MVP invalidation/damage. Record input/retained/merged region count, useful/union/clipped/
presented area, amplification, promotion/no-op reason, rebuilt/reused ranges, CPU/RSS/latency/context
switches, WPR power and RenderDoc scissor/pixel parity.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of the single bounding-Region redraw ABI and parallel geometry implementations after the
retained damage set and canonical semantic policies are authoritative.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own scene-generation damage regions from pointer/layout changes through retained command ranges and
backend presentation, including measured full-redraw promotion.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own canonical logical rect, exact intersection, device paint coverage, pixel conversion and damage-set
contracts shared by Runtime UI and editor host.

## `docs/plans/zircon_runtime/render/17-performance-and-profiling.md`

Own multi-scissor/damage-range backend counters and RenderDoc draw/scissor/GPU parity. CPU/WPR evidence
remains owned by the MVP performance plan.

## Acceptance handoff

The handoff requires 9/9 post-change fingerprints, focused and managed Rust behavior tests, the region/
extent/scale/source/backend matrix, same-executable WPR/power artifacts on D/E/F, RenderDoc scissor/GPU/
pixel parity, milestone commit and quantified WeCom notification. Protected ledgers remain unchanged
until then.
