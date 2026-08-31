---
title: Editor host page overflow arranged generation protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-host-page-overflow-arranged-generation-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Replace the host-page-overflow portion of the obsolete 2026-07-17 combined coverage with one concise
`pending.md` module entry:

`zircon_editor retained-host host_contract/host_page_overflow_menu.rs + host_page_overflow_menu/**`
- 4/4 current Rust files source-reviewed. Preprojected widest-title width, O(1) extent/range math and
  strict clipped/gutter behavior are retained. M0 one-candidate row hit is applied and statically
  GREEN. Pending M1-M3 and M0 dynamic acceptance: committed page-overflow layout/navigation artifact;
  pointer/keyboard/paint cutover; page/repeat/WPR/UI/GPU acceptance.

Do not add these files to `review.md` before M0-M3 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M3 to MVP tab input/paint latency. Record artifact builds/rebinds/reasons, geometry/content
queries, hit candidates/frame probes, keyboard row builds/page visits/cloned bytes, visible/painted
rows, input CPU p50/p95/p99, WPR CPU/context switches/power and exact source/workload fingerprints.

## `docs/plans/performance/01/2026-08-22-editor-ui-workbench-page-tabs-single-strip-layout-authority-architecture-review.md`

Extend its immutable page-strip layout target with overflow popup/viewport/scrollbar frames, hidden
identity, page-to-row lookup and uniform row geometry. Do not create a separate invalidation owner.

## `docs/plans/zircon_editor/editor_layout/15a-page-tab-strip-overflow.md`

Own page-strip/overflow topology, strict viewport and gutter behavior, scroll/active-page reveal and
the shared arranged artifact contract.

## `docs/plans/zircon_editor/editor_layout/19-focus-and-navigation-model.md`

Own retained overflow keyboard rows/current index and O(1) page-to-hidden-row navigation without
event-time vector construction.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own generation publication and pointer/keyboard/paint borrowing of the page-overflow artifact.

## Acceptance handoff

The handoff requires 4/4 post-change fingerprints, focused and managed Rust tests, page/viewport/
width/press/scroll/repeat/update/scale matrices, same-executable WPR artifacts on D/E/F, relevant GPU
overflow parity, milestone commit and quantified WeCom notification. Protected ledgers remain
unchanged until then.
