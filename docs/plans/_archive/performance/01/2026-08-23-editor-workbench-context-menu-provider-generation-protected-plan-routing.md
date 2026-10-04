---
title: Editor workbench context menu provider generation protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-workbench-context-menu-provider-generation-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Replace the context-menu portion of the obsolete 2026-07-17 combined coverage with one concise
`pending.md` module entry:

`zircon_editor retained-host host_contract/workbench_context_menu.rs + workbench_context_menu/** + workbench_context_menu_tests.rs`
- 6/6 current Rust files source-reviewed. Resolved-hit input, bounded provider classification and
  popup-row rejection are retained. M0 single-buffer target URI is applied and statically GREEN.
  Pending M1-M3 and M0 dynamic acceptance: typed immutable plugin provider/action registry; one
  retained-subtree context-menu surface patch; plugin-scale/WPR/UI/GPU acceptance.

Do not add these files to `review.md` before M0-M3 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M3 to MVP editor secondary-input/plugin latency. Record lookup probes, provider owners,
sections/actions, predicate CPU/allocations, URI/menu allocations, property mutations, tree/subtree
visits, refresh reasons/changed nodes, open CPU p50/p95/p99, WPR CPU/context switches/power and exact
source/workload fingerprints.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of string-prefix provider authority, encoded menu-row contracts, click-time plugin
discovery, fragmented bridge mutation and per-open surface-tree lookup.

## `docs/plans/zircon_editor/editor/12-plugin-management.md`

Own immutable context-menu provider/action registration generations, stable owner ids, unload cleanup,
ordering, enable/visibility predicates and per-provider performance budgets.

## `docs/plans/zircon_editor/editor_layout/08-plugin-page-interface-and-messaging.md`

Own typed plugin contribution messages and the compact context snapshot passed from the resolved hit
to registered context-menu providers.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own typed resolved-hit provider keys, compact invocation context and one context-menu state patch into
the retained workbench surface.

## `docs/plans/zircon_editor/editor_layout/11-data-binding-and-reactive-contract.md`

Own retained context-menu control/subtree handles, atomic property patching, one scoped refresh and
shared popup generation. This is the dominant current open/close cost found by this review.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own reusable typed menu descriptors/state patches and retained subtree mutation contracts without
editor-only encoded string parsing.

## Acceptance handoff

The handoff requires 6/6 post-change fingerprints, focused and managed Rust tests, UI-node/plugin/
contribution/selection/provider/URI/update/scale matrices, same-executable WPR artifacts on D/E/F,
relevant GPU popup parity, milestone commit and quantified WeCom notification. Protected ledgers
remain unchanged until then.
