---
title: Editor paint-recording tri-state damage and retained-range protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-paint-recording-tristate-damage-retained-range-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor retained-host host_contract/paint_recording{.rs,/**}`
- 3/3 Rust files current-source reviewed. Explicit disjoint/invalid region damage clips to `None` and
  is then misclassified as full recording/rebuild; accepted regions still enter the complete workbench
  root dispatcher and rely on leaf clips/gates. M0-M4 counters, typed Full/Regions/Empty hard cut,
  DamageRegionSet, retained-range routing and dynamic acceptance remain pending.

Do not add these files to `review.md` before M0-M4 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M4 to MVP redraw/record/present work. Record damage transitions/reasons, owner/node visits,
range reuse/rebuild, command allocations, CPU/RSS/latency, WPR power and RenderDoc/pixel parity.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of bool/Option full-patch conventions, private damage geometry and clip-as-scheduler root
traversal after typed damage and retained owner/range routing are authoritative.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own explicit Empty propagation, root/chrome/dock/overlay range identity and damage-to-owner spatial
routing for editor and plugin-pane presentation generations.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own the shared Full/Regions/Empty and bounded DamageRegionSet ABI consumed by Runtime UI recording,
prepared draw lists and editor presentation.

## `docs/plans/zircon_runtime/render/17-performance-and-profiling.md`

Own backend skip-empty semantics, range/scissor/upload counters and same-workload RenderDoc evidence.

## Acceptance handoff

The handoff requires 3/3 post-change fingerprints, focused and managed Rust behavior tests, the damage/
surface/scene/backend matrix, same-executable WPR/power artifacts on D/E/F, RenderDoc/pixel parity,
milestone commit and quantified WeCom notification. Protected ledgers remain unchanged until then.
