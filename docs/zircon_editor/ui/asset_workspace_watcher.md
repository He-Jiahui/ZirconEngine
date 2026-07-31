---
related_code:
  - zircon_editor/src/ui/host/asset_editor_sessions/watcher.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/watcher
  - zircon_editor/src/ui/host/asset_editor_sessions/dependency_index/generation.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/imports/generation.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/imports/traversal.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/reconcile.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/pipeline/queue.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/pipeline/service.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/pipeline/job.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/pipeline/commit.rs
  - zircon_editor/src/ui/asset_editor/session/import_reference_access.rs
plan_sources:
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/09/failure-2026-07-17-ui-asset-watcher-unbounded-refresh.md
tests:
  - zircon_editor/src/ui/host/asset_editor_sessions/watcher/tests.rs
  - tools/tests/test_editor09_ui_asset_watcher_bounded_refresh_contract.py
doc_type: module-detail
---

# UI Asset Workspace Watcher

The UI asset workspace watcher retains filesystem notifications in a bounded latest-set keyed by
physical path. Repeated write/create notifications for the same path occupy one pending entry while
cumulative received and coalesced counts remain observable. The notify callback performs no file
I/O, parsing, editor-session traversal, or host refresh.

The ingress and overflow-reconcile enumeration phase of each retained-host poll has both an item
count budget and a wall-time budget. Unvisited paths and reconcile cursor state remain for later
ticks. Changed identities enter an `EditorJobSystem` `Index`/`Background` generation; filesystem
read, source parse, direct-session rebuild, and transitive import traversal run in that worker
rather than on the retained-host tick. The public poll result is a
`UiAssetWorkspaceWatchPollReport`, which carries changed `res://` identities and diagnostics for
pending paths, active reconcile cursor, received/coalesced counts, overflow generations,
oldest pending age, budget exhaustion, pending refresh assets, active worker state,
deferred/exhausted retry counts, and superseded generations.

Capacity overflow never treats a partial path set as authoritative. The partial generation is
discarded and the host starts a cursor over currently open UI asset routes and direct import roots.
The cursor borrows one route/import at a time through O(1) count/index access, shares the same poll
allowance, and releases the session lock between bounded batches; it never materializes the full
reconcile set. Existing import traversal then re-expands transitive imports, so the current
open-document consumers converge without adding a second asset inventory. Reconcile enumeration is
completed before ordinary ingress resumes.

Open UI documents are projected into one reverse-dependency generation containing direct route
edges and normalized import edges. Import edges are recorded before resolve/read/parse, so a
missing or invalid dependency remains targetable after it is repaired. All affected documents in
one worker generation share a canonical physical-path parse cache; each physical source is loaded
at most once for that generation.

The UI-thread commit validates project root, dependency generation, direct route identity, disk
baseline, and source fingerprint. Resolved imports, stale diagnostics, and reverse edges commit
under the `dependency_generation -> ui_asset_sessions` lock order. A same-tick newer notification
supersedes the completed worker result. True project-root transitions cancel and clear old work;
same-project save/restart preserves queued work and the existing watcher. Transient worker/source
or commit failures use per-asset exponential retry cohorts (50ms base, 2s cap, 6 attempts), while a
fresh filesystem event resets only that asset's retry state.

The implementation boundary is now hard-cut. Editor09 still requires source-bound Cargo tests,
1k/10k filesystem-storm memory/main-thread p95 evidence, independent zero-finding review, fixed
return, and managed commit before the parent performance failure can close.
