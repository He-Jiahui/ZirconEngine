---
title: Editor Asset Workspace、Catalog、Provider、Preview、Import、Reimport 与 Runtime 边界当前工作树工程化差距
category: zircon_editor
report_id: Editor248
review_date: 2026-08-30
baseline_head: working-tree
related_code:
  - zircon_editor/src/core/asset
  - zircon_editor/src/core/project
  - zircon_editor/src/core/document
  - zircon_editor/src/core/jobs
  - zircon_editor/src/ui/host/editor_asset_manager
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/retained_host/app/assets
  - zircon_editor/src/tests/editing/ui_asset
  - zircon_runtime/src/asset/pipeline/manager
  - zircon_runtime/src/asset/project
tests:
  - zircon_editor/src/tests/editing/asset_workspace.rs
  - zircon_editor/src/tests/editing/ui_asset
  - zircon_editor/src/tests/editor_asset_type_registry
  - zircon_editor/src/ui/host/editor_asset_manager/manager/catalog_generation/tests.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/226-editor-asset-workspace-content-browser-current-source-review.md
  - docs/plans/optimize/zircon_editor/247-editor-scene-world-authoring-play-hierarchy-document-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/188-runtime-asset-resource-lifecycle-locator-registry-load-cache-import-cook-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/87-runtime-asset-reference-identity-locator-guid-subasset-redirector-rename-move-resolution-repair-migration-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/88-runtime-asset-watch-change-ingress-coalescing-rename-overflow-targeted-reimport-generation-reload-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/AssetRegistryState.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/SoftObjectPath.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/PrimaryAssetId.h
  - dev/bevy/crates/bevy_asset/src/server/mod.rs
  - dev/bevy/crates/bevy_asset/src/handle.rs
  - dev/bevy/crates/bevy_asset/src/io/mod.rs
  - dev/Fyrox/fyrox-resource/src/manager.rs
  - dev/Fyrox/fyrox-resource/src/loader.rs
  - dev/godot/editor/file_system/editor_file_system.cpp
  - dev/godot/core/io/resource_loader.cpp
  - dev/godot/core/io/resource_uid.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Material/AssetReimportUtils.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Importers/ShaderGraphImporter.cs
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Editor Asset Workspace、Catalog、Provider、Preview、Import、Reimport 与 Runtime 边界当前工作树工程化差距

## 1. 结论

Editor 已有可保留的产品骨架：`EditorAssetIndex` 能从 Runtime project snapshot 构造 metadata projection，并保留 dirty/importing/transient state；`EditorAssetCatalogGeneration` 使用 immutable `Arc` rows 和 UUID/locator/folder index；`EditorAssetManager` 已把 Runtime 删除、重定位、模型导入、preview refresh 接成 ticket；`PreviewScheduler` 有 visible/dirty/in-flight admission；asset type registry 已有 plugin owner、toolkit、thumbnail、creation template 和 context command 的 generation。

但是这些结构仍然集中在一个“默认项目 asset manager + 单一 workspace state + full catalog rebuild”的产品通道中。`EditorAssetIndex::rows()` 仍把 Runtime registry entries 全量收集成 Vec（`zircon_editor/src/core/asset/index.rs:174-180`）；`AssetWorkspaceState` 只有一个 selected folder/asset/query/filter（`zircon_editor/src/ui/workbench/project/asset_workspace_state.rs:43-77`），folder tree 递归 materialize，资产过滤是字符串线性扫描（`.../asset_workspace_state.rs:653-705`）；catalog record 只发布 `ResourceKind`，没有 exact `AssetTypeId`、schema、provider capability 或 source health（`zircon_editor/src/ui/host/editor_asset_manager/records.rs:18-64`）。

本轮没有新增独立 P0。Editor226 的 Locate 无 target、Browser activation 不可达、exact type 在 catalog 丢失，以及 Editor60/61 的 project/document/runtime generation 边界仍是先决阻断。本轮登记 32 项 P1、12 项 P2 和 25 个资格门，目标是将 asset workspace 收敛为 source/provider/action/operation/receipt 体系，而不是继续在 ZUI、retained host、preview job 和 Runtime watcher 之间添加一次性分支。

