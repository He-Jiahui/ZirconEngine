---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: asset-management-generation-projection
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/management.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/resource_manager_contract.rs
  - zircon_runtime/src/asset/pipeline/manager/resource_streamer_accessors
  - zircon_runtime/src/asset/assets/scene/asset.rs
  - zircon_runtime/src/asset/assets/scene/management.rs
tests:
  - cargo test -p zircon_runtime --lib asset::tests::pipeline --locked --jobs 1 -- --nocapture --test-threads=1
  - stable polling, one-percent delta, query page, selected detail and scene entity matrices
---

# Runtime04：asset management generation投影缺失

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime asset pipeline逐Rust文件性能审查，PERF-MVP-500
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：compact rows、kind/source/issue索引与summary必须随registry/resource generation原子发布，不能由Editor每帧重扫或维护第二套资产权威。
- 生命周期键：`asset-management-generation-projection`

## 失败现象与复现证据

kind查询每次全registry scan+sort；完整record sets为多类资产重复scan并深clonepayload。scene record和scene entity record分别加载/投影同一scene；overview、family summary、status与issue只需聚合却先物化全部详情。resource list排序在比较器中反复`primary_locator.to_string()`，让稳定Editor轮询持续产生全量clone/sort/String分配。

PERF-MVP-519已把scene overview、scene summary和entity summary的17/18次row扫描收敛为单遍，并删除entity list先建scene aggregate再clone rows的中转；但每entity overview仍clone完整direct-reference Vec只取数量，scene record仍内嵌全部entity rows，稳定consumer仍可重复构建/复制宽投影。PERF-MVP-520要求这些rows、counts与reference indices归入同一scene generation。

## 最低共享层根因

registry/resource generation只发布底层entry与payload，没有面向管理查询的compact rows、稳定ordered views、delta索引、summary counter和selected-detail边界。

## 架构修复验收

- generation发布时增量维护compact management rows、kind/source/issue/stability索引和summary counters。
- Editor09缓存`generation + query + page`，stable 60Hz不重建全量record sets；详情仅按visible/selected id懒取。
- scene summary/detail/entities共享一次generation parse/load，禁止同请求重复完整scene projection。
- scene/entity rows与aggregate counters随scene generation一次发布；reference count读取compact index，不构造AssetReference Vec，entity page不复制全scene rows。
- resource list使用借用stable order key/locator直接比较，不在sort comparator分配String。
- assets/scenes/entities 1/1k/100k、stable/1% change/page记录scans、sort/key alloc、deep-clone bytes和p95：stable build/sort/clone=0，changed近delta+page。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止Editor私建另一套全registry索引或定时全量cache refresh。
- 禁止把深clone推迟到另一个helper但仍为summary/list创建全部详情。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.

2026-07-30 retained-host startup caller补充：`sync_asset_workspace`首帧同步调用`ResourceManager::list_resources()`；当前实现克隆registry全部`ResourceRecord`并以`primary_locator.to_string()`排序，随后把完整Vec交给runtime state。即使catalog已同步，这一步仍独立产生全表clone、排序和locator String，启动成本随N增长。Runtime04应把compact rows与stable ordered view随resource generation发布，retained host只传generation/Arc或MVP visible page；不得在Editor缓存第二份registry。验收增加startup/stable调用的registry scans、record deep-clone bytes、sort/key String bytes：warm unchanged必须为0，changed近delta+visible page。证据：`docs/plans/performance/01/2026-07-30-editor-retained-host-startup-current-review.md`；无动态pass声明。

2026-07-30 retained-host tick补充：任意非空resource event batch都会再次调用完整`list_resources()`，随后`EditorState::sync_resources`把每条locator重新格式化为String并从零构建map，同时无差别标记render/presentation dirty。现有256项/600us drain slice不约束这段O(N log N)+O(N)消费；持续resource backlog可每tick重建。generation projection必须直接发布共享stable ordered rows/map与affected-domain invalidation，stable batch全表clone/sort/map build=0；记录batch apply wall、registry scans、locator bytes、map entries和不相关render invalidation。证据：`docs/plans/performance/01/2026-07-30-editor-retained-host-assets-current-review.md`；无动态pass声明。

### 2026-08-01 current-source implementation

- `core::resource` now owns the immutable management generation, stable ordered
  compact rows, lookup indexes, summary counters and a bounded shared event
  stream. Mutation paths publish the projection with the resource generation;
  facade/manager consumers borrow that snapshot instead of rebuilding a second
  registry truth.
- The exact source was sealed as snapshot `1411`. Managed ticket
  `608273936187445f955af3f6bbe24e89` was accepted and remains receipt-only
  evidence; no terminal Cargo pass is inferred from its queued state.

Open state: `实现完成，受管验证待回执`; accepted closeout remains deferred.

### 2026-08-14 current-source continuation

- `ProjectAssetManager::asset_ids_by_kind` now reads the published
  `ResourceManagementGeneration` through a typed kind query. It no longer scans
  the mutable registry or performs a caller-side ID sort for each management
  query.
- `SceneEntityAsset::overview` now obtains its direct-reference count from
  allocation-free owner methods on camera, mesh/LOD, and prefab payloads. The
  existing `direct_references()` API remains the authoritative materialized
  view; collider-shape references remain outside its established count scope.
- The typed asset record-set generation itself is still open: project manager
  state and its fenced publication owner are concurrently changing and must
  publish one immutable scene/entity projection before stable polling can claim
  zero record-set rebuilds. No managed Cargo terminal receipt or performance
  matrix has been obtained, so this handoff remains `open`.

