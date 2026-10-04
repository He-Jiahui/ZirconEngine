---
title: Editor floating-window projection generation receipt spatial scale protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-floating-window-projection-generation-receipt-spatial-scale-architecture-review.md
---

# Protected plan updates

This record requests owner-plan changes without overwriting shared ledgers or numbered plans.

## `docs/plans/performance/pending.md`

Keep one concise owner entry:

`zircon_editor/src/ui/retained_host/floating_window_projection.rs` - 1/1 Rust file source-reviewed;
M0 counters, mutation-owner receipts, shared geometry artifact, pointer O(1) cursor, F-scale WPR,
allocation, power and real-window parity remain pending.

Do not add the file to `review.md` until M0-M5 pass dynamically.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Track this after basic no-floating editor startup/input paths as MVP-P1. Require counter baselines
before receipt work and prohibit a per-recompute fake generation. Stable slow recomputes must reuse
one floating artifact across main/native/pointer consumers.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Route M2-M4 as a persistent-window/invalidation-root cutover. Delete value-reconstruction paths only
after exact source receipts and all consumers use the shared artifact; preserve no compatibility
geometry facade.

## `docs/plans/zircon_editor/editor_ui/08`

Own the composite Workbench layout/floating receipt and shell-pointer integration. Coordinate with
the existing shell-pointer single-release and host-scene single-generation records instead of
creating parallel generations.

## `docs/plans/zircon_editor/editor_window/13`

Make the native-window registry publish a bounds/tree-id/host-presence generation and changed-window
receipt. A consumer must not rescan or clone every native host to discover that no window changed.

## `docs/plans/zircon_runtime/runtime/09`

Ensure Runtime UI surface invalidation can accept a producer generation and skip unchanged geometry
publication without rebuilding the retained hit tree.

## Existing performance owner records

Merge pointer M3/M4 with
`2026-08-23-editor-retained-shell-pointer-single-release-receipt-architecture-review.md`; merge scene
artifact M2 with
`2026-08-22-editor-ui-workbench-host-scene-single-generation-projection-architecture-review.md`.

## Acceptance handoff

The plan owner should receive the 1/1 fingerprint, M0 counter schema, F/H reconciliation matrix,
managed tests, same-build WPR/allocation/power artifacts, real-window evidence and any render-impact
RenderDoc capture. Commit and quantified WeCom notification wait for accepted dynamic evidence.
