---
title: Editor retained menu pointer surface generation protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-retained-menu-pointer-surface-generation-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Replace the obsolete retained-menu-pointer portion with one concise `pending.md` entry:

`zircon_editor retained_host/menu_pointer (28/28 current Rust files): M0 shared layout/stable
publication/popup-index retention; immutable menu generation, incremental surface suffix updates,
WPR/power and RenderDoc parity pending.`

Move it to `review.md` only after M0-M3 pass on one source/executable fingerprint. Detailed findings
remain in the owner report, not the protected ledgers.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Own menu build/reject/clone/reindex/surface-suffix/publication counters and WPR/power matrices.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own typed menu interaction receipts, stable ids and generation-coherent event dispatch.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own the one immutable menu/layout generation shared by chrome, paint and pointer consumers.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Own unrelated-invalidation reuse, changed-domain routing and zero stable publication.

## `docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md`

Own the retained open-menu stack, longest-common-prefix suffix update and hit/paint generation gate.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own atomic UI topology mutation across tree, dispatcher, route owner, arranged tree, hit grid and
render cache, with incremental rebuild reports.

## Acceptance handoff

The handoff requires 28/28 post-change fingerprints, focused and managed Rust tests, menu scale and
storm matrices, same-executable WPR artifacts on D/E/F, RenderDoc parity, milestone commit and
quantified WeCom notification. Protected ledgers remain unchanged until then.
