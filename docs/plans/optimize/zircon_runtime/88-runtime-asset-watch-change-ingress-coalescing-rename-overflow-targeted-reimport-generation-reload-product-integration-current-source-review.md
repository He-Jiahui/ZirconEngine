---
title: Runtime Asset Watch、Change Ingress、Coalescing、Rename、Overflow、Targeted Reimport、Generation、Reload 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime88
review_date: 2026-08-21
baseline_head: be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1
baseline_epoch: 336
related_code:
  - zircon_runtime/src/asset/watch
  - zircon_runtime/src/asset/pipeline/manager
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/watch_dispatch.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/watch_diagnostics.rs
  - zircon_runtime/src/asset/project/manager
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/sources.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/targeted.rs
  - zircon_runtime/src/asset/project/catalog_input_generation.rs
  - zircon_editor/src/ui/host/editor_asset_manager
  - zircon_editor/src/ui/host/asset_editor_sessions/watcher
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh
  - zircon_editor/src/ui/retained_host/app/assets/refresh.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/resources/events.rs
tests:
  - zircon_runtime/src/asset/tests/watcher.rs
  - zircon_runtime/src/asset/tests/pipeline/manager/watcher.rs
  - zircon_runtime/src/asset/tests/project/manager/targeted_import.rs
  - zircon_runtime/src/asset/tests/project/manager/full_generation.rs
  - zircon_runtime/src/asset/tests/project/manager/catalog_input_generation.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime/tests.rs
  - zircon_editor/src/tests/host/asset_manager_boundary
  - zircon_editor/src/tests/host/retained_asset_refresh
  - zircon_editor/src/tests/host/retained_event_bridge/asset_refresh_effects.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md
  - docs/plans/optimize/zircon_runtime/51-runtime-asset-registry-index-persistence-rebuild-incremental-query-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/53-runtime-dynamic-scene-asset-reload-event-generation-reconciliation-stage-apply-instance-replacement-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/87-runtime-asset-reference-identity-locator-guid-subasset-redirector-rename-move-resolution-repair-migration-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/57-editor-asset-workspace-content-browser-folder-source-tree-selection-open-create-import-rename-move-delete-history-collection-product-integration-review.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Developer/DirectoryWatcher/Public/IDirectoryWatcher.h
  - dev/UnrealEngine/Engine/Source/Developer/DirectoryWatcher/Private/Windows/DirectoryWatchRequestWindows.cpp
  - dev/UnrealEngine/Engine/Source/Developer/DirectoryWatcher/Public/FileCache.h
  - dev/UnrealEngine/Engine/Source/Developer/DirectoryWatcher/Private/FileCache.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/AutoReimport/ContentDirectoryMonitor.cpp
  - dev/godot/editor/file_system/editor_file_system.h
  - dev/godot/editor/file_system/editor_file_system.cpp
  - dev/bevy/crates/bevy_asset/src/io/mod.rs
  - dev/bevy/crates/bevy_asset/src/io/file/file_watcher.rs
  - dev/bevy/crates/bevy_asset/src/processor/mod.rs
  - dev/bevy/crates/bevy_asset/src/processor/tests.rs
  - dev/Fyrox/fyrox-resource/src/manager.rs
  - dev/Fyrox/fyrox-resource/src/event.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/SampleDependencyImportSystem/SampleDependencyImporter.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Material/AssetReimportUtils.cs
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Runtime Asset Watch、Change Ingress、Coalescing、Rename、Overflow、Targeted Reimport、Generation、Reload 与 Product Integration 当前源码工程化差距

## 1. 结论

当前资产监听链不是临时空壳。`AssetWatcher`已经同时限制ingress与pending的entries/bytes，具备debounce和max latency；overflow会生成显式reconciliation token。`ProjectAssetManager`又用Pending/Draining/Active/Retired activation、single worker、project generation、preparation epoch和三次候选提交重试，避免项目切换期间的旧watcher覆盖新project。full/targeted import都先在candidate上prepare，再把文件、registry、resource与project generation纳入提交；Editor asset manager也能从当前ProjectManager重新计算added/modified/removed/renamed delta。这些底座应保留。

但从“文件发生变化”到“产品消费者观察到已提交资产事实”的控制面仍有三条确定性P0。第一，普通`notify::Error`只广播字符串错误，不把source标成dirty，也不触发reconciliation；只有错误队列本身溢出后才全扫。OS已经明确表示事件流不可靠时，Runtime仍可无限期保留旧catalog、artifact和resource。第二，reconciliation提交后重新发布的是输入端原始`changes`，不是candidate与前代之间的committed delta；overflow会主动清空输入，因此可能出现Runtime已提交新generation，而Editor只订阅的`AssetChange`流完全没有事件。第三，声明为Compound的目录成员发生一次Modify时，watcher产生成员URI，shape-based incremental分支把成员文件当成`AssetSourceUnit::Single`导入并新建成员sidecar，既不重导父Compound也不验证父owner，造成独立资产与父复合资产事实分叉。

