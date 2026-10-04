---
title: Editor workbench dock pane damage and retained paint-plan protected routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-workbench-dock-pane-damage-paint-plan-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor retained-host paint_workbench_renderer/docks shell`
- 18/18 Rust files source-reviewed. Pre-M1 pane paint lacked root damage rejection and probed viewport,
  native, template and debug backends before final primitive clip; active content still has no retained
  paint plan, stable toolbar text is remeasured, and rail/floating candidates are fully scanned. M1
  changes off-damage shell/body calls `1 -> 0` and backend probes `4 -> 0` (focused GREEN 2/2; owned
  contracts GREEN 61/61). M0/M2-M6 retained-plan/range/index/profile/power acceptance remain pending.

Do not add these files to `review.md` before M0-M6 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M6 to the basic editor dock/pane paint path. Record pane rejects/backend probes, retained
plan/range rebuild/reuse bytes, toolbar text measures/layout, rail/floating candidate scans, CPU/
allocation/RSS/latency/context switches, WPR power/energy and RenderDoc draw/GPU/pixel parity.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own removal of string-based per-repaint pane backend fan-out and immediate dock command
reconstruction after retained pane plans and prepared render-list consumers migrate.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own pane/dock identities and generations, exact damage routing, active backend/layer paint plans,
retained toolbar/chrome ranges, active rail indices and floating dirty-window candidates.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own the shared prepared render-list/range contract, text-layout artifact generations and canonical
batch ordering used by editor dock consumers.

## Acceptance handoff

The handoff requires 18/18 post-change fingerprints, focused and managed Rust behavior tests, the
pane/floating/rail/damage matrix, same-executable WPR/power artifacts on D/E/F, RenderDoc draw/GPU and
pixel/text parity, milestone commit and quantified WeCom notification. Protected ledgers remain
unchanged until then.
