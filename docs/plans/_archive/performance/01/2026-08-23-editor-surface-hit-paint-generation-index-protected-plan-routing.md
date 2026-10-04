---
title: Editor surface hit and paint generation index protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-surface-hit-paint-generation-index-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Replace the obsolete 2026-07-17 coverage with one concise `pending.md` module entry:

`zircon_editor retained-host host_contract/surface_hit_test/**`
- 18/18 current Rust files source-reviewed. Generation-owned cell-bounded point hits, indexed popup
  rows and partial rebind foundations are retained. M0 single-pass pane surface build and M0b borrowed
  popup-candidate reuse for keyboard discovery are applied and statically GREEN. Pending M1-M3 and
  M0/M0b dynamic acceptance: split hit/paint owners with build/memory telemetry; allocation/sort-free
  multi-cell paint ranges and subtree intervals; converge arranged generation ownership and run
  current-source scale/WPR/UI acceptance.

Do not add these files to `review.md` before M0-M3 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M3 to MVP editor input latency and damage-bounded painting. Record hit builds/rebinds,
point candidates/path depth, paint models/builds, cells/bucket entries, query cells/raw/unique rows,
duplicates, allocations/sorts, subtree visits, CPU/bytes/context switches/power and source/workload
fingerprints.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own the hard separation of hit-route and paint-invalidation lifecycles and deletion of normal current-
generation linear fallback paths. No compatibility owner may keep rebuilding the combined index.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own the arranged presentation generation, stable model identity/version, O(1) paint-index selection,
popup/workbench/pane projection parity and partial replacement rules.

## `docs/plans/zircon_editor/editor_layout/19-focus-and-navigation-model.md`

Own persistent hit membership, bubble ancestry, popup/clip/z-order/disabled behavior and bounded point-
input candidates without event-time reconstruction.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own reusable runtime surface hit/paint projections, cached paint ranges, invalidation reason codes and
query scratch ownership. Do not expose editor-only duplicate spatial trees as the permanent contract.

## Acceptance handoff

The handoff requires 18/18 post-change fingerprints, focused and managed Rust tests, node/pane/window/
plugin/frame/clip/update/input/damage/scale matrices, same-executable WPR artifacts on D/E/F, GPU
RenderDoc parity where applicable, milestone commit and quantified WeCom notification. Protected
ledgers remain unchanged until then.