`.zmeta`变化还被测试明确固化为“不得发资产事件、不得提升revision”。这避免了Runtime自身写sidecar形成反馈环，但同时把source-control pull、外部编辑、import settings或identity变化永久排除在监听控制面之外。正确解法不是恢复无条件监听循环，而是区分internal committed write token与external metadata mutation，并把后者映射到source owner、recipe、identity和依赖闭包。

本报告新增登记 **3项P0、48项P1、12项P2和48个资格门**。Runtime25继续唯一拥有底层FileSystem/Source/Watch provider、path/URI与rename mapping；Runtime51拥有registry generation与durability-before-publication；Runtime53拥有动态Scene实例替换；Runtime64拥有resource load/reload authority；Runtime85拥有import/build graph、辅助输入与artifact；Runtime87拥有显式rename/move/reference repair；Editor57拥有Asset Workspace交互。本篇只拥有source change不确定性进入资产一致性控制面、source owner映射、reconciliation计划、committed generation delta及Runtime/Editor交付，不重复累计父finding。

## 2. 审查边界、currentness与证据

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test markers | 证据等级 | fingerprint |
|---|---:|---|---|
| Runtime watch、pipeline manager与project generation production | **99 / 10,570 / 9,679 / 388,691 / 32** | E3读取watch、activation、targeted/full、resource publication与generation；相邻manager逐文件盘点 | `445994ccbc7fe1a3fe38c2e0664d13610bc41edb1db22b5154eadd9da5a4736e` |
| Runtime focused tests | **8 / 2,720 / 2,477 / 97,477 / 55** | E3核对error、overflow、sidecar、rename、compound、catalog generation与reload意图 | `297403893dc7a3935f99cd754fa1be13b1dcec2f4aab00f3dd3224461cad3f00` |
| Editor committed-change consumers与独立ZUI watcher | **84 / 10,043 / 9,192 / 358,671 / 102** | E3读取startup订阅、drain/accumulate/plan/apply、catalog sync及ZUI watcher/reconcile | `025eb008f1ac6d08ea62a93e5fed83830291ac29659874f7944e57504bcc4bfc` |
| 五引擎参考切片 | **18 / 16,251 / 14,113 / 596,488 / 44** | E2/E3读取watch uncertainty、snapshot diff、meta/source event、reverse dependency与产品consumer | `58bfc1355bd03ce303ada402b78ea78c4019c0c6c7ca191e51db68b96760eb13` |

fingerprint按normalized lowercase relative path排序，串联`path + NUL + lowercase per-file SHA-256 + LF`后再取SHA-256。它冻结的是2026-08-21共享working tree，不是只读HEAD，也不是ABI或验收receipt。

Git基线为`be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1`，coordinator baseline epoch为336。Godot、Fyrox、Bevy与Unity Graphics参考revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`与`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal镜像无独立`.git`，由参考aggregate fingerprint冻结。

冻结集合中有8份非本轮dirty文件：Runtime的`management.rs`、`project_asset_manager.rs`、`runtime.rs`、`runtime/tests.rs`、`watch_dispatch.rs`，以及Editor的asset manager boundary、project deactivation与project access。本报告审查这些当前working-tree内容，不覆盖其改动；实施前必须重新冻结并重验三条P0。

### 2.2 证据限制

- E3能证明当前静态调用链和测试意图；本轮没有执行Cargo、Editor或真实filesystem watcher。
- 没有执行Windows/Linux/macOS rename、buffer overflow、network share、source-control、项目切换、fault、soak或benchmark。
- Unity `dev/Graphics`不包含Unity AssetDatabase引擎核心，只能证明package consumer如何接收import/move回调和批量reimport，不能用来声称完整Unity parity。
- 用户要求暂不审查未来将迁移为Rust的tooling；本篇只规定Runtime/Product合同与Editor handoff。

### 2.3 当前产品链

```text
notify backend
  -> bounded WatchIngress
  -> path-to-AssetUri + event fold
  -> AssetWatchBatch { raw changes, reconciliation flag, diagnostics }
  -> ProjectWatcherActivation bounded merge / single worker
  -> shape-based targeted import OR full generation
  -> file + registry + resource + ProjectManager candidate commit
  -> publish raw input AssetChange list + capacity-one generation wake

Editor retained host
  -> subscribes raw AssetChange, EditorAssetChange, ResourceEvent
  -> raw AssetChange nonempty => recapture current Runtime project
  -> EditorAssetManager computes its own catalog delta
  -> visual invalidation / scene and workspace consumers

ZUI authoring sessions
  -> second independent notify watcher
  -> path-only bounded queue
  -> open-session/import-reference reconciliation
```

## 3. 唯一 owner 与继承边界

