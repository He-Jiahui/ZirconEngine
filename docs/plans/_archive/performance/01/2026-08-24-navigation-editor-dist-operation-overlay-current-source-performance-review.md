---
title: Navigation Editor Dist Operation Overlay Current-Source Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/navigation/editor/src
  - zircon_plugins/navigation/dist/src
status: static_complete_m0_implemented_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/14-first-party-navigation-source-native-runtime-editor-dist-catalog-recast-detour-crowd-tilecache-query-bake-product-integration-review.md
  - docs/plans/optimize/zircon_editor/19-navigation-navmesh-settings-agent-area-surface-modifier-obstacle-off-mesh-link-bake-query-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/68-editor-scene-viewport-display-mode-lighting-skybox-show-flag-debug-visualization-overlay-composition-profile-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/49-runtime-debug-gizmo-command-buffer-retained-extract-filter-budget-render-product-integration-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Private/NavigationSystem.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Private/NavMesh/RecastNavMeshGenerator.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/Navigation/CrowdManager.cpp
---

# Navigation Editor Dist Operation Overlay Current-Source Performance Review

## 1. Coverage and execution truth

The post-M0 production scope is **18/18 Rust files**, **1,989 physical / 1,798 non-empty lines**, **68,480 bytes** and **2 inline tests**. The captured worktree is based on revision `436445c7cf3b6d60e60023c8968e2ffe3221e7da`; ordered fingerprint `aa5a4db3a836dceb47d255b44c5007282dae185d7bbd40a3876086840fd114c7`.

| Folder | Files | Static result |
|---|---:|---|
| `editor/src` root | 8 | Bake panel, mirror, overlay, provider and plugin shell reviewed. |
| `editor/src/operation_command` | 4 | Submit/poll/harvest, undo snapshot and error semantics reviewed. |
| `editor/src/plugin/registration` | 5 | Views, templates, components, operations and assets reviewed. |
| `dist/src` | 1 | Native distribution descriptor reviewed. |

## 2. Structural performance and product findings

### P0: Bake UI does not have a real asynchronous operation contract

`operation_command/command.rs:38-87` submits and then polls at most 16 times on the executing editor command, calling `std::thread::yield_now()` between polls. It cannot suspend/resume the transaction, express cancellation or survive a bake longer than this CPU-spin budget. The runtime navigation handler currently rejects Bake preparation with `navigation bake requires a pure prepare backend` (`zircon_runtime/src/navigation/operation/handler.rs:137-147`).

Editor tests hide this boundary: their gateway creates and applies the result during `submit_operation`, and every `poll_operation` immediately returns Completed. The standalone Bake panel uses a separate generic backend/progress model that is not the registered operation command. There are therefore two UI/operation concepts and no product Bake job.

### P0: overlay is rebuilt and copied through every layer

The runtime materializes every loaded asset's debug triangles and links into a new `NavigationOverlayFrame` on each subscribed frame. The editor mirror owns the entire latest frame behind `Arc<Mutex<_>>`. The viewport provider locks the mirror while calling `build_navigation_overlay`, which then allocates three line segments per triangle, a line/pick shape per link, agent path segments and velocity vectors. Before M0, this path also cloned the complete triangle/link arrays; the implemented M0 removes that intermediate copy for the default all-visible case but does not solve the cross-layer frame rebuild.

No generation cache, selected tile/agent, viewport bounds, update frequency or primitive/byte budget exists. `NavigationPieMirror::agent` is a linear search. Demand gating avoids all work with zero subscribers, but one subscriber enables the maximum payload every frame.

### P1: Dist advertises capability without carrying behavior

`dist/src/lib.rs` is declared stateless with no command invocation, state, unload hook or bridge method. Diagnostics say navmesh/Recast services remain hosted by the source runtime module. This is a registration marker, not a native product implementation, while package manifests advertise runtime/navigation/Recast capability for client, server and editor host.

### P1: undo snapshots clone generated assets