### 2026-08-14 immutable asset projection boundary

- The missing owner is `zircon_runtime::asset::pipeline::manager::ProjectAssetManager`, not
  `core::resource` and not an Editor cache. `ResourceManagementGeneration` remains the lower
  typed resource input; the asset manager publishes an `Arc`-backed immutable asset-management
  generation containing only asset-derived compact rows, record-set summaries, and stable
  kind/source/issue lookup indexes. It must not add asset DTOs, diagnostics sinks, or renderer
  state to `core::resource`.
- Every project transition publishes the asset generation under the existing
  `project_generation_write` fence: resource sync and the active-project replacement complete
  first; the new asset projection is installed next; only then may
  `publish_project_generation` broadcast changes and wake consumers. Close installs the empty
  projection under that same fence. Watch commits follow the same order after either their
  reconciliation or incremental resource commit. Readers take the matching project-generation
  read boundary and retain the immutable snapshot; stable polling therefore has no registry scan,
  asset load, record-set rebuild, sort, or clone work.
- `RenderMaterialManagementRecordSet` is a renderer-owned, independently refreshed detail
  product. It must be composed by the graphics consumer with the immutable asset snapshot rather
  than captured in the project asset generation. Selected/visible detail remains a narrow lazy
  `ResourceId` lookup that verifies the snapshot generation before loading an asset payload; it
  cannot materialize all scene entities or all renderer-prepared materials to answer a page query.
- Reference basis: Unreal `AssetRegistryState::AddAssetData` / `UpdateAssetData` maintain the
  canonical entry and query indexes as one mutation, and
  `AssetRegistryStateTest.cpp::FEnumerateAssetsPerformance` exercises a million-entry filtered
  query. Bevy `bevy_asset::Assets` keeps generational storage and emits lifecycle events at
  insert/remove rather than rebuilding read models. Zircon adopts their publish-time indexing and
  event boundary, while retaining the project-generation fence already owned by
  `ProjectAssetManager`.
- This is an architecture decision and implementation contract, not a fixed return. The future
  source slice must add a generation identity, snapshot read API, open/close/watch publication
  coverage, and the declared stable/one-percent/page/detail performance matrix before this
  handoff can move out of `open`.

### 2026-08-14 implementation gap audit

- The current `ProjectAssetManager` state has project-generation fencing, project/source-path
  state, watchers, and subscribers, but no asset-management generation or immutable snapshot
  field. This is the correct lower owner for the missing projection; no Editor-side cache can
  substitute for it.
- `management.rs::asset_ids_by_kind` correctly starts from
  `ResourceManager::management_generation()`, but every public record-set method still repeats
  a kind scan and then eagerly loads each asset to reconstruct management records. Stable polling
  therefore still rebuilds asset rows and record sets instead of retaining a published `Arc`
  snapshot.
- The same file currently imports `RenderMaterialManagementRecordSet` and accepts it in
  `asset_management_record_sets_with_prepared_materials`. That keeps renderer-prepared material
  detail in the asset-manager aggregation path, contrary to the required graphics-consumer
  composition boundary.
- No source change was made in these concurrently modified asset-owner files. This audit narrows
  the next source slice to one ProjectAssetManager-owned immutable projection and confirms that
  the failure remains `open`; it is not validation or performance acceptance evidence.

### 2026-08-14 publication cut points

- `project_asset_manager/open_project.rs::open_prepared_project` is the open installer: it already
  holds `project_generation_write`, commits resource sync, replaces `*active_project`, and only
  then activates watchers and calls `publish_project_generation`. Install the new immutable asset
  snapshot after the project replacement and before watcher activation/publication.
- `project_asset_manager/close_project.rs::close_project` is the empty installer: it removes the
  project resources, clears source paths, assigns `*project = None`, and then publishes `Removed`
  changes under the same generation guard. Replace the asset snapshot with the empty generation
  between the project assignment and that publication.
- `project_asset_manager/runtime.rs::process_watch_batch_in_generation` is the sole watch
  installer: after either incremental or reconciliation resource sync succeeds, it assigns the
  candidate project and returns to `publish_project_generation`. Build/install the next asset
  snapshot in that successful branch before `commit_result` is observed as published. Do not add a
  second watcher-side cache or a post-publication rebuild path.

### 2026-08-14 reload transition audit hard cut

- The Runtime04 structural audit had retained a stale public-facade-only reload source view after
  `ResourceManager::start_reload` and `fail_reload` became thin transaction delegates. The state
  transition, runtime-slot update, failure event, and error-recovery rejection now live in
  `core/resource/manager/commit.rs`; that file is therefore a source owner, not a compatibility
  fallback.
- The source inventory, Rust absorption inventory, and boundary reader now count 26 current
  Runtime04 source files and read `registry_ops.rs`, `commit.rs`, and asset resource sync together.
  The anchors pin the delegate boundary, `Ready|Reloading|Error` entry rule, transition/runtime
  update, `ReloadFailed` event, error recovery rule, and imported-resource commit without
  reintroducing old facade mutations.
- Static evidence: `python -B tools/tests/test_runtime_asset_pipeline_audit.py` passed 2/2 on
  2026-08-14; `rustfmt +1.94.1 --check
  zircon_runtime/src/tests/runtime_absorption/asset_pipeline/inventory.rs` passed; and Python
  compilation of the updated audit scripts and test passed. No Cargo command was started while the
  shared validation lane is reserved. This keeps the handoff `open`: it repairs the current-source
  audit root only and does not implement or validate the required ProjectAssetManager immutable
  generation.