| 既有owner | 唯一拥有内容 | Runtime88只负责 |
|---|---|---|
| Runtime25 | FileSystem/Source/Mount provider、path/URI codec、case/Unicode、rename mode与mapping uncertainty | 任意provider uncertainty如何标记资产source dirty并进入reconciliation |
| Runtime51 | registry index、catalog generation、persistence、durability-before-publication | committed generation delta的change-stream合同，不重记durability P0 |
| Runtime53 | SceneAsset事件进入world/instance的stage-replace事务 | 发布qualified asset generation；不重记Scene实例错换 |
| Runtime64 | resource handle、load/reload、lease、cache与revision authority | 关联同代asset/resource delta和terminal receipt |
| Runtime85 | source discovery、import recipe、reverse dependency、build graph、DDC/artifact/cook/package | 消费source owner图来制定watch invalidation，不重记外部auxiliary input P0 |
| Runtime86 | exact asset type/schema、document codec、typed dependency | 让change plan带exact type/schema，不重记dependency extractor P0 |
| Runtime87 | GUID/locator/subasset、显式rename/move、redirect/reference repair | 观察外部move并请求其mutation/reconcile能力，不把watcher猜测当rename事务 |
| Editor57 | Asset Workspace、Content Browser、用户mutation与conflict UX | 提供同代committed delta、gap与snapshot；不拥有Editor操作本身 |

## 4. 当前实现中应保留的底座

### 4.1 双层有界入口与明确最大等待

ingress和folded pending都同时限制entry与approximate bytes；overflow使用单槽信号唤醒loop，并在batch上设置`requires_reconciliation`。debounce与max batch latency分离，连续事件风暴不会无限延迟首批处理。现有测试覆盖pending overflow、ingress overflow和max latency。

### 4.2 项目切换期间的watcher activation协议

新watcher先Pending收集，project candidate安装后进入Draining，再变Active；旧activation先Retired，迟到callback会被拒绝。generation与preparation epoch在prepare/commit之间复核，旧项目候选不能直接覆盖新项目。这比“callback里直接reimport”可靠得多。

### 4.3 Candidate generation与资源提交已有事务轮廓

targeted/full都在ProjectManager clone上prepare，resource publication有preflight/cancel，文件generation提交后才替换active project。catalog input generation能计算added/modified/removed/renamed，并能跨跳过generation重建delta；Editor asset manager也有bounded subscriber mailbox，overflow会折叠为`CatalogChanged`。

### 4.4 Editor刷新已有预算和局部失速恢复

retained host按stream限制单次drain count/time，使用quiet/max-deferral accumulator；ResourceEvent流有Lagged状态并能触发reconcile。ZUI refresh pipeline也有source hash、dirty conflict、missing/invalid/retry、dependency generation和bounded job。这些能力应并入统一committed generation消费，而不是删除。

## 5. 参考实现差异与适用边界

| 引擎 | 本地源码证据 | 对Zircon的最低要求 | 不应误读 |
|---|---|---|---|
| Unreal | `FFileChangeData`包含`FCA_RescanRequired`；Windows `ERROR_NOTIFY_ENUM_DIR`和buffer overflow会显式生成rescan事件。`FFileCache`维护可持久目录快照、hash/move detection、outstanding transaction和change threshold；AutoReimport按addition/modification/deletion分阶段处理 | watcher不确定性必须成为可执行reconcile disposition；以snapshot diff而非原始事件作为资产提交事实；变更需有可完成事务和节流 | Unreal的FileCache/AutoReimport仍有历史复杂度，不应原样移植其线程或UI结构 |
| Godot | `EditorFileSystem::scan_changes`对filesystem snapshot做差分；`_update_scan_actions`先更新UID/依赖、计算reimports与reloads，再发`filesystem_changed`、`sources_changed`、`resources_reload`；依赖source会进入重导闭包 | source snapshot、UID/owner、dependency closure、reimport与产品信号必须按提交顺序统一 | Godot定期scan不等于Zircon应放弃watcher，只说明watcher必须能回到truth scan |
| Bevy | `AssetSourceEvent`区分Asset/Meta/Folder的add/modify/remove/rename与RemovedUnknown，并由`AssetSourceId`限定source；processor对meta rename检查old/new，reverse dependency测试证明单个leaf变化只重处理相关闭包 | source-qualified typed event、meta一等公民、unknown disposition、reverse owner graph和无变化短路 | Bevy file watcher错误目前主要日志化，不是Zircon错误收敛的完整上限 |
| Fyrox | `need_rescan()`直接重建registry并reload resources；From/To/Both各自处理；事件执行前重查当前filesystem truth；meta create/modify/remove都会更新或恢复registry | 错误/overflow必须有rescan fallback，处理迟到事件前验证当前事实，metadata变化不能永久静默 | Fyrox全量reload不满足Zircon最终的增量性能与generation资格 |
| Unity Graphics | `OnPostprocessAllAssets`接收import/delete/move/from路径；`AssetReimportUtils`用GUID选资产并在`StartAssetEditing/StopAssetEditing`批次中reimport | 产品consumer至少要接收committed import/move批次，并按稳定identity选择目标 | 本地corpus不是Unity AssetDatabase实现，不能作为watch/reconcile内部架构主证据 |

主导参考为Unreal的rescan + persistent file-state transaction；Godot稳定snapshot/reimport/signal顺序，Bevy稳定source/meta/依赖事件模型，Fyrox稳定rescan与metadata行为，Unity Graphics只作产品consumer旁证。

## 6. P0正确性阻断

### `WATCH88-P0-001`：普通watcher错误不会把source标为dirty或触发reconciliation

**确定性证据链：**

