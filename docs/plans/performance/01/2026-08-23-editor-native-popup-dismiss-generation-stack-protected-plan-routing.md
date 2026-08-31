---
title: Editor native popup dismiss generation stack protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-native-popup-dismiss-generation-stack-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Replace the popup-dismiss portion of the obsolete 2026-07-17 combined coverage with one concise
`pending.md` module entry:

`zircon_editor retained-host host_contract/native_popup_dismiss.rs + native_popup_dismiss/**`
- 3/3 current Rust files source-reviewed. Overlay precedence, trigger/popup containment and bounded
  damage are retained. M0 coherent generation plus indexed borrowed discovery is applied and
  statically GREEN. Pending M1-M3 and M0 dynamic acceptance: typed top-popup/dismiss artifact shared
  by keyboard/hit/paint, telemetry and current-source scale/repeat/WPR/UI acceptance.

Do not add these files to `review.md` before M0-M3 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M3 to MVP pointer latency. Record outside-press requests, indexed candidates, node visits,
state/node clones, popup artifact builds/rebinds/reasons, contains probes, damage area/regions,
pointer CPU p50/p95/p99, WPR CPU/context switches/power and exact source/workload fingerprints.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of per-event raw-node popup discovery and independently reconstructed dismiss geometry.
No compatibility path may re-read window state after the input generation is selected.

## `docs/plans/zircon_editor/editor_layout/19-focus-and-navigation-model.md`

Own active-popup stack order, top-popup policy, nested dismissal, focus-loss behavior and exact
trigger/popup containment.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own coherent generation propagation and the shared popup artifact/cancel binding used by keyboard,
hit, dismiss and paint.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own reusable runtime popup stack, arranged containment/damage projection and reason-coded lifecycle
telemetry without editor-only duplicate popup ownership.

## Acceptance handoff

The handoff requires 3/3 post-change fingerprints, focused and managed Rust tests, node/depth/press/
identity/update/scale/repeat matrices, same-executable WPR artifacts on D/E/F, relevant GPU popup
parity, milestone commit and quantified WeCom notification. Protected ledgers remain unchanged until
then.
