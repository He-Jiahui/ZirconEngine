---
title: Editor paint-theme prepared snapshot protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-paint-theme-precomputed-scaled-snapshot-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor retained-host host_contract/paint_theme{.rs,/**}`
- 6/6 Rust files source-reviewed. The captured theme is generation-stable, but each metric read still
  rebuilt the complete scaled metric value through 26 scalar transforms; palette copies and text Arc
  clones remain leaf-owned. M1 now precomputes scaled metrics at the publication boundary, reducing
  stable transforms `26R -> 0` (focused GREEN 3/3; owned contracts GREEN 82/82). M0/M2-M4 counters,
  explicit context, atomic publication hard cut and managed CPU-power-visual acceptance remain pending.

Do not add these files to `review.md` before M0-M4 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M4 to MVP retained editor paint/layout/hit-test work. Record theme reads, scaled-table
rebuilds, snapshot publications, palette bytes, text Arc clones, CPU/allocation/RSS/frame latency,
WPR power and scale/theme visual-pointer parity.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of thread-local/global leaf theme reads and separate component publications after the
prepared appearance context and atomic update path are authoritative.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own one prepared appearance snapshot per presentation generation and explicit borrowed propagation
through editor paint, layout, pointer hit testing and plugin-pane chrome.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own the shared Runtime UI appearance-context contract so editor and runtime controls consume prepared
metrics/palette/typography without parallel global authorities.

## Acceptance handoff

The handoff requires 6/6 post-change fingerprints, focused and managed Rust behavior tests, the
theme/scale/scene/plugin/backend matrix, same-executable WPR/power artifacts on D/E/F, visual and pointer
parity, milestone commit and quantified WeCom notification. Protected ledgers remain unchanged until
then.