## 2. 审查边界与证据

### 2.1 当前 owner 链

| 层 | 关键文件 | 当前职责 | 本轮核验 |
|---|---|---|---|
| Runtime projection | `core/asset/index.rs`、Runtime `AssetRegistryIndex`/`ProjectManager` | catalog、meta、dirty/importing、reference/subasset | identity、generation、query、authoritative source |
| Manager/API | `ui/host/editor_asset_manager/{api,catalog,generation,records}.rs` | catalog/details/change API、preview/refactor/import tickets | action surface、provider boundary、receipt |
| Workspace | `ui/workbench/project/asset_workspace_state.rs`、asset tree/content pointer | folder/query/selection/view/utility projections | instances、history、keyboard/pointer activation、scale |
| Preview/import | `manager/preview_refresh`、`core/asset/import_flow` | bounded preview jobs、model import admission | cache key、cancel、type/variant、derived output |
| Project/document/jobs | `core/project`, `core/document`, `core/jobs`, retained assets app | project generation, dirty/reference preflight, operation execution | lifecycle, stale result, undo/redo, failure compensation |

### 2.2 证据等级

- **E3**：逐文件读取生产代码和 owner 调用链；声明存在 ticket 不等于 UI 已经可达。
- **E2**：读取 editor asset tests 与 Runtime188 资产生命周期结论，并对照本地 Unreal/Bevy/Fyrox/Godot/Unity Graphics 源码。
- **E1**：测试文件用于确定意图和静态覆盖；本轮不运行 Editor、真实 1M catalog、preview soak 或 import/reimport E2E。
- **E0**：不能据此宣称 Editor 已经达到 Unreal/Unity 的交互、吞吐或恢复能力。

## 3. 可保留底座与当前接线

1. `EditorAssetIndex::from_runtime_project` 以 Runtime registry + `ProjectCatalogInputGeneration` 为输入，避免 Editor 私自扫描文件；这个 authority 方向必须保留。
2. `EditorAssetCatalogGeneration` 把 rows/details/index 作为 generation 发布，preview completion 通过 COW 更新单行；这比可变全局 map 更接近正确的读模型。
3. `EditorAssetImportFlow` 的 admission、dedup、cancel、diagnostics 和 model ticket 是 operation kernel，但当前仍是 model-only compound flow。
4. `AssetDeletePreflight`、`EditorAssetRelocationTicket`、`EditorAssetDeletionTicket` 已经能做 Runtime-owned source mutation；下一步应统一为通用 mutation coordinator。
5. `PreviewScheduler` 的 in-flight 上限、token 和 stale completion 拒绝是正确内核，但 cache 本身没有预算/eviction/owner generation。
6. `AssetTypeRegistry` 的 exact type、toolkit、creation/context owner 结构是可用底座；问题是 catalog/detail/workspace projection 没有一路保留它。

## 4. 既有 P0 归属，不在本轮重复计数

| 父 owner | 未关闭边界 | Editor248 处理 |
|---|---|---|
| Editor226 | Locate request 无目标；Browser item 没有 Open/Activate 入口；catalog 丢 exact AssetTypeId | 继续作为 P0，下面只登记其下游 P1/资格门 |
| Editor60/61/247 | project/document/play/world generation、dirty conflict、scene reload、retired callbacks | asset mutation/preview/catalog 必须消费同一 generation fence |
| Runtime85/87/88/188 | source snapshot、identity/remap、watch gap、load/residency/build graph | Editor 不能在 catalog 或 UI 层自行修补这些 authority |
| tooling | tooling 将迁移 Rust | 按用户请求排除，不把 tooling 质量混入 Editor 计数 |

## 5. P1 工程化差距（32 项）

### 5.1 Catalog、identity 与 source/provider