1. `watch_loop_inner`收到`Err(error)`时只调用`on_error(AssetWatchError::from_notify_error(...))`，不设置`requires_reconciliation`、started time或retry。
2. `ProjectWatcherActivation::enqueue_error`只把错误放入容量64的队列；仅当队列驱逐旧错误时才调用`mark_reconciliation_required`。
3. worker先广播error；普通单个错误对应的batch为空，`process_watch_batch_in_generation`不会执行。
4. 测试`watcher_failure_on_removed_directory_surfaces_observable_error`明确断言能收到错误且收不到change；没有后续reconcile断言。
5. Unreal把`ERROR_NOTIFY_ENUM_DIR`/buffer overflow提升为`FCA_RescanRequired`，Fyrox的`need_rescan()`直接重建registry，说明成熟实现不会把“观察流不可信”只当日志。

**产品后果：** watcher backend重建、目录临时不可达、OS buffer损失或网络文件系统错误后，某些源变化可永远不再被发现。catalog、artifact、resource和Editor都保留旧事实，但系统只显示一条无generation、无恢复状态的字符串错误。

**必须修复：** provider错误必须分类为Transient/SourceUnavailable/EventsLost/Terminal。任何可能丢事件的状态原子设置`DirtyScope`，停止targeted admission，调度truth snapshot reconcile；成功后发布新generation与terminal recovery receipt，失败保留last-known-good并继续有界退避，不能依赖“错误队列也溢出”才恢复。

### `WATCH88-P0-002`：reconciliation提交的是新generation，发布的却仍是原始输入事件

**确定性证据链：**

1. 任一ingress/pending/activation overflow都会清除partial changes并只保留`requires_reconciliation=true`。
2. `process_watch_batch_in_generation`在该状态执行full generation和完整resource reconciliation。
3. commit成功后`published_changes`直接由进入函数的`changes`克隆，不读取candidate的`ProjectCatalogInputGeneration` delta。
4. `publish_project_generation`即使changes为空也发capacity-one generation wake；内部测试专门验证这一点。
5. Editor startup只订阅`subscribe_asset_changes_with_wake`、Editor asset changes和ResourceEvent，不订阅project generation wake。
6. Editor仅在`events.asset_changes`非空时调用`refresh_from_runtime_project()`，因此空输入reconcile无法触发Editor catalog recapture。

**产品后果：** overflow或纯reconcile可能正确更新Runtime registry/resource/project，却让Asset Workspace、preview、visual cache及其他只消费AssetChange的产品保留旧generation。Runtime与Editor从同一进程内形成两个合法但不同的资产世界。

**必须修复：** publication只能发布`CommittedAssetGeneration { project_session, previous_generation, generation, delta, disposition, receipt }`。delta由提交前后catalog/source snapshot计算；若消费者gap或delta不可用，必须收到`SnapshotRequired`而不是空列表。generation wake、delta、resource correlation与durability状态使用同一序列和cursor。

### `WATCH88-P0-003`：Compound成员单事件被误导入为独立Single source

**确定性证据链：**

1. full scan通过目录旁的`.zmeta`识别Compound，并把成员列入`included_files`，同时跳过compound root下的普通单文件source。
2. watcher对成员文件产生成员自身URI，例如`res://bundles/source_bundle/first.json`。
3. `watch_changes_use_incremental_path`只看“恰好一个Added/Modified/Removed且无previous_uri”，不查询source owner。
4. Added/Modified通过`existing_or_primary_project_source_path_for_uri`解析到成员文件。
5. `prepare_targeted_import_source`对任何非目录路径固定构造`AssetSourceUnit::Single`、空included files和成员sidecar。
6. 现有Compound测试只直接调用父目录的topology validator，没有覆盖“成员watch event -> parent owner”。

**产品后果：** 修改复合模型/包/目录中的一个成员可创建新的成员资产和`.zmeta`，父Compound的digest、artifact、subasset与dependency保持陈旧。下一次full scan又会跳过该成员，形成targeted与clean scan不等价、身份冲突和引用漂移。

**必须修复：** generation必须维护`SourceOwnerIndex`，把每个declared member、metadata、recipe、auxiliary input映射到qualified source owner。watch event先解析owner，再按owner dependency closure生成invalidation plan；成员集合变化提升为owner topology reconcile。禁止由event数量/形状直接决定targeted correctness。

## 7. P1工程化差距（48项）

### 7.1 Authority、identity与事件合同（P1-001至P1-006）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| WATCH88-P1-001 | `AssetChange`只有kind、current URI、previous URI，没有project/session身份 | `ProjectSessionId + AssetSourceId + ProjectGenerationId`限定事件归属 |
| WATCH88-P1-002 | 多project root都映射成`res://`，raw event丢失物理root/source instance | 保留provider/source/root identity直到owner resolution完成 |
| WATCH88-P1-003 | 没有provider generation、monotonic sequence、received-at与observation id | versioned `SourceChangeEnvelope`和可追踪sequence |
| WATCH88-P1-004 | observed filesystem intent与committed asset delta复用同一DTO | `ObservedSourceChange`、`AssetInvalidationPlan`、`CommittedAssetDelta`分层 |
| WATCH88-P1-005 | error只有assets root与字符串，无法区分events-lost、unavailable、permission或terminal | typed `SourceWatchFailure`、recoverability与dirty scope |
| WATCH88-P1-006 | change没有terminal disposition，调用方无法知道committed/rejected/superseded/reconciled | generation-bound `AssetChangeReceipt` |

