---
related_code:
  - zircon_editor/src/ui/workbench/window_registry
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/host/workspace_state.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
owner_plans:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/zircon_editor/editor_layout/07-windowing-chrome-tabs-and-dockable-drawers.md
  - docs/plans/zircon_editor/editor_layout/08-plugin-page-interface-and-messaging.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-layout-metadata-full-recompute-and-sync-io.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Docking/TabManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/TabManager.cpp
---

# Protected plan routing: Workbench window registry dead mirror

## Reason for routing

The main performance plan, `pending.md`, `review.md`, Optimize01/13/50 and numbered owner plans are
protected or foreign dirty. This record routes the 9/9-file current evidence without editing those
authorities. Detailed evidence source:
`2026-08-19-editor-ui-workbench-window-registry-dead-mirror-layout-generation-architecture-review.md`.

## Requested Performance01 updates

### PERF-MVP-077

Upgrade the window registry finding from generic full rebuild to a proven dead production mirror.
`EditorUiHost.window_registry` has initialization and replacement writes but zero product readers.
Every changed metadata transaction still clones all `ViewInstance` payloads and rebuilds the unused
registry. Acceptance must require host registry build/read/resident bytes and payload clone bytes
`=0`, then apply typed layout deltas only to indexes with real consumers.

### PERF-MVP-097

Correct the earlier partial GREEN. `instances_by_id()` borrows rows only after
`recompute_session_metadata()` has deep-cloned all instances. The existing source guard therefore
does not prove the product path clone-free. Require end-to-end clone counters from session input to
native/presentation consumers, not a string assertion inside the second-stage helper.

### PERF-MVP-131 and PERF-MVP-602

Keep grouped drawer resize and multi-tab close acceptance tied to removal of this mirror. One resize
or close group performs at most one canonical layout transaction, typed metadata delta, native host
sync, persistence request and invalidation; no intermediate full registry rebuild is permitted.

## Requested Optimize and owner updates

### Optimize13 + EditorUI08

Hard-delete the unused host registry field/lock/init/rebuild and remove the upstream aggregate
instance clone. `WorkbenchLayout` owns persisted topology, session generation owns live instance
metadata, and native host generation owns OS handles. Layout apply publishes typed affected IDs;
focus is near O(1), dock/close touches the affected subtree, and only real consumer indexes exist.

### EditorLayout07

Rewrite the plan before S1/S2. Its current statement that `EditorWindowRegistry` is the unique fact
and should not be rewritten is false in current production: the field is never read, while layout,
session and `WindowHostManager` are authoritative. PurposeView, Chrome tabs and drawer transitions
must commit through canonical layout/session generations. Do not add more mutable methods or state
to the dead mirror.

### EditorLayout08 + Optimize50

Plugin pages enter through the extension owner generation, then transact with canonical layout and
session generations. Disable/reload/quiesce cannot leave a page in a separate window registry. A
missing plugin view becomes a bounded layout placeholder, not a live stale callback or orphan row.

### Optimize01

Retained/native presenters consume the published layout/window generation directly and reconcile
each changed window once. They must not request a rebuilt window registry or copy complete instance
payloads to discover one window/drawer relation.

## Requested protected index state

- `pending.md`: add or retain one concise module row for
  `zircon_editor/src/ui/workbench/window_registry/**`, 9/9 files, 481 lines, fingerprint
  `f5b043fdeceb...`, `source_recheck_required=true`, and
  `static_complete / dead_mirror_hard_cut_required / dynamic_pending`.
- `review.md`: do not add the module. Require dead mirror removal, typed layout delta, corrected
  EditorLayout07 authority, current-source managed Cargo/F4 and WPR/ETW CPU/allocation/lock/power.
- Keep protected indexes module-level and concise; detailed evidence remains in the companion review.

## Acceptance handoff

| owner | required proof |
|---|---|
| Optimize13 + EditorUI08 | host registry build/read/resident/payload-clone bytes `=0`; stable/no-op work `=0`; focus near O(1), dock/close affected-subtree bounded |
| EditorLayout07 | updated authority contract; PurposeView/tab/drawer operations use canonical layout/session generations and cannot create a third mutable registry |
| EditorLayout08 + Optimize50 | plugin page register/revoke/reload reconciles extension/layout/session/native generations atomically; stale page/callback/orphan `=0` |
| Optimize01 | one presenter/native reconcile per changed window generation; unchanged window visits/copies/OS calls `=0` |
| Performance01 | 31-run F4 focus/dock/resize/close/restore WPR/ETW CPU, allocation, lock, input latency, RSS and package-power matrix; artifacts on D/E/F |

RenderDoc remains conditional on rendering-visible changes. It cannot replace WPR/ETW proof for the
dead rebuild, aggregate JSON copies, lock cost or package power.