| ID | 当前证据与差距 | 必须重构为 |
|---|---|---|
| ED6-P1-001 | `EditorAssetCatalogRecord` 只有 UUID/id/locator/`ResourceKind`，exact plugin type 在投影时丢失。 | Runtime 发布 `AssetTypeId + schema/version + compatibility`，Editor 全链保留，未知类型显示 opaque/unavailable。 |
| ED6-P1-002 | `EditorAssetReferenceRecord`/subasset 仍以 string locator/UUID + optional kind 表示，不能带 generation/remap lineage。 | `QualifiedAssetRef { project, source, asset, subasset, revision, expected_type }`。 |
| ED6-P1-003 | catalog rows 与 `EditorAssetIndex` 各保存 metadata/reference/transient 事实，更新靠 replace/retain 规则。 | 一个 immutable catalog generation + explicit editor-local overlay（dirty/importing/selection）。 |
| ED6-P1-004 | `EditorAssetIndex::rows()` 全量复制 runtime entries，缺 page/query cursor。 | `CatalogQuerySession` 提供 paged sort/filter/continuation token，UI 只物化可见窗口。 |
| ED6-P1-005 | folder record 只有 id/prefix/name/children/count，没有 source/provider id、mount health、trust、read/write/watch/cook capability。 | `ContentSourceProvider` 自带 identity/capability/health/generation，folder/item payload 不靠 scheme 猜测。 |
| ED6-P1-006 | `AssetSourceAuthority::from_locator` 只按 scheme 分类，package 不能表达 mount revision、remote/offline 或只读原因。 | provider authority + write policy + mount generation + actionable denial reason。 |
| ED6-P1-007 | catalog revision/publish epoch 与 Runtime `ProjectCatalogInputGeneration` 关联靠 pointer/sequence 检查，缺统一 receipt。 | `AssetCatalogPublicationReceipt` 同时携带 source/runtime/editor generation、changed pages 和 currentness。 |
| ED6-P1-008 | catalog details 中 `included_files`、references、subassets 可为空/Arc slice，但没有 per-field provenance 和 stale reason。 | details 记录 producer, input revision, completeness, unavailable diagnostic；局部缺失不可伪装成空集合。 |

### 5.2 Workspace、navigation 与 activation

| ID | 当前证据与差距 | 必须重构为 |
|---|---|---|
| ED6-P1-009 | `AssetWorkspaceState` 只有一份 selection/query/filter/view，Activity 与 Browser 共享/clone snapshot。 | `AssetBrowserInstanceId` + 独立 navigation/history/query/sort/column/selection/focus/expansion。 |
| ED6-P1-010 | folder tree 每次从全部 folders 递归构建；没有 lazy expansion 或 virtualized branch。 | provider page + expanded branch cache；100k folder 不做全树 clone。 |
| ED6-P1-011 | asset filter 对 display/file/locator 做 lowercase String allocation 和线性过滤。 | indexed token/prefix/kind/tag query；查询 generation 和 cancellation 可观测。 |
| ED6-P1-012 | 选择状态只有一个 UUID 字符串；没有 multi-select、anchor、keyboard focus、stale selection receipt。 | selection model 带 set/anchor/focus/current catalog generation，并在 rename/remove 时原子 reconcile。 |
| ED6-P1-013 | 只有 string `selected_folder_id`；不能表达 virtual source、favorites、collections、search result 或 saved query。 | `NavigationTarget` union + persistable history/favorites/collections。 |
| ED6-P1-014 | `AssetToolkitOpenRoute` 只有 locator + operation path，没有 type/toolkit generation/activation intent。 | route 携带 qualified item、toolkit id/generation、open mode、expected catalog revision。 |
| ED6-P1-015 | pointer click 只选择 item，press 只产生 drag payload；OpenAsset handler 与主 Browser 入口断开。 | pointer/keyboard/menu/reference 共用 `AssetActivationIntent` 和 terminal `ActivationReceipt`。 |
| ED6-P1-016 | Locate/Reveal 的 target、scroll/focus/filter adjustment 没有统一结果，Editor226 P0 仍暴露。 | `RevealAssetRequest` 必须带 browser instance/qualified target/generation/reason，返回 opened/revealed/hidden/stale。 |

### 5.3 Preview、toolkit 与 editor operations