### 7.2 Normalization、coalescing与metadata政策（P1-007至P1-014）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| WATCH88-P1-007 | Runtime25登记的rename mode/path mapping uncertainty没有进入资产admission outcome | `Mapped/Ignored/ReconcileRequired/TerminalError`必须到达Runtime88 |
| WATCH88-P1-008 | chained rename fold会用后一次from覆盖原始source lineage | 保留rename chain、provider file identity与最终truth disposition |
| WATCH88-P1-009 | add/modify/remove/rename跨batch组合没有完整状态转换表 | property-based fold algebra与filesystem truth oracle |
| WATCH88-P1-010 | activation只合并相邻Added/Modified，同URI跨remove edge的语义靠vector位置隐含 | 显式per-source change state machine与barrier |
| WATCH88-P1-011 | directory create/remove与普通asset source没有typed区分 | Folder/Asset/Meta/Recipe/Unknown分类和owner-aware policy |
| WATCH88-P1-012 | `.zmeta`无条件忽略，外部settings/GUID/unit/included-files变化永久不可见 | internal write token suppression + external metadata mutation |
| WATCH88-P1-013 | 原子事务兄弟文件按名称模式忽略，没有transaction owner/generation证明 | writer-issued suppression token与有限TTL/sequence |
| WATCH88-P1-014 | 映射后不再次验证current truth，迟到Added/Modified可对应已删除/重命名文件 | planner snapshot阶段stat/read identity复核 |

### 7.3 Backpressure、调度与失败恢复（P1-015至P1-020）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| WATCH88-P1-015 | debounce、latency和两层容量使用硬编码默认，未绑定project/source capability | per-source policy profile与validated bounds |
| WATCH88-P1-016 | watcher与activation各有独立固定预算，无法证明端到端memory upper bound | 统一change-ingress budget与pressure receipt |
| WATCH88-P1-017 | public asset change subscriber使用unbounded channel | bounded cursor stream、Lagged/Gap与snapshot recovery |
| WATCH88-P1-018 | subscriber send失败只删除订阅，没有close reason、last cursor或consumer health | subscription lifecycle receipt和诊断快照 |
| WATCH88-P1-019 | full reconcile占用单一`watch_refresh_gate`，没有优先级、deadline、取消或foreground load公平性 | execution runtime中的budgeted reconcile operation |
| WATCH88-P1-020 | scan/import/commit/supersession失败后只广播错误，没有保留dirty retry latch | last-known-good + dirty-until-success + bounded backoff |

### 7.4 Source owner、依赖与invalidation计划（P1-021至P1-028）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| WATCH88-P1-021 | event URI直接当source owner，未查询Runtime85反向source dependency图 | immutable `SourceOwnerIndex` |
| WATCH88-P1-022 | package roots参与full scan却不创建watcher，也没有immutable/package mutation政策 | package source capability与显式watch/rescan/immutable disposition |
| WATCH88-P1-023 | manifest、importer registry、recipe profile和project settings不在同一watch invalidation domain | typed configuration inputs进入source snapshot |
| WATCH88-P1-024 | multiple changes固定退化full scan，即使owner互不相交 | proof-based affected owner set与bounded parallel actions |
| WATCH88-P1-025 | rename固定full scan，不消费Runtime87稳定identity/mutation evidence | identity-aware move observation或truth reconcile，禁止猜测repair |
| WATCH88-P1-026 | incremental选择只按event数量/shape，不按topology、dependency、importer capability | planner决定Targeted/Closure/Reconcile/Reject |
| WATCH88-P1-027 | external auxiliary input P0由Runtime85拥有，但Runtime88没有消费其未来reverse owner contract | source dependency edge变更必须进入watch owner resolution |
| WATCH88-P1-028 | full/targeted输出没有canonical invalidation/build plan与per-owner结果 | deterministic `AssetInvalidationPlan`和action receipts |

### 7.5 Generation、publication与consumer cursor（P1-029至P1-036）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| WATCH88-P1-029 | `AssetChange`没有catalog sequence、project epoch或source snapshot id | committed envelope携带全部generation pre/post condition |
| WATCH88-P1-030 | generation wake只是容量一空token，public consumer无法判断跳过几代 | sequence-bearing wake + cursor query |
| WATCH88-P1-031 | generation wake与change stream分离，没有原子关联或统一snapshot | 单一generation journal和subscription API |
| WATCH88-P1-032 | subscriber中途加入只能等未来raw event，没有current snapshot/cursor handshake | subscribe返回snapshot lease + next cursor |
| WATCH88-P1-033 | explicit `reimport_all`把所有imported条目都发布Modified，不是真实catalog delta | full/explicit/watch统一发布snapshot diff |
| WATCH88-P1-034 | targeted explicit reimport只按status存在与否决定Modified，不能表达unchanged/remove/failure | action outcome转换为exact committed delta |
| WATCH88-P1-035 | asset delta与ResourceEvent revision没有同代correlation id | generation transaction同时产出asset/resource projections |
| WATCH88-P1-036 | watch error不关联失败generation、affected owner或last-known-good generation | structured failure journal与health snapshot |

