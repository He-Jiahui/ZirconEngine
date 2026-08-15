---
related_code:
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh
  - zircon_editor/src/ui/host/editor_asset_manager
  - zircon_editor/src/ui/asset_editor/preview
  - zircon_editor/src/ui/retained_host/app/assets
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_runtime/shader/05-ide-and-authoring-dx.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
doc_type: implementation-evidence
status: routing_blocked_by_protected_plan_owner
created_at: 2026-08-15
---

# Editor asset refresh/preview protected-plan routing evidence (2026-08-15)

## Coordinator decision

The 78/78-file review is complete in
`2026-08-15-editor-asset-refresh-preview-current-architecture-review.md`. The current freeze is
10,186 physical lines, 62 inline tests and manifest
`671ed66386f72066c34c161734894c604bcfde6bec399d88ee847eb0708f8724`.

Performance01 requested all owner writes on 2026-08-15. The coordinator returned
`protected_plan_definition` for both numbered performance plans and all numbered Editor/Runtime
plans, and `outside_registered_child` for `performance/pending.md` and `performance/review.md`.
This Session is restricted to `docs/plans/performance/01/**`; it did not bypass those boundaries or
touch either foreign-dirty index.

This is routing evidence, not a terminal `failure-*` handoff. Useful static review can continue while
the product build baseline is repaired; protected merges and dynamic acceptance remain pending.

## Required owner merges

### Performance main plan

Add `PERF-MVP-637` as P0 after the pending PERF-MVP-636 renderer artifact convergence task and link
it to PERF-MVP-357/358/404/623/626/627/633/636:

| ID | Priority | Current root cause | Required hard cut | Acceptance summary |
|---|---|---|---|---|
| PERF-MVP-637 | P0 | UI asset direct/save/watcher paths are separate refresh authorities; worker completion reparses/recompiles under dependency/session locks; editor refresh clones ProjectManager then captures/scans all records, rebuilds reference/folder/detail/catalog generations, and synchronously writes shader IDE artifacts; each thumbnail completion copies O(N) catalog rows and retained preview refresh rebuilds chrome; stable UI asset presentation duplicates state-graph/projection work | Plan02 M1/M4, Runtime04/09/11, Editor09, EditorUI05/08, Shader05 and Render08 hard-cut `RuntimeAssetSourceDelta -> EditorAssetIndexDelta -> UiAssetImportGeneration -> UiAssetCompiledPreviewGeneration -> ThumbnailGeneration -> RetainedAssetSurfaceDelta`, with shader IDE as a shared-source artifact branch. Runtime registry owns incremental indexes/journal; TaskGraph owns keyed single-flight and count/time/byte/age/deadline budgets; main thread only swaps generations and invalidates exact rows/panes. Delete all direct/full-rebuild/embedded-preview/chrome-discovery/duplicate-graph authorities in the replacing milestones | 78/78 static manifest retained pending. Stable clone/scan/I/O/parse/compile/rebuild/copy=0; one-asset work proportional to touched rows+affected edges+folder depth, not total N; main apply is budgeted swap only; thumbnail queue has count/source/decoded/resident-byte/age budgets and targeted row completion; one preview graph build per input generation. Current Cargo plus F0/F4 WPR/xperf, RSS/energy, GPU timestamps/RenderDoc and functional save/reload/cancel gates pass |

Do not merge this task as a local “make refresh async” change. PERF-MVP-627 must first provide the
shared scheduler and PERF-MVP-636 the canonical shader/artifact identity; PERF-MVP-637 owns removal
of editor-local duplicate generations and presentation publication.

### Plan02 M1 and M4

M1 must include asset/import/thumbnail/IDE artifact domains in the single TaskGraph: keyed
single-flight, dependencies, priority/affinity, queued/source/decoded/resident-byte and age budgets,
cancellation/currentness, bounded main-thread completion and deadline shutdown. Editor-private jobs
must not survive as a second scheduler.

M4 must add the editor-side generation chain after Runtime04/11 exists:

1. editor/runtime acquire immutable project/catalog handles; no `ProjectManager` value clone or
   editor full-registry capture is allowed in refresh;