| ID | 当前证据与差距 | 必须重构为 |
|---|---|---|
| ED6-P1-017 | `PreviewCache` 只按 UUID+variant 写 PNG 路径，没有 byte/entry/LRU/project/mount eviction。 | content-addressed preview artifact cache，预算、lease、eviction、orphan sweep 可诊断。 |
| ED6-P1-018 | `PreviewArtifactKey::thumbnail` 以 source hash 字符串拼 variant，未包含 asset type/schema/toolkit/platform/scale/color space。 | typed preview variant key + generator version + target/profile + source revision。 |
| ED6-P1-019 | `PreviewScheduler` 只有 dirty/visible/in-flight=64；不可取消等待队列，缺 priority/deadline/reason。 | priority queue + cancel token + generation fence + visible distance/viewport budget。 |
| ED6-P1-020 | preview job currentness 检查 catalog revision/Arc row/source hash/meta path，但无 Runtime payload revision/lease。 | preview 读取带 resource snapshot/lease，completion 同时校验 project/catalog/resource generations。 |
| ED6-P1-021 | preview error 以 `JobError::failed` 和 record state 表示，缺 per-variant last-good/negative cache/retry policy。 | variant-level state machine、backoff、last-good pointer、user-actionable diagnostic。 |
| ED6-P1-022 | toolkit registry 有 definition/owner，但 catalog action availability 没有 provider capability、busy/permission、unload drain。 | `AssetActionProvider` 统一 availability/preflight/invoke/result，plugin unload 等待 action/preview drain。 |
| ED6-P1-023 | editor asset state 保存 `ProjectManager` clone、`Arc<Mutex<EditorAssetIndex>>`、preview cache/scheduler；跨 project 生命周期靠手工 deactivate。 | `ProjectAssetWorkspaceSession` 显式 owner token、mount lease、teardown cancellation、retired receipt。 |
| ED6-P1-024 | poisoned RwLock/Mutex 统一 `into_inner`，可能继续使用 panic 后半写入的 catalog/preview state。 | poison 标记为 degraded，读可显示旧 generation，写入和 activation 必须拒绝直到 recovery。 |

### 5.4 Import、reimport、mutation 与 external change

| ID | 当前证据与差距 | 必须重构为 |
|---|---|---|
| ED6-P1-025 | `EditorAssetManager` API 只有删除、重定位、model import、preview；没有通用 create/rename/duplicate/reimport/bulk/open/activate。 | `AssetOperationProviderRegistry` 覆盖 capability/preflight/execute/undo/receipt/per-item outcome。 |
| ED6-P1-026 | `submit_model_import` 把 source path、目标目录/default material、mesh import 与 scene placement 组合在一个 model-only flow。 | source snapshot + importer/recipe/output plan；Import Asset 与 Place in Scene 是可独立撤销的 operation。 |
| ED6-P1-027 | import flow 有 admission/cancel，但没有 target folder provider、typed options schema、derived output graph、cross-platform target。 | operation request 固定 source/provider/type/recipe/target/profile，并返回 output/dependency receipts。 |
| ED6-P1-028 | delete/relocate 有 ticket，但 mutation 前后没有统一 expected generation、open-document/dirty/source-control/trash/undo policy。 | mutation coordinator 做 preflight -> expected generation -> durable commit -> compensation/undo -> receipt。 |
| ED6-P1-029 | watcher projection 只把 Runtime changes 压成 catalog change kind/UUID/locator；缺 producer/source/event sequence、digest、gap。 | qualified external change + decision/reconciliation plan；gap 触发 bounded resync。 |
| ED6-P1-030 | rename/remove 会局部 reconcile selection，未原子更新 folder/history/focus/open toolkit/document/reference repair。 | `AssetIdentityReconciler` 统一更新所有 browser instances、documents、toolkits、references 和 previews。 |
| ED6-P1-031 | `sync_from_project_with_runtime_generation` 每次改变都 full-build catalog/folders/details，shader IDE refresh 作为附带副作用。 | runtime delta -> changed pages/records，按 provider/page 增量发布；shader/preview 作为独立 subscribers。 |
| ED6-P1-032 | UI tests 多为 pointer/layout/replay 单元，不能证明真实 double-click/Enter/OpenWith/create/rename/bulk/cancel/fault/recovery。 | host integration/E2E 覆盖 input -> provider -> operation -> Runtime commit -> catalog/receipt -> UI reconciliation。 |