### 7.6 Editor与产品集成（P1-037至P1-042）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| WATCH88-P1-037 | Editor不订阅project generation wake，也没有asset stream lag状态 | committed generation subscription + Gap/SnapshotRequired |
| WATCH88-P1-038 | 每个raw AssetChange都触发Editor重新capture整个ProjectManager，再自行diff | Runtime提供immutable catalog delta/snapshot lease |
| WATCH88-P1-039 | queued AssetChange无project/session身份，项目切换后旧队列可被当前项目消费 | session-qualified cursor在deactivate时失效 |
| WATCH88-P1-040 | visual invalidation以observed URI而非committed locator delta为依据 | presentation只消费committed added/modified/removed/renamed |
| WATCH88-P1-041 | ResourceEvent能报告Lagged，AssetChange流没有等价恢复语义 | 所有产品流统一lag/gap/reconcile合同 |
| WATCH88-P1-042 | watch health/error没有进入Editor project health、notification history或可重试operation | typed health item、operator action与terminal receipt |

### 7.7 ZUI独立watch authority与生命周期（P1-043至P1-048）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| WATCH88-P1-043 | ZUI authoring sessions对相同project root启动第二套notify watcher | 消费Runtime committed source generation，不建立第二authority |
| WATCH88-P1-044 | ZUI watcher callback直接丢弃`notify::Error` | shared dirty/reconcile health contract |
| WATCH88-P1-045 | ZUI ingress只保存path，丢失event kind、source identity和provider uncertainty | qualified committed source/asset delta |
| WATCH88-P1-046 | ZUI overflow reconcile只扫描open sessions与import references，不是project filesystem truth | Runtime snapshot为truth，Editor只做open-document conflict reconcile |
| WATCH88-P1-047 | Runtime catalog generation与ZUI source hash/dependency generation没有共同precondition | document refresh plan绑定committed project generation |
| WATCH88-P1-048 | watcher drop无deadline地join；ZUI与Runtime双watcher都没有flush/retire terminal receipt | bounded quiescence、forced detach policy和shutdown evidence |

## 8. P2长期能力（12项）

| ID | 长期能力 | 目标 |
|---|---|---|
| WATCH88-P2-001 | provider file identity correlation | 在支持的平台用file id/journal提高rename chain精度，不能替代truth reconcile |
| WATCH88-P2-002 | adaptive debounce | 按source/import cost和burst动态调节，仍受max latency硬门约束 |
| WATCH88-P2-003 | persistent source-change journal | 崩溃/重启后从last committed cursor恢复或明确full reconcile |
| WATCH88-P2-004 | cross-process watcher leadership | Editor、cook、Hub并存时共享source generation而非重复扫描 |
| WATCH88-P2-005 | remote/VCS/content-addressed source provider | 同一change/reconcile合同支持非本地filesystem source |
| WATCH88-P2-006 | source-control batch hint | changelist/checkout/sync提供批次边界，但最终以snapshot truth为准 |
| WATCH88-P2-007 | change provenance timeline | 从OS event到owner/build/commit/consumer ack的可检索链路 |
| WATCH88-P2-008 | deterministic watcher simulator | 可脚本注入rename split、reorder、duplicate、loss、overflow和backend restart |
| WATCH88-P2-009 | large-project snapshot acceleration | directory shard、content summary与incremental stat cache，保持exactness |
| WATCH88-P2-010 | multi-machine reconciliation proof | clean import与incremental generation做可比较manifest/digest |
| WATCH88-P2-011 | workload-aware priority | 当前可见/打开资产优先，但不得饿死后台dirty owner |
| WATCH88-P2-012 | competitive benchmark suite | 同硬件同内容比较watch-to-visible latency、CPU、I/O、RSS和correctness gates |

## 9. 目标架构与hard cut

### 9.1 目标组件

```text
Runtime25 SourceWatchProvider / SourceSnapshotProvider
  -> SourceChangeAuthority
     -> qualified SourceChangeEnvelope + provider sequence
     -> SourceDirtyState / SourceWatchHealth
  -> SourceChangeNormalizer
     -> typed observed changes OR ReconcileRequired
  -> SourceOwnerIndex                     (Runtime85 graph input)
     -> member/meta/recipe/auxiliary -> qualified owner set
  -> AssetInvalidationPlanner
     -> Targeted / DependencyClosure / TopologyReconcile / FullReconcile
  -> AssetBuildCoordinator                (Runtime85)
  -> ProjectAssetGenerationTransaction    (Runtime51 + Runtime64)
     -> CommittedAssetGeneration
        -> exact catalog delta
        -> correlated resource delta
        -> terminal receipt / health transition
  -> AssetGenerationJournal
     -> snapshot + cursor + Gap/SnapshotRequired subscription

Editor
  -> one AssetGenerationConsumer
  -> EditorAssetCatalog projection
  -> open-document/ZUI conflict pipeline
  -> workspace/preview/visual/scene consumers
```