2. catalog row/detail/folder/reference indexes update affected keys/ranges and publish through stable
   slots or structural sharing; preview state is a separate keyed generation;
3. UI asset save, direct edit and watcher changes enqueue the same generation key; worker output is
   already import-resolved and preview-compiled, so commit is swap plus exact invalidation;
4. retained layout/virtualization owns visible asset UUIDs; thumbnail completion never calls
   `build_chrome` or republishes the complete catalog;
5. UI asset pane presentation is cached by document/import/mock/selection/surface generations and
   builds the state graph once for inspector and mock fields;
6. all legacy APIs and dual writes are deleted in the milestone, with no alias or compatibility shim.

M3/Render08 and Shader05 must expose the canonical shader source artifact used by runtime material,
plugin, IDE and preview. Shader IDE generation becomes a bounded async artifact and performs no I/O
under the editor source-sync gate.

### Performance pending/review indexes

Add one concise pending module entry per folder, not 78 file rows:

- `zircon_editor/src/ui/host/asset_editor_sessions/refresh/**`: 15/15 static reviewed, dynamic pending;
- `zircon_editor/src/ui/host/editor_asset_manager/**`: 45/45 static reviewed, dynamic pending;
- `zircon_editor/src/ui/asset_editor/preview/**`: 8/8 static reviewed, dynamic pending;
- `zircon_editor/src/ui/retained_host/app/assets/**`: 10/10 static reviewed, dynamic pending.

Link the evidence report, line/test counts and manifest. Keep all four out of `review.md` until
current-source Cargo, hard-cut functional/complexity gates and F0/F4 WPR/xperf/RenderDoc/energy
evidence pass. At that point the owner moves the same four module-level entries atomically from
pending to review.

### Editor plan owners

- Editor09: replace the existing bounded-watcher/full-catalog failure entries with the single runtime
  delta/editor index/UI compile generation chain. Preserve bounded coalescing/retry; remove direct
  refresh, broad dependency lock, main-thread compile, cloned source generations, full reference/
  folder/detail publication and embedded preview rows.
- EditorUI05: retain visible UUIDs in the virtualized layout generation; consume row/folder/preview/
  UI-pane typed deltas and target row/paint invalidation. Preview completion cannot rebuild chrome.
- EditorUI08: bind the asset surface to the same pre/layout/post/paint invalidation transaction used
  by the retained workbench; stable preview and catalog changes do not trigger full host recompute.
- Editor14: background UI import/thumbnail/IDE work must use Runtime11 tickets and bounded completion;
  it does not own a private scheduler or block save/close/Drop.

### Runtime/render plan owners

- Runtime04: immutable runtime asset catalog handle, typed change journal, dense generational slots and
  incremental UUID/locator/kind/folder/dependency/reverse-dependency indexes.
- Runtime11: shared keyed task DAG with count/time/byte/age/deadline admission and instrumentation;
  no UI/import/thumbnail/IDE private worker pool.
- Runtime09: immutable UI compile/presentation artifact contract and exact dirty-domain publication;
  no runtime/editor duplicate UI document authority.
- Shader05/Render08: canonical source artifact and dependency DAG shared by runtime, plugin, IDE and
  preview; IDE persistence is explicit async work with stale cancellation.
- Render17/Editor profiling owner: counters for clone/capture/visit/edge/folder/catalog bytes, UI
  compile and lock time, thumbnail queue/decoded bytes/I/O/apply, retained chrome/full recompute and
  presentation cache; WPR/xperf plus RenderDoc/GPU timestamp/energy acceptance.
- Optimize indexes: treat container/sort/string tweaks as post-hard-cut work. Do not optimize the
  current full-capture, embedded-preview or direct-refresh owners as permanent architecture.

## Completion condition

This routing record can be retired only after protected owners merge the plan/index updates, M0
restores a current product baseline, the hard cut deletes all legacy authorities, and the cited
Cargo/complexity/WPR/xperf/RenderDoc/energy gates pass. Until then the module is
static-complete/dynamic-blocked and no milestone commit or WeCom completion message is permitted.