## 6. P2 性能、质量与维护

1. **ED6-P2-001**：catalog/folder/selection 使用大量 String clone；建立 100k/1M assets、深目录、多 source memory/p95/p99 基线。
2. **ED6-P2-002**：preview cache 没有 byte accounting、orphan cleanup 和 disk quota；补 cache hit/miss/eviction/stale/negative metrics。
3. **ED6-P2-003**：preview PNG 写入未声明 atomic temp/rename、fsync 或 crash recovery；与 Runtime artifact transaction 对齐。
4. **ED6-P2-004**：scheduler 常量 `MAX_PREVIEW_IN_FLIGHT=64` 不按 GPU/CPU/viewport profile 调整；变成 project/platform policy。
5. **ED6-P2-005**：asset matching 每次 `to_ascii_lowercase`；改为 normalized query/token cache，保留 locale-aware display search 边界。
6. **ED6-P2-006**：catalog details 的 `Option<Arc<...>>` 缺 reason/partial state，UI 只能将未加载与真正 empty 混同。
7. **ED6-P2-007**：change stream 事件只有 kind/revision/UUID/locator；补 event sequence、producer、coalesced count、gap/resync telemetry。
8. **ED6-P2-008**：type registry generation 有 creation menu action index，但 action availability 与 command registry/current toolkit generation 不共享 cache。
9. **ED6-P2-009**：source authority 错误和 importer/preview job 文本缺稳定 code/parameters，难以本地化和聚合。
10. **ED6-P2-010**：UI 资产表/树的 dynamic template、accessibility、keyboard navigation、localization 仍与 data provider 分离。
11. **ED6-P2-011**：现有 ignored/静态测试没有 multi-instance, project switch, plugin unload, stale completion, crash/poison recovery soak。
12. **ED6-P2-012**：Editor/Runtme preview/import/catalog 指标没有统一 correlation id，无法追踪一次 source change 的端到端 latency。

## 7. 参考引擎对照

| 参考 | 可吸收合同 | 当前差异 |
|---|---|---|
| Unreal AssetRegistry/SoftObjectPath/PrimaryAssetId | Registry 可按 tag/dependency/package/mount/filter 序列化；SoftObjectPath 区分 package/top-level/subobject；PrimaryAssetId 保留 type/name。 | Editor catalog 仅 `ResourceKind` + string locator，缺 mount/provider、exact type、subobject lineage 和 target-specific query。 |
| Bevy AssetServer/Handle | `AssetServer` 统一 source、loader、mode、meta check、event；Handle 可区分 strong/index/UUID；source reader/writer/watcher 是可替换能力。 | Editor 没有 ContentSourceProvider registry；Browser 由 scheme/prefix 推断 source，handle/activation 没有 strong/weak/currentness 层级。 |
| Fyrox editor/resource | ResourceManager 明确 loader/registry/task pool/watcher owner；loader 有 async load/convert/import options；editor item 双击打开、move 前 preflight。 | Zircon 有 tickets 和 preflight 局部实现，但 Browser item activation 入口仍断裂，generic operation/undo/async source provider 未闭合。 |
| Godot EditorFileSystem/ResourceLoader/UID | FileSystem 维护 UID、type、dependencies、import validity、modified/import times；ResourceLoader 支持 type/UID/dependency rename/cache mode。 | Editor catalog 没有 import validity/source digest confidence/UID reverse cache 的统一 projection，外部 rename/reimport 不能原子 reconcile 所有 consumers。 |
| Unity Graphics reimport/importer | `AssetReimportUtils.cs:17-49` 有 batch/progress/finally；`ShaderGraphImporter.cs:108-142,230-260` 声明 dependency 并生成 primary/subassets。 | Editor model import 和 preview 没有通用 batch/provider/typed recipe/output receipt；UI 可见进度与 cancellation 不能覆盖所有 asset kinds。 |

