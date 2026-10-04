---
title: Workbench menu bar shared layout generation protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-ui-workbench-menu-bar-shared-layout-generation-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` folder entry:

`zircon_editor/src/ui/workbench/menu_bar` - 2/2 Rust files reviewed and static-format passed; pure
slot metrics are constant-time, while parent menu generation, shared chrome/pointer layout
generation, managed tests and current-source WPR/allocation/power evidence remain pending.

Do not add this folder to `review.md` before all M0-M4 gates pass.

## Existing menu generation record

Update
`docs/plans/performance/01/2026-08-19-editor-ui-workbench-model-domain-generation-menu-compilation-architecture-review.md`
with the downstream requirement that retained menu pointer layout and chrome projection consume the
same immutable generation. A stable unrelated Host recompute must perform zero menu tree allocation,
action/preset clone, text measurement or asset projection before an equality check.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Keep this within the existing MVP command/menu generation milestone. Add counters for pointer layout
builds, menu asset projections, label/popup measures, `MenuItemSpec` nodes, cloned bytes and
compare-after-build discards.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

After all consumers use the shared generation, delete production duplicate menu tree/geometry
materialization paths. Do not leave a compatibility pointer-menu compiler.

## `docs/plans/zircon_editor/editor_ui/08`

Own the retained projection/pointer consumer cutover and structural/geometry/context invalidation
split. Reuse the parent command/menu generation and the layout-preset catalog generation.

## `docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md`

Treat chrome/pointer generations as consumers of the indexed command/menu registry. Plugin owner
reload or revocation publishes one generation diff; unchanged rows and layout do no work.

## `docs/plans/zircon_runtime/runtime/09`

Provide the retained invalidation/generation receipt needed for one changed-menu refresh. Font/style
or shell metrics changes invalidate geometry only; unrelated presentation changes reuse it.

## Acceptance handoff

The owner handoff must include the 2/2 source fingerprint, managed focused tests, baseline/optimized
counters and WPR/ETW artifacts on D/E/F, real-window parity, milestone commit and quantified WeCom
notification. Shared ledgers remain protected until then.
