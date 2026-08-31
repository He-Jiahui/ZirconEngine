---
title: Editor host global interaction transaction protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-host-global-interaction-transaction-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Replace the obsolete 2026-07-17 globals portion with one concise `pending.md` entry:

`zircon_editor retained_host/host_contract/globals (16/16 current Rust files): M0 event-batched
interaction/no-op chrome/capture-result fixes; typed atomic patch/reply, WPR/power and RenderDoc
parity pending.`

Move it to `review.md` only after M0-M3 pass on one source/executable fingerprint. Detailed findings
remain in the owner report, not the protected ledgers.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Own event-level transaction/clone/generation/redraw counters and the 1K-Hz WPR/power matrix.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own typed callback replies and one interaction patch per native input fact.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own atomic host structure/interaction/viewport/hit/damage publication and stable ids.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Own no-op preflight, transaction coalescing and exact generation advance semantics.

## `docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md`

Own generation-coherent hit/paint state and exact callback dirty-owner results.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own reusable typed invalidation transaction/reply contracts aligned with retained UI generations.

## Acceptance handoff

The handoff requires 16/16 post-change fingerprints, focused and managed Rust tests, interaction
storm/retained-reader matrices, same-executable WPR artifacts on D/E/F, RenderDoc parity, milestone
commit and quantified WeCom notification. Protected ledgers remain unchanged until then.