## 8. 目标架构与重构顺序

```text
EditorAssetWorkspaceService
  -> BrowserInstanceRegistry (history/query/selection/expansion/focus)
  -> ContentSourceProviderRegistry (mount/folder/page/capability)
  -> CatalogQueryService (paged immutable generation)
  -> AssetActivationRouter (exact type/toolkit -> receipt)
  -> AssetActionProviderRegistry (preflight/invoke/undo)
  -> AssetMutationCoordinator (Runtime expected-generation commit)
  -> PreviewService (variant cache/lease/budget/cancel)
  -> AssetChangeReconciler (watch gap/document/toolkit/reference)
```

1. **M248.0 关闭父 P0**：为 Locate/Activation/ExactType 接入 qualified target、dynamic item event、toolkit generation 和 unavailable receipt；没有这三项之前不增加更多 UI 按钮。
2. **M248.1 exact catalog/source contract**：与 Runtime188 对齐 exact AssetTypeId、schema/version、source/provider/mount identity、capability、currentness 和 page cursor。
3. **M248.2 Browser instances**：拆分 Activity/Browser instance state，加入 lazy tree、history/favorites/collections、multi-select、keyboard focus、persist/restore。
4. **M248.3 activation/action**：pointer/keyboard/context/reference 统一 activation；Open/Open With/Preview/Create/Reveal 都走 provider availability/preflight/invoke/result。
5. **M248.4 mutation/import**：create/rename/move/delete/duplicate/reimport/bulk 统一 expected generation、dirty/reference/document/source-control/trash/undo；把 model import 与 scene placement 分离。
6. **M248.5 preview/runtime**：preview 使用 Runtime resource snapshot/lease，按 typed variant、source revision、project generation 做 cache/eviction/cancel；completion 只接受当前 receipt。
7. **M248.6 external/scale**：qualified watch delta、gap resync、identity reconciler、plugin/project teardown；paged catalog、100k/1M benchmark、E2E/a11y/localization/telemetry。

## 9. 资格门（25 个）

- **Catalog identity**：exact type/schema/subasset/provider/mount generation 全链保留；unknown plugin 不静默变 builtin；catalog publication 带 Runtime source generation。
- **Browser instances**：两个 browser 的 folder/query/history/selection/focus/expansion 相互独立；100k folders 不全树 materialize；external rename/remove 原子更新所有实例。
- **Activation**：single click 只 select；double-click/Enter/Open command 只触发一次 qualified activation；toolkit 缺失返回 unavailable receipt；Locate 能 reveal/scroll/focus。
- **Action/mutation**：所有 create/rename/move/delete/duplicate/reimport/bulk 有 capability、preflight、expected generation、per-item result、undo/compensation、dirty/reference/document policy。
- **Import**：source path 先变 immutable Runtime snapshot；recipe/options/target/provider/derived outputs/dependencies 有 receipt；Import Asset 与 Place in Scene 可独立取消/撤销。
- **Preview**：typed variant key、entry/byte/disk budget、LRU/eviction、cancel、negative cache、last-good；stale project/catalog/resource completion 不得发布。
- **External/lifecycle**：watch event 保留 producer/source/sequence/digest/reason；gap 触发 bounded resync；project switch/plugin unload 取消 jobs/leases/drag/toolkits，旧回调全部退休。
- **Performance/quality**：catalog query、folder expansion、preview churn、multi-instance memory、import/reimport throughput、project switch、fault/poison recovery、keyboard/a11y/localization/telemetry 有动态报告。

## 10. Review-only 交付规则

本报告仅新增审查文档，没有修改 Editor/Runtime/tooling 生产代码。实施每个里程碑前必须重新导出 focused manifest/fingerprint，并把资格门变成真实 host integration/E2E；不得用 catalog 行数变化、单元测试或 ticket 构造成功替代 Runtime durable receipt 和用户可达的完整工作流。