### 9.2 必须维持的不变量

1. watcher事件只是一条观察线索，不是资产事实；只有committed generation可进入产品流。
2. 任意可能丢事件的错误、overflow、mapping uncertainty或gap都保持dirty，直到truth reconcile成功。
3. incremental与clean scan对同一source snapshot必须产生相同owner、identity、dependency、artifact和catalog generation。
4. member/meta/recipe/auxiliary输入先解析到owner，再决定工作范围；事件shape不得承担correctness。
5. project/session/source/generation identity贯穿observe、plan、build、commit、publish与consumer ack。
6. 失败保留last-known-good；不得发布半代、空假delta或“错误但看起来已处理”的状态。
7. consumer backlog必须有上界；任何gap都以snapshot recovery闭合，不能用unbounded queue掩盖。
8. Runtime是project asset truth owner；Editor只维护authoring projection和dirty conflict，不再直接监听同一root。

### 9.3 Hard cut清单

- 用`CommittedAssetGeneration`替换公开raw `AssetChange`通知；不保留同名compat stream或双发。
- 删除shape-based `watch_changes_use_incremental_path`作为correctness决策入口，改由owner-aware planner选择范围。
- 删除`.zmeta`无条件ignore政策，改为internal write token suppression；外部metadata变化必须显式处理。
- 删除Editor ZUI的独立filesystem watcher；其refresh pipeline改消费Runtime generation和document conflict facts。
- 不为旧subscriber保留unbounded adapter；迁移所有consumer到snapshot+cursor后直接删除旧API。
- 不以“遇错全量扫描”永久替代owner graph；full reconcile是正确性fallback，不是唯一正常路径。
- 不在Runtime88重建Runtime25的VFS/provider，也不绕过Runtime85/51/64另建第二套import/registry/resource authority。

## 10. 依赖有序重构里程碑

| Milestone | 内容 | 退出条件 |
|---|---|---|
| M0 三项P0封口 | error-loss dirty latch、committed delta publication、Compound member owner修复 | 三条确定性序列有unit/integration/product test，incremental/clean等价 |
| M1 Change authority合同 | qualified envelope、typed failure、dirty state、health、provider sequence与internal write token | 每种provider outcome都有terminal disposition；`.zmeta`外部变更可达 |
| M2 Source snapshot与owner index | project/package/member/meta/recipe/auxiliary输入的immutable snapshot与reverse owner | 任一输入可确定owner/closure或显式要求reconcile |
| M3 Invalidation/build统一 | planner替换shape heuristic，full/targeted/watch/explicit reimport使用同一action模型 | 同snapshot产生canonical plan与相同generation |
| M4 Generation transaction与journal | exact catalog/resource delta、snapshot+cursor、gap recovery、last-known-good和failure receipt | 无空假delta、无unbounded asset channel、跳代可恢复 |
| M5 Editor hard cut | Editor catalog消费generation，删除ZUI watcher，open-document conflict绑定project generation | Runtime/Editor catalog同代，project switch无旧事件污染 |
| M6 Lifecycle、调度与观测 | bounded operation、priority/fairness/cancel/deadline、retry/backoff、quiescence与health UI | storm/failure/shutdown下预算与terminal receipt完整 |
| M7 平台与性能资格 | Windows/Linux/macOS、network/VCS、large project、fault/soak与同条件benchmark | 全部correctness gates先绿，再报告竞争性性能 |

M0依赖Runtime25提供mapping uncertainty入口并消费Runtime85的最小owner映射；M2/M3与Runtime85 build graph共同演进；M4依赖Runtime51/64 generation/resource transaction；M5依赖Editor57/Runtime53消费者迁移。不得用跨cratere-export、compat module或旁路facade延缓hard cut。

## 11. 资格门（48项）

### 11.1 P0与truth convergence（G01至G08）

| Gate | 必须证明 |
|---|---|
| G01 | 单个events-lost/notify error立即把source标dirty并调度reconcile |
| G02 | error后reconcile失败保留last-known-good并有界重试，不静默clean |
| G03 | ingress/pending/activation任一overflow都最终发布exact committed delta |
| G04 | changes为空但generation改变时，所有产品consumer仍能观察并收敛 |
| G05 | Compound成员内容变化只重导正确父owner/closure，不创建成员Single资产 |
| G06 | Compound成员增加、删除、rename触发topology reconcile并保持稳定identity |
| G07 | incremental generation与fresh clean scan的registry/artifact/catalog manifest一致 |
| G08 | 任何失败都不得发布半代或把dirty提前清除 |

### 11.2 Event、metadata与coalescing（G09至G16）

| Gate | 必须证明 |
|---|---|
| G09 | From/To/Both/Any/Unknown、mapping failure均有显式disposition |
| G10 | rename chain、rename-back、cross-root move、case-only rename可truth reconcile |
| G11 | add/modify/remove全部排列、重复、跨batch与迟到事件通过property tests |
| G12 | event处理前复核current existence/type/file identity，迟到事件不制造资产 |
| G13 | Runtime内部sidecar写不会反馈循环 |
| G14 | 外部sidecar settings/GUID/unit/included-files变化会进入正确owner plan |
| G15 | transaction sibling suppression只能由有效writer token触发，伪同名文件不被永久忽略 |
| G16 | project root、package root、meta、folder、asset与unknown event均保留source identity |

