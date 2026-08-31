---
title: Editor native keyboard generation popup navigation protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-native-keyboard-generation-popup-navigation-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Replace the obsolete 2026-07-17 coverage with one concise `pending.md` module entry:

`zircon_editor retained-host host_contract/native_keyboard.rs + native_keyboard/**`
- 13/13 current Rust files source-reviewed. Closed command enum and virtualized command-palette
  window navigation are retained. M0 indexed borrowed popup discovery and allocation-free prefix
  probes are applied and statically GREEN. Pending M1-M3 and M0 dynamic acceptance: generation-owned
  typed navigation artifact shared by paint/hit/dismiss, telemetry and current-source scale/key-
  repeat/WPR/UI acceptance.

Do not add these files to `review.md` before M0-M3 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M3 to MVP editor input latency. Record target requests, indexed candidates, node visits,
node/row/string/frame clones, allocations, artifact builds/rebinds/reasons, key CPU p50/p95/p99,
search probes/bytes, window requests, interaction updates, WPR CPU/context switches/power and exact
source/workload fingerprints.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of event-time popup reconstruction and raw-node fallbacks after the generation artifact
lands. No compatibility helper may retain a second independently versioned popup navigation owner.

## `docs/plans/zircon_editor/editor_layout/19-focus-and-navigation-model.md`

Own retained active-popup/focus identity, typed row navigation, boundary behavior and interaction-only
active-index updates, following Slate's focus-path routing principle rather than whole-tree discovery.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own `HostPopupNavigationArtifact`, its generation/version rules, popup-window partial rebind and
shared paint/hit/dismiss/keyboard projections.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own reusable runtime popup navigation/focus contracts, arranged row descriptors, normalized search
keys and build/rebind reason telemetry without editor-only duplicate ownership.

## Acceptance handoff

The handoff requires 13/13 post-change fingerprints, focused and managed Rust tests, node/popup/row/
key-repeat/query/update/scale matrices, same-executable WPR artifacts on D/E/F, relevant GPU popup
parity, milestone commit and quantified WeCom notification. Protected ledgers remain unchanged until
then.
