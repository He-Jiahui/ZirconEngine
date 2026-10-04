---
title: Editor native pointer scroll transition batching protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-native-pointer-scroll-transition-batching-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Replace the obsolete 2026-07-17 native-pointer scroll portion with one concise `pending.md` entry:

`zircon_editor retained-host host_contract/native_pointer/scroll_dispatch.rs + scroll_dispatch/**`
- 19/19 current Rust files source-reviewed. One committed generation, popup/menu priority, boundary
  consumption, typed target dispatch and bounded bridge offsets are retained. Pending M0-M4: no-change
  redraw gates, typed replies/asset ids, atomic interaction patches, exact visible-range damage,
  WPR/power/RenderDoc acceptance.

Do not add these files to `review.md` before M0-M4 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M4 to MVP editor scrolling. Record route/callback/reply counts, consumed/changed reasons,
state clone/String allocation/publications, bridge preparation/rebuild, visible ranges, damage area,
CPU p50/p95/p99, WPR scheduling/power and exact source/workload fingerprints.

## `docs/plans/performance/01/2026-08-23-editor-native-pointer-move-transition-generation-architecture-review.md`

Share typed asset surface/list identities, one committed input generation and atomic interaction patch
semantics across move and scroll.

## `docs/plans/performance/01/2026-08-23-editor-pointer-redraw-result-region-promotion-architecture-review.md`

Consume typed scroll receipts and old/new visible owner ranges; preserve exact regions through redraw
and presenter.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own typed consumed/changed scroll replies, ordered precision-wheel facts and retained route owners.

## `docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md`

Own stable scroll owner ids and popup/menu/pane priority without String route identities.

## `docs/plans/zircon_editor/editor_layout/11-data-binding-and-reactive-contract.md`

Own atomic multi-field interaction patches and one generation publication per input fact.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own retained prepared scroll-surface generations, visible ranges and exact owner-frame projection.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own reusable typed scroll replies, virtualized visible-range receipts and reason-coded multi-region
invalidation shared with editor UI.

## Acceptance handoff

The handoff requires 19/19 post-change fingerprints, focused and managed Rust tests, wheel/sample/
row/route/offset/backend/scale matrices, same-executable WPR artifacts on D/E/F, RenderDoc GPU/
scissor/pixel parity, milestone commit and quantified WeCom notification. Protected ledgers remain
unchanged until then.
