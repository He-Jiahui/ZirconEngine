---
title: Editor native pointer menu geometry generation-stack protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-native-pointer-menu-geometry-generation-stack-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Replace the obsolete 2026-07-17 menu-geometry portion with one concise `pending.md` entry:

`zircon_editor retained_host/native_pointer/menu_geometry (27/27 current Rust files): M0
borrowed-row/reset-state hard cut; generation-owned popup stack, suffix updates, bar index,
WPR/power and RenderDoc parity pending.`

Move it to `review.md` only after M0-M3 pass on one source/executable fingerprint. Detailed findings
remain in the owner report, not the protected ledgers.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Own the stable-event zero-build/zero-clone gate, generation-stack counters and scale/WPR/power
matrix. Supersede the old Slint-only reference with the Unreal-primary source basis.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own one topmost menu hit result per pointer fact and typed reply reuse across move/scroll/button.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own atomic menu structure/interaction/geometry publication from the Workbench menu bridge.

## `docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md`

Own the retained popup geometry stack, shared hit/paint/damage frames and top-bar interval index.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Own menu generation ids and longest-common-prefix suffix updates without a second open-state owner.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own reusable invalidation-driven popup stack and topmost hit-path contracts for runtime UI.

## Acceptance handoff

The handoff requires 27/27 post-change fingerprints, focused and managed Rust tests, scale/depth/
state matrices, same-executable WPR artifacts on D/E/F, RenderDoc parity, milestone commit and
quantified WeCom notification. Protected ledgers remain unchanged until then.