Navigation operations serialize before/after `NavigationGeneratedBakeSnapshot` values through JSON and retain both in command history. Large generated assets can be cloned/serialized multiple times during snapshot, prepare, result, journal, undo and redo. The operation should reference immutable generation-qualified artifacts plus a transactional receipt.

## 3. Unreal source constraints

- `NavigationSystem.cpp:1633-1811` treats dirty update, async build and task telemetry as persistent ticked state rather than a short synchronous polling loop in an editor command.
- `RecastNavMeshGenerator.cpp:8241-8345` maintains pending/running tile tasks and applies completed tiles on later ticks; user-facing completion follows actual job state.
- `CrowdManager.cpp:43-83` gates detailed debug work by selected agents or explicit debug variables, and `:232-320` instruments debug-relevant crowd phases separately.

The editor should observe generation/job state, render a bounded retained debug artifact and commit a receipt into history. It should not own the worker wait or serialize whole generated assets into each history entry.

## 4. Dependency-ordered optimization plan

### M0: remove redundant default overlay DTO clone (implemented; dynamic acceptance pending)

When both navmesh-area and off-mesh-link options are enabled (the default and normal provider path), `build_navigation_overlay` now projects the borrowed `NavigationGizmoSnapshot` directly. The filtered temporary remains only when one category is disabled. The default-path intermediate copy changed from `T` triangle DTOs plus `L` link DTOs to zero; required output construction remains `3T + L + agent debug` primitives. `navigation_overlay_category_filters_are_independent` fixes the all-visible, mesh-only, link-only and hidden behavior contract.

### M1: unify Bake into one persistent job protocol

Replace the 16-poll loop and duplicate panel backend with submit -> persisted job handle -> progress events/state -> cancellable completion -> transaction commit. Closing a document/session or superseding settings must cancel/join or explicitly detach a durable job. The command history receives a generation receipt only after commit.

### M2: retain generation-qualified overlay artifacts

Transmit immutable mesh debug geometry once per owner generation. Send small agent/debug deltas at a bounded frequency. Cache line/pick buffers by generation/options and rebuild only changed categories. Filter by selected agent, visible bounds, tile, area and primitive/byte budget before serialization.

### M3: make capability declarations truthful

Dist either carries/negotiates the actual navigation provider lifecycle or declares itself a host-carried registration manifest without implying standalone behavior. Editor controls must disable with explicit diagnostics when Bake/query/debug providers are unavailable.

### M4: qualify editor responsiveness

Measure Bake submit/UI event latency, queue/progress/cancel latency, history bytes, mirror deserialize/copy bytes, lock wait/hold, overlay build/upload time and primitives for `1k/100k/1m` triangles and `1/100/10k` agents. Report editor frame p50/p95/p99, CPU, wakeups, RSS and power with overlay off/on/selected-only.

## 5. Acceptance gates

1. No editor command polls or waits for a navigation worker on the UI thread.
2. Bake progress/cancel/completion survives frames and is tied to one runtime generation/job receipt.
3. Stable overlay generations perform zero mesh DTO clone/serialization and reuse retained draw artifacts.
4. Debug extraction is selected/bounds/frequency/primitive/byte budgeted.
5. History retains generation receipts, not duplicate complete generated assets.
6. Dist/editor capability diagnostics match the actually loaded provider.
7. Current-source editor WPR/ETW and RenderDoc captures pass timing, power and draw/upload budgets before acceptance.

## 6. Validation status

- Per-production-Rust-file static review: **18/18 complete**.
- M0: implemented after this plan was recorded; default-path mesh/link DTO clone count changed from `T + L` to `0`.
- Static checks: targeted `rustfmt --check`, `git diff --check` and source-branch invariants passed.
- Behavioral coverage: one focused unit test added, but not executed in this session.
- Cargo/tests: pending because the managed Windows validation session is unavailable.
- WPR/ETW and RenderDoc: pending because no launchable current-source executable exists and the tools were not found on the current PATH/tool roots.
- Protected ledgers, commit and WeCom completion remain pending.