### 11.3 Owner graph与invalidation（G17至G24）

| Gate | 必须证明 |
|---|---|
| G17 | 每个declared member、meta、recipe与auxiliary input可查询qualified owner |
| G18 | owner ambiguity fail closed并提升reconcile，不任选第一个root |
| G19 | 独立多owner batch可并行且结果顺序确定 |
| G20 | dependency leaf变化只重建reverse closure，无关资产不重建 |
| G21 | dependency边删除/rename后旧closure被清理，新closure同代生效 |
| G22 | package root明确证明watched、snapshot-polled或immutable，不能隐含漏变更 |
| G23 | importer/profile/manifest变化进入同一source snapshot与plan |
| G24 | Runtime87显式move receipt与外部filesystem move observation都不产生双重repair |

### 11.4 Generation、stream与failure（G25至G32）

| Gate | 必须证明 |
|---|---|
| G25 | committed envelope带project session、source snapshot、previous/current generation和sequence |
| G26 | catalog delta精确表达added/modified/removed/renamed/unchanged与reconciled |
| G27 | asset/resource delta共享correlation id且对同一generation可原子观察 |
| G28 | subscriber加入获得snapshot + next cursor，无初始化窗口 |
| G29 | bounded subscriber溢出返回Gap/SnapshotRequired并能恢复到current generation |
| G30 | explicit full/targeted/watch reimport发布同一种committed envelope |
| G31 | superseded、cancelled、failed、committed都有terminal receipt与affected owner |
| G32 | restart可从journal恢复cursor或显式要求full reconcile，不伪造连续性 |

### 11.5 Editor产品与生命周期（G33至G40）

| Gate | 必须证明 |
|---|---|
| G33 | Editor Asset Catalog与Runtime committed generation始终可证明同代 |
| G34 | project switch使旧session cursor失效，旧事件不得刷新新项目 |
| G35 | visual/preview/workspace消费committed locator delta，不消费raw filesystem path |
| G36 | ZUI独立notify watcher已删除，open-document refresh只消费Runtime generation |
| G37 | ZUI dirty local edit与external committed generation产生可决定conflict，不盲覆写 |
| G38 | asset stream lag在Editor触发snapshot reconcile，行为与ResourceEvent lag一致 |
| G39 | watcher health、retry和terminal failure进入project health/notification history |
| G40 | shutdown有quiescence deadline、flush/retire receipt且无callback-after-owner |

### 11.6 平台、压力与性能（G41至G48）

| Gate | 必须证明 |
|---|---|
| G41 | Windows/Linux/macOS watcher conformance覆盖rename、overflow、backend restart |
| G42 | network share、temporary unavailable和permission change都收敛或显式terminal |
| G43 | 100k+ source initial snapshot和single change满足声明的CPU/I/O/RSS预算 |
| G44 | burst/storm下ingress、activation、operation与subscriber总memory有硬上界 |
| G45 | foreground load、save与watch reconcile调度公平，无无限饥饿或全局冻结 |
| G46 | fault injection覆盖scan/read/import/file commit/registry/resource/publish/consumer gap |
| G47 | 长时soak无重复generation、丢owner、stuck dirty、thread leak或unbounded journal |
| G48 | 与Unreal/Godot/Fyrox/Bevy同条件比较watch-to-visible latency时，先满足全部correctness gates再宣称性能优势 |

## 12. 禁止的临时实现

- 禁止在`notify::Error`回调里只加日志或toast；可能丢事件的错误必须保持dirty并收敛。
- 禁止为P0-002简单发送一个伪`Modified(res://)`事件；必须发布真实committed delta或SnapshotRequired。
- 禁止用“成员路径向上找第一个`.zmeta`”替代generation-built owner index；多root、nested compound和package会再次歧义。
- 禁止继续把event count等于1当成targeted correctness证明。
- 禁止把AssetChange channel容量从unbounded改成更大bounded后直接drop；必须有cursor/gap/snapshot合同。
- 禁止保留Runtime watcher与ZUI watcher双authority并靠debounce调参缓和竞态。
- 禁止用永久full scan掩盖reverse owner graph缺失；fallback与正常增量路径必须分离。
- 禁止在Runtime88复制Runtime25 provider、Runtime85 build graph或Runtime51/64 publication owner。

## 13. 完成边界

本报告完成的是当前源码静态审查与重构需求登记，不是代码修复。只有M0-M7按48项资格门取得可复验回执，且Runtime25/51/64/85/87与Editor57的依赖合同完成hard cut后，Runtime88才能把`implementation_status`改为`complete`。

本轮只修改review文档与索引，未修改Rust、Cargo、资源或tooling实现；未运行Cargo、Editor、真实watcher、source-control、fault、soak或benchmark。因此不能据此宣称功能已修复、动态测试通过，或性能/表现已经优于Unreal。
