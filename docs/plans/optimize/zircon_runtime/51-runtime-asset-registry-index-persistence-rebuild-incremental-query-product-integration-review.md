---
title: Runtime Asset Registry、Index、Persistence、Rebuild、Incremental、Query 与 Product Integration 工程化差距
category: zircon_runtime
report_id: Runtime51
review_date: 2026-08-20
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/asset/registry
  - zircon_runtime/src/asset/project/manager
  - zircon_runtime/src/asset/project/catalog_input_generation.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/asset/pipeline/manager/records/project_info_from_project.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui.rs
  - zircon_runtime/src/scene/world/project_io/references.rs
  - zircon_editor/src/core/asset/index.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync
tests:
  - zircon_runtime/src/asset/tests/registry_index
  - zircon_runtime/src/asset/tests/project/manager/full_generation.rs
  - zircon_runtime/src/asset/tests/project/manager/targeted_import.rs
  - zircon_runtime/src/asset/tests/project/manager/catalog_input_generation.rs
  - zircon_runtime/src/asset/tests/pipeline/manager/watcher.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/AssetRegistryState.h
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/IAssetRegistry.h
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetDataGatherer.h
  - dev/bevy/crates/bevy_asset/src/server/info.rs
  - dev/bevy/crates/bevy_asset/src/server/mod.rs
  - dev/bevy/crates/bevy_asset/src/io/source.rs
  - dev/godot/editor/file_system/editor_file_system.h
  - dev/godot/editor/file_system/editor_file_system.cpp
  - dev/godot/core/io/resource_uid.h
  - dev/godot/core/io/resource_uid.cpp
  - dev/Fyrox/fyrox-resource/src/manager.rs
  - dev/Fyrox/fyrox-resource/src/registry.rs
  - dev/Fyrox/fyrox-resource/src/graph.rs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Importers/ShaderGraphImporter.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Importers/RenderPipelineChangedCallback.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Material/AssetReimportUtils.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: false
---

# 51 · Runtime Asset Registry、Index、Persistence、Rebuild、Incremental、Query 与 Product Integration 工程化差距

## 1. 结论

Zircon的Asset Registry不是空壳。16个production registry文件已经提供UUID/path/source反查、referencer反向边、tag/type/path/package过滤、JSON持久化、corrupt后重建、重复GUID检测、watch增量入口、targeted replacement和project inspection；产品全量导入也不是“扫描后逐文件裸写”。`ProjectedMetaInventory`会先在内存中投影sidecar变化，最终把`.zmeta`、artifact和`asset-registry.json`放进同一个durable journal，`ProjectCatalogInputGeneration`又提供不可变`Arc` generation、sequence与added/removed/renamed delta。这些基础必须保留。

但当前registry仍是项目扫描缓存，而不是工程级内容数据库。持久row只有UUID、URI、宽泛`AssetKind`、tag、裸UUID依赖和source digest；没有exact type/schema、artifact/cook key、source/mount/provider、package revision、dependency category、tombstone/redirect或registry generation。JSON v1不绑定project、BuildSet、root/config或source inventory，合法但陈旧的文件会直接被接受；所有查询都在HashMap上扫描或重新分配排序，多数public query只有测试caller。

本轮确认两项新的产品硬阻塞。第一，重复GUID会被自动remint并写回sidecar，但没有扫描和迁移所有serialized inbound UUID reference，也没有redirect、tombstone、migration receipt或operator decision；独立`rebuild_from_project`还会在registry持久化前逐个改写sidecar，失败可留下部分身份变更。第二，durable transaction可能返回`RecoveryDeferred`，而ProjectManager、ProjectAssetManager、resource registry、watcher和project generation event已经先安装/发布candidate；随后`ensure_durable()`才把该结果转换为错误，形成“调用者收到失败，但live state和事件代际已经推进”的unknown-outcome边界。

因此Runtime51登记 **2项P0、60项P1和16项P2**。目标不是复制Unreal UObject/package历史，而是建立一个有明确schema与generation的不可变Asset Registry State：扫描、import、依赖提取、artifact验证和identity migration都在candidate中完成；durability得到最终terminal disposition后再原子发布registry/resource/catalog/event同一代；query走预编译二级索引、cursor/visitor和有界结果；失败保留last-good generation与可恢复证据。

本轮只做静态review，没有修改production、tests、Cargo或reference source；没有运行Cargo、项目导入、Editor、watch、fault injection、soak或benchmark。

## 2. 审查边界、规模与 currentness

### 2.1 物理冻结

| 范围 | 文件 / 行 / bytes / tests | 结论 |
|---|---:|---|
| `asset/registry` production | 16 / 1,861 / 64,031 / 3 | 逐文件读取entry、index、query、persistence、rebuild、incremental、targeted、inspection和extractor |
| dedicated registry tests | 6 / 890 / 30,186 / 19 | 逐项读取query、persistence、incremental、extractor与scan safety用例 |
| ProjectManager与catalog generation | 11个focused文件 | 读取open、full/targeted generation、projected inventory、durable transaction、publication与registry access |
| AssetManager产品链 | 5个focused文件 | 读取open/reimport/watch、resource同步、generation event和diagnostic投影 |
| Runtime/Editor消费者 | 5个focused文件 | 读取Runtime UI、Scene reference、Editor index与project sync |
| 产品集成tests | 4个focused文件 | 读取full/targeted/catalog/watch generation回归 |
| focused fingerprint集合 | 47 / 11,574 / 419,973 / 74 | SHA-256 `d7a1f02ecd673972dabda38fa98f6d8935bcaf1d9e8c63342314e2d6074c6597` |

focused fingerprint按相对路径小写排序，将每项编码为 `path + NUL + per-file SHA-256`，以LF连接后再次计算SHA-256。它只标识本次读取集合，不是artifact、ABI、registry generation或release identity。基线HEAD为`bea1acf91b909525ab1759e2c800858b0eda6528`。

冻结时47个focused文件均无working-tree差异；相邻的`project_asset_manager/management.rs`已有其他会话/用户修改，但不在本轮证据集合，本报告没有覆盖、暂存或回退它。`source_recheck_required`因此在当前冻结点为false；任一focused文件或reference snapshot变化后必须重新置true。

### 2.2 当前产品链与独立helper必须分开判断

```text
产品full generation
  scan sources -> ProjectedMetaInventory
  -> 内存normalize duplicate GUID
  -> import + dependency projection + candidate registry/catalog
  -> journal(.zmeta + artifact + asset-registry.json)
  -> resource/project candidate install
  -> generation event publish
  -> ensure_durable()  // 当前过晚

独立registry rebuild
  scan *.zmeta
  -> normalize_duplicate_guids()逐文件save
  -> rebuild index
  -> persist registry
```

两条路径不能互相替代。产品full/targeted path已经有较强的prepare/journal基础；独立public rebuild与`AssetRegistryIndex::apply_watch_changes`仍会重扫、clone并直接改sidecar。重构应删除第二套mutation authority，让inspection保持纯读，所有写入统一进入project generation transaction。

### 2.3 当前可保留底座

- `AssetRegistryIndex`在插入/替换时维护UUID、path、asset id、source和referencer反向映射；
- persistence使用typed error、版本字段、duplicate校验和atomic registry write；
- product full generation用`ProjectedMetaInventory`在commit前投影sidecar，不会像独立rebuild那样立即逐文件写；
- product source集合最终按URI排序，full generation candidate的主要输入顺序是确定的；
- durable transaction已有owner lock、journal、recovery和`Durable/RecoveryDeferred`区分；
- `ProjectCatalogInputGeneration`已是不可变、共享、带sequence与delta的发布对象；
- targeted path能只重导入单个added/modified/removed source，并在generation/epoch变化时拒绝旧candidate；
- scanner拒绝symlink/reparse并使用canonical containment，避免明显越界。

这些能力说明正确方向是收敛authority与publication protocol，而不是再建一套“更高级registry”旁路。

## 3. 关键代码事实

### 3.1 Row与graph只够做局部查找

`AssetRegistryEntry`只有`AssetUuid`、`AssetUri`、`AssetKind`、`BTreeSet<String>` tags、`Vec<AssetUuid>` dependencies和`String` source digest。依赖去重使用临时`HashSet`并保留首次出现顺序，但edge没有required/optional、hard/soft、source/artifact/manage/searchable、expected type/schema、minimum revision或provenance。dependency extractor只对Scene、Material和Model返回非空结果，其余`ImportedAsset` variant静默得到空依赖。

`AssetRegistryIndex`维护多张HashMap和diagnostic Vec，但没有registry generation。`entries()`每次分配Vec并排序，`get_assets()`全表扫描后排序；type/tag/path/package均无二级索引。dependency/referencer查询对“asset不存在”和“存在但无边”都返回空结果，`resolve_asset_id_for_reference`在UUID错误或缺失时静默退回path。快照、cursor、错误和诊断无法说明自己属于哪个Project/BuildSet/catalog generation。

### 3.2 JSON存在不等于current

`asset-registry.json` v1只保存version和entries。load会整文件读取并一次性parse，未限制bytes、entries、tags、dependencies、字符串长度、嵌套深度、时间或取消。成功decode后不核对project identity、root inventory、sidecar/source digest、importer/config/target、artifact key或catalog generation，因此一个格式完全合法但内容陈旧的registry会被直接接受。

`load_or_rebuild`把I/O、权限、decode、unsupported version和duplicate等不同错误统一转成rebuild。原损坏文件没有quarantine/backup/evidence，unsupported version也没有migration。`CorruptPersistenceRebuilt` diagnostic在rebuild persist之后追加，下一次打开看不到这条恢复事实。

### 3.3 扫描与重复身份处理不是确定性迁移

独立scanner递归`read_dir`，没有排序目录项或最终meta path。遇到重复GUID时，第一个被枚举的sidecar保留身份，后续sidecar生成随机UUID；谁是owner取决于filesystem enumeration。多个asset root映射到同一logical URI、大小写折叠冲突、Unicode规范化冲突、orphan `.zmeta`、hidden/vendor/generated输入和root precedence没有统一policy。

更严重的是remint只改`.zmeta`。场景、材质、模型、插件数据、save/cook artifact或外部索引中已经序列化的旧UUID不会被reference closure scanner重写；也没有old→new redirect、tombstone、冲突选择、dry-run、备份、receipt或回滚。产品journal解决了“sidecar和registry是否同批落盘”，没有解决“稳定语义目标是否仍指向同一个对象”。

### 3.4 Incremental存在两套语义

public `AssetRegistryIndex::apply_watch_changes`只有registry tests caller：它clone整张index、扫描全部meta、调用非事务duplicate normalization、刷新边并重写完整JSON。产品ProjectManager没有使用它，而是单个Added/Modified/Removed走targeted generation，rename或多事件回退full generation。继续维护两套API会让测试证明错误的产品路径。

产品targeted path也不是规模化增量数据库。单源更新仍clone整个`AssetRegistryIndex`；source entry枚举来自HashSet，affected owner和diagnostic顺序不稳定；dependency path retarget只删除第一个匹配并直接append新path；dependency去重使用`Vec::contains`。当前affected set从直接referencer开始，刷新只处理该集合，没有可证明的transitive readiness/diagnostic closure协议。

### 3.5 Durability terminal disposition晚于发布

`ProjectFileCommitOutcome::RecoveryDeferred`表示commit marker durability未决，要求重启或reopen恢复。可是direct `ProjectManager::scan_and_import*`先执行`*self = candidate`，再`ensure_durable()`；ProjectAssetManager的open/import/reimport先安装project/resource candidate并调用`publish_project_generation`，再检查durability；watch也先记录commit、发布generation与changes，再广播durability error。

因此同一操作可同时产生：磁盘事务状态未决、live registry/resource/catalog已切换、watcher已激活、generation event已发送、API返回Err。调用方无法安全重试，因为重试可能重复import或覆盖已发布代；订阅者也无法根据成功event推断durable terminal state。这不是底层journal缺失，而是publication protocol使用顺序错误。

### 3.6 Product消费面很窄且分裂

production中，Runtime UI通过`project.asset_registry().entries()`同步构建UI asset store并逐项加载artifact；Scene reference使用UUID/path fallback；project info只投影diagnostic count。`get_assets`、dependency/referencer query和registry `apply_watch_changes`没有真实production caller。Editor另有`EditorAssetIndex`，从`Arc<AssetRegistryIndex>`重新构建rows并维护import state；Editor project catalog又从`ProjectCatalogInputGeneration`建立自己的source/reference graph。

这证明当前不是“一张权威registry服务所有产品”，而是runtime registry、catalog generation、resource registry和editor index各自拥有部分事实。三套Editor truth的最终owner仍是Editor04；Runtime51只要求runtime发布一个可租用、带generation与schema的权威内容图，供Editor做非权威投影。

## 4. 与参考引擎的可迁移差异

| 参考 | 已核对能力 | Zircon应吸收的合同 | 不照搬项 |
|---|---|---|---|
| Unreal AssetRegistry | State按package/path/class/tag建立索引；依赖查询有category/query；serialization options控制registry/dependency/searchable/manage/package/tag数据；gatherer有async context、progress、cache、path wait和prioritized scan | immutable indexed state、category edge、visitor/query、可配置持久schema、async gather/progress/cache与规模内存核算 | 不复制UObject/package兼容债务、全局singleton或同步load旁路 |
| Bevy AssetServer | `AssetInfo`跟踪typed path、load/dependency/recursive state、failed/loading set、dependents、loader dependencies/hash和pending tasks；AssetSource区分named source及processed/unprocessed reader/writer/watcher | registry row与load/cook state分层、source capability、reverse edge成本策略、typed failure state | 不把进程内TypeId当长期schema，也不把Bevy内存server冒充持久数据库 |
| Godot EditorFileSystem/ResourceUID | FileInfo记录UID/type/mtime/import mtime/md5/dest/import validity/deps；扫描/导入有progress/thread/change action；ResourceUID维护ID↔path与持久cache | currentness输入、import state/progress、明确change action、稳定UID反查与cache update | 不复制EditorFileSystem单体、裸路径全局表或其主线程耦合 |
| Fyrox ResourceManager/Registry/Graph | manager暴露loaded status、异步request/wait/event与excluded folders；dependency graph可通过reflection收集；registry维护UUID→path | async状态/事件、excluded root policy、schema驱动dependency discovery | Fyrox registry本身较轻量，只作二级证据，不把简单map当目标上限 |
| Unity Graphics消费者 | ShaderGraph importer区分source/artifact/custom dependency，生成main/subasset/metadata；render pipeline hash触发custom dependency invalidation；批量reimport有editing scope/progress | source/artifact/custom edge分类、subasset identity、custom environment hash和批量reimport scope | 本地`dev/Graphics`不含Unity核心AssetDatabase实现，不能用consumer API推断其内部架构 |

共同底线不是“API数量多”，而是registry state必须可证明current、依赖可分类、scan/import可进度与取消、query可索引、publication可关联同一generation。Zircon可通过紧凑Rust布局、immutable generation与批量visitor争取比Unreal更低的查询和发布成本，但必须用同corpus统计基准证明。

## 5. 与既有 canonical owner 的边界

| 事实 | Canonical owner | Runtime51只拥有 |
|---|---|---|
| exact asset type/schema、typed dependency graph、artifact/cook identity、last-good、pack | Runtime04 | registry row/index/persistence对这些字段的承载与查询 |
| stable UUID算法、owner/domain、generation/exhaustion、跨schema迁移 | Runtime24 | duplicate asset GUID remint的reference closure、redirect与transaction |
| filesystem/source/mount、root collision、watch映射、durable writer primitive | Runtime25 | registry scan plan和project publication消费这些合同 |
| Editor三套asset truth、browser/import/thumbnail/reference UX | Editor04 | runtime immutable generation作为唯一上游authority |
| prepare/commit/publish全局operation与unknown outcome | Tooling37 Transaction报告 | AssetManager中`RecoveryDeferred`晚于live publication的具体P0 |
| hot path全表重建与EditorAssetIndex性能 | PERF-MVP-556 | registry index/query/cursor与产品caller资格 |

Runtime51的2项P0是当前asset registry纵向实现中的具体数据完整性和publication缺陷；修复时必须同时更新父owner的shared contract，不得复制另一套UUID migration、filesystem transaction或Editor catalog。

## 6. P0差距清单（2项）

| ID | 当前证据 | 风险 | 必须达到的修复合同 |
|---|---|---|---|
| ASSETREG-P0-001 | duplicate GUID normalization会为后续sidecar随机remint；产品path只把sidecar与registry同批提交，独立rebuild还会逐文件提前保存；两者均不扫描/改写serialized inbound UUID references | 已存在的Scene/Material/Model/插件/save/cook引用可能继续指向保留owner或错误对象；失败可留下部分身份改写，且没有operator可审计证据 | duplicate必须先进入只读collision report；由确定性policy或显式operator选择owner，构建全repository reference closure，原子提交sidecar+serialized docs+registry+redirect/tombstone+receipt；任何未知codec或未迁移引用使publish失败并保留last-good |
| ASSETREG-P0-002 | direct ProjectManager和ProjectAssetManager open/import/reimport/watch均在candidate/resource/watcher/generation event发布后调用`ProjectFileCommitOutcome::ensure_durable()`；`RecoveryDeferred`此时变Err | API失败与live成功同时发生，consumer已看见新代但磁盘terminal state未决；自动retry不具备幂等性，restart recovery可能与已广播事实不一致 | commit protocol在持有publication fence时取得明确`Durable`或显式`AcceptedRecoveryPending(OperationId)`；普通成功event只能在durable terminal后发布。若允许pending，API/event/state必须一致标记同一OperationId并禁止盲重试；失败不得安装candidate或必须有可证明compensation |

## 7. P1差距清单（60项）

### 7.1 Identity、row与dependency graph（10项）

| ID | 差距 |
|---|---|
| ASSETREG-P1-001 | row只有宽泛`AssetKind`，没有稳定exact `AssetTypeId` |
| ASSETREG-P1-002 | row没有`SchemaId`、codec/importer schema version或migration chain |
| ASSETREG-P1-003 | row不绑定artifact manifest hash、cook key、target/profile或toolchain identity |
| ASSETREG-P1-004 | row不记录source instance、mount/provider、physical identity或trust provenance |
| ASSETREG-P1-005 | row没有package revision、registry generation、BuildSet或project identity |
| ASSETREG-P1-006 | dependency只是裸UUID，没有category、required、expected type/schema、minimum revision和provenance |
| ASSETREG-P1-007 | dependency Vec的顺序来自extractor首次出现，未冻结canonical排序和duplicate diagnostic |
| ASSETREG-P1-008 | rename/remint/delete没有redirect、tombstone、retirement epoch或expiry policy |
| ASSETREG-P1-009 | registry row不表达source discovered/importing/ready/stale/failed/missing-artifact等状态 |
| ASSETREG-P1-010 | diagnostic没有稳定ID、generation、source span、owner、operation或remediation结构 |

### 7.2 Persistence、currentness与recovery（10项）

| ID | 差距 |
|---|---|
| ASSETREG-P1-011 | 格式合法的陈旧registry不会核对sidecar/source digest或artifact currentness |
| ASSETREG-P1-012 | persistence header不绑定project/root/config/importer set/BuildSet/catalog generation |
| ASSETREG-P1-013 | load整文件读入并整体JSON decode，没有byte/entry/time/cancel admission |
| ASSETREG-P1-014 | tag、dependency、字符串、path和diagnostic没有per-row与total budget |
| ASSETREG-P1-015 | I/O、permission、decode、version和duplicate全部走同一自动rebuild disposition |
| ASSETREG-P1-016 | corrupt/unsupported原文件没有quarantine、backup、hash、raw evidence或operator action |
| ASSETREG-P1-017 | unsupported version直接rebuild，没有versioned migration和reader/writer window |
| ASSETREG-P1-018 | `CorruptPersistenceRebuilt`在persist后追加，恢复事实不进入持久receipt |
| ASSETREG-P1-019 | save clone并排序全部entry，再pretty JSON全量编码；没有峰值内存与写放大预算 |
| ASSETREG-P1-020 | load/rebuild没有read-only/degraded/last-good模式，权限错误也可能触发有副作用的扫描 |

### 7.3 Scan、rebuild与root policy（10项）

| ID | 差距 |
|---|---|
| ASSETREG-P1-021 | `read_dir`与meta path未排序，duplicate owner和diagnostic顺序依赖filesystem enumeration |
| ASSETREG-P1-022 | 多root可投影到同一AssetUri，registry scan没有显式root priority/collision report |
| ASSETREG-P1-023 | 缺少Windows/macOS case-fold、Unicode normalization和canonical URI collision policy |
| ASSETREG-P1-024 | 没有include/exclude、hidden/vendor/generated/source-control-ignore等scan policy schema |
| ASSETREG-P1-025 | 任一symlink/reparse会中止整个scan，没有per-entry typed disposition与policy-controlled skip |
| ASSETREG-P1-026 | orphan `.zmeta`在source缺失时静默跳过，没有tombstone、diagnostic或recovery action |
| ASSETREG-P1-027 | rebuild信任sidecar的source digest，不读取source或验证digest/artifact对应关系 |
| ASSETREG-P1-028 | scan没有entries/depth/bytes/time/deadline/cancel/progress budget |
| ASSETREG-P1-029 | metadata load和decode串行执行，没有有界I/O/CPU pipeline或deterministic merge |
| ASSETREG-P1-030 | public `rebuild_from_project`在registry persist前直接改多个sidecar，没有统一generation transaction |

### 7.4 Incremental、watch与closure（10项）

| ID | 差距 |
|---|---|
| ASSETREG-P1-031 | public registry `apply_watch_changes`没有production caller，却形成第二套mutation authority |
| ASSETREG-P1-032 | 该helper为小变更clone整张index，未达到增量数据结构语义 |
| ASSETREG-P1-033 | 该helper仍扫描全部meta并做duplicate normalization，不是watch delta apply |
| ASSETREG-P1-034 | empty change直接返回，不验证watch overflow、root generation或source drift |
| ASSETREG-P1-035 | 产品targeted单源更新同样clone整个`AssetRegistryIndex` |
| ASSETREG-P1-036 | rename或多个folded event总是回退full generation，没有transactional multi-delta plan |
| ASSETREG-P1-037 | affected set没有可证明的transitive dependency/readiness/diagnostic closure |
| ASSETREG-P1-038 | dependency path retarget只删除首个old path并append new path，未canonical dedup |
| ASSETREG-P1-039 | dependency owner刷新以`Vec::contains`去重，高fan-out下为二次复杂度 |
| ASSETREG-P1-040 | source entry、affected owner和diagnostic来自HashMap/HashSet，发布顺序未规范化 |

### 7.5 Query、index与snapshot（10项）

| ID | 差距 |
|---|---|
| ASSETREG-P1-041 | type/tag/path/package查询没有二级索引，全部全表扫描 |
| ASSETREG-P1-042 | `entries()`每次分配Vec并排序，没有稳定iterator/visitor |
| ASSETREG-P1-043 | query没有cursor、pagination、result budget、deadline或early-stop callback |
| ASSETREG-P1-044 | filter只是AND条件集合，没有compiled predicate、query plan或invalid-filter error |
| ASSETREG-P1-045 | missing asset和present-with-zero-edge都返回空dependency/referencer结果 |
| ASSETREG-P1-046 | UUID错误/缺失时静默path fallback，未返回`Exact/StaleUuid/PathRecovered/Ambiguous` disposition |
| ASSETREG-P1-047 | referencer排序通过UUID `to_string()`分配，且没有稳定binary sort key合同 |
| ASSETREG-P1-048 | borrowed index/snapshot不携带generation lease，无法与catalog/resource代际关联 |
| ASSETREG-P1-049 | inspection、query、resolve和rebuild错误模型不统一，consumer不能稳定处理availability |
| ASSETREG-P1-050 | 大部分public query只有tests caller，API表面未由Runtime/Editor/cook产品需求资格化 |

### 7.6 Product、Editor、extractor与qualification（10项）

| ID | 差距 |
|---|---|
| ASSETREG-P1-051 | Runtime UI同步调用`entries()`并逐asset打开artifact，启动/会话构建没有批量、预算或异步进度 |
| ASSETREG-P1-052 | `ProjectManager::open`可接受合法陈旧registry，直接consumer在AssetManager full activation前可观察旧state |
| ASSETREG-P1-053 | registry自身没有generation，不能证明与`ProjectCatalogInputGeneration`、ResourceRegistry和event同代 |
| ASSETREG-P1-054 | EditorAssetIndex重新复制/校验rows并维护另一套import state；runtime未提供足够projection contract供Editor只读消费 |
| ASSETREG-P1-055 | project info只公开diagnostic count，没有typed severity/code/generation/remediation摘要 |
| ASSETREG-P1-056 | dependency extractor只覆盖Scene、Material、Model三种handwritten variant |
| ASSETREG-P1-057 | 其他ImportedAsset variant静默返回空依赖，无法区分“已验证无依赖”和“未实现提取” |
| ASSETREG-P1-058 | extractor没有schema/reflection/provider registration、coverage manifest或unknown-field policy |
| ASSETREG-P1-059 | symlink安全测试在创建链接权限不足时直接return并通过，未证明目标平台policy |
| ASSETREG-P1-060 | 没有覆盖durability-deferred-after-publication、multi-sidecar remint fault、reference closure、valid-stale persistence和restart product parity的失败测试 |

## 8. P2差距清单（16项）

| ID | 候选优化 | 前置条件 |
|---|---|---|
| ASSETREG-P2-001 | path/tag/type/source字符串intern与dictionary encoding | schema和memory baseline冻结 |
| ASSETREG-P2-002 | compact columnar row/edge storage，减少HashMap/Vec/UUID重复 | immutable generation contract完成 |
| ASSETREG-P2-003 | mmap/segmented snapshot与按section校验读取 | bounded binary persistence和跨版本format完成 |
| ASSETREG-P2-004 | 有界并行directory walk、meta decode和dependency extraction | deterministic merge与source capability完成 |
| ASSETREG-P2-005 | WAL/delta segment加周期checkpoint，降低单row更新写放大 | crash protocol与generation transaction完成 |
| ASSETREG-P2-006 | tag/type/path/package bitmap或sorted-posting index | representative query workload测量完成 |
| ASSETREG-P2-007 | zero-allocation visitor/batch callback query | lifetime、cancel和reentrancy contract完成 |
| ASSETREG-P2-008 | compiled query cache与cost model | filter schema、secondary index与eviction budget完成 |
| ASSETREG-P2-009 | sharded candidate builder与RCU generation publish | single-generation correctness和retirement完成 |
| ASSETREG-P2-010 | SCC/topological component cache增量失效 | typed dependency graph与closure oracle完成 |
| ASSETREG-P2-011 | per-generation retained bytes、index bytes、edge fan-out与clone telemetry | stable metrics schema完成 |
| ASSETREG-P2-012 | scan/import/query progress projection与Editor可视化 | operation identity、budget和event generation完成 |
| ASSETREG-P2-013 | offline collision/reference repair inspector | P0-001 migration transaction和codec coverage完成 |
| ASSETREG-P2-014 | corruption quarantine explorer与minimal reproducer export | secure evidence/redaction policy完成 |
| ASSETREG-P2-015 | property/fuzz model覆盖query index、delta apply、persistence migration与path normalization | deterministic reference model完成 |
| ASSETREG-P2-016 | 10K/100K/1M asset与高fan-out同场景竞争基准 | correctness、memory、failure和BuildSet qualification先通过 |

## 9. 目标架构

### 9.1 单一Asset Registry Authority

```text
Source/Mount generation
  -> bounded ScanPlan + SourceInventory
  -> Import/Dependency/Identity candidate
  -> AssetRegistryGeneration {
       header(project, BuildSet, schema, source inventory, operation),
       rows(exact type/schema/artifact/state),
       categorized edges,
       secondary indexes,
       diagnostics + redirects/tombstones
     }
  -> durable ProjectGeneration transaction
  -> terminal durability disposition
  -> atomic publish {
       registry + resource + catalog + watcher + event generation
     }
```

`AssetRegistryGeneration`由`Arc`持有并有明确sequence/hash。reader取得generation lease后执行visitor/query；任何跨generation组合必须显式失败或通过delta/rebase，不允许borrow当前ProjectManager内部可变index后再猜代际。

### 9.2 Row、edge与state schema

row至少包含stable asset/subasset identity、logical URI、exact type/schema、source/mount/provider、source digest、importer/config/toolchain/target、artifact manifest、package/owner、availability和revision。edge至少包含category、required、target identity、expected type/schema/revision和provenance。redirect/tombstone是版本化row，不是日志字符串。

load state、artifact residency和registry discovery不应混成一个enum，但必须用同一asset/generation identity连接。Bevy可参考状态分层，Unreal可参考大规模索引与dependency category，Godot可参考source/import currentness；Zircon最终schema必须适合跨进程、跨版本和无UObject运行时。

### 9.3 Prepare、durable、publish

prepare阶段纯构建candidate并校验所有precondition，不修改live state或sidecar。commit阶段在journal中写入sidecar、serialized reference migration、artifact、registry snapshot和receipt。只有commit得到durable terminal后，publication fence才交换registry/resource/catalog/watcher并发布成功event。若平台只能给recovery-pending，则它是明确的非成功terminal state，携OperationId并阻止重复提交。

### 9.4 Query与增量

query编译为受支持的index intersection/union plan，返回visitor/cursor和result budget；常见lookup不分配、不排序整表。incremental输入是带source/mount generation的normalized delta，candidate builder只copy受影响shard/posting/edge component，并通过reference oracle证明与full rebuild等价。full rebuild仍保留为reconciliation oracle，不成为每次rename的默认产品路径。

## 10. 重构里程碑

| Milestone | 内容 | 退出证据 |
|---|---|---|
| M0 · Freeze与fail-close | 固定current row/query/persistence/publication inventory；给dead public incremental API加reachability决策；复现两项P0 | 两个失败测试在未修复代码上稳定RED；所有product caller映射唯一 |
| M1 · Duplicate identity migration | collision report、deterministic owner policy、reference closure、redirect/tombstone、migration receipt和operator flow | multi-codec fixture remint后所有semantic target不变；任一unknown codec零publish |
| M2 · Publication transaction | OperationId、durability terminal、publication fence、idempotency/recovery与event ordering | 每个kill/fault point证明API、live generation、event和restart结果一致 |
| M3 · Registry schema v2 | exact type/schema/source/artifact/state/generation header与bounded binary reader/writer | legacy v1 corpus迁移、unknown/newer version fail-close、valid-stale被拒绝 |
| M4 · Scan authority | SourceInventory、root/case/ignore/orphan policy、budget/progress/cancel和deterministic merge | 多root/case/link/orphan/huge tree跨平台golden一致 |
| M5 · Dependency graph | provider/reflection extractor、category edge、coverage manifest、SCC/closure和typed diagnostics | 所有ImportedAsset variant明确`Complete/Unsupported/Error`；full/targeted graph等价 |
| M6 · Index与query | immutable generation、secondary index、visitor/cursor、typed disposition和retirement | 10K/100K/1M query不全表scan，结果与reference model一致 |
| M7 · Product convergence | Runtime UI、Scene、cook、ProjectInfo和Editor投影迁移到同一generation | open/import/watch/reopen/cook/Editor全链报告同一generation/BuildSet |
| M8 · Scale与竞争资格 | 内存、scan/import/query/update、failure、soak及同硬件参考基准 | correctness gate全绿后再比较CPU/RSS/I/O/p95/p99；不得以静态设计宣称优于Unreal |

## 11. 验收门（40项）

| Gate | 验收内容 |
|---|---|
| G01 | duplicate GUID只生成collision candidate，不在inspection/scan阶段写文件 |
| G02 | remint覆盖全部registered serialized reference codec，semantic target保持不变 |
| G03 | unknown/unavailable codec使migration fail-close且live/disk零变化 |
| G04 | redirect/tombstone有generation、owner、reason、expiry和migration receipt |
| G05 | duplicate owner选择与filesystem enumeration、线程数和平台无关 |
| G06 | `RecoveryDeferred`不能产生普通成功event或普通Err+已发布live state组合 |
| G07 | operation API、event、journal和restart recovery共享同一OperationId |
| G08 | commit前/中/后每个fault point均满足零变化或明确可恢复terminal state |
| G09 | registry/resource/catalog/watcher/event只发布同一project generation |
| G10 | failed、cancelled、superseded、pending与durable成功使用互斥typed disposition |
| G11 | v2 header绑定project、BuildSet、schema、source inventory、root/config和operation |
| G12 | valid-but-stale sidecar/source/artifact输入在publish前被识别并重建或拒绝 |
| G13 | corrupt文件被quarantine并保留hash/evidence，permission错误不自动改写项目 |
| G14 | reader限制bytes、rows、edges、tags、strings、depth、time和allocation |
| G15 | writer峰值RSS与写放大有预算，不依赖pretty JSON whole-state clone |
| G16 | legacy v1、新er unsupported、unknown field和partial file都有明确migration/error |
| G17 | source scan顺序、row顺序、edge顺序和diagnostic顺序跨运行确定 |
| G18 | 多root同URI、case-fold、Unicode normalization和canonical path collision fail-close |
| G19 | ignore/include/hidden/vendor/generated policy进入source inventory identity |
| G20 | symlink/reparse/orphan meta按policy产生typed disposition，不静默pass或全树误报成功 |
| G21 | scan有entries/depth/bytes/time/deadline/cancel/progress与terminal receipt |
| G22 | targeted delta与full rebuild对同一normalized event corpus生成相同state hash |
| G23 | rename/multi-event可在一个transaction delta中处理，未知fold触发有界reconcile |
| G24 | transitive dependency/readiness/diagnostic closure有独立reference oracle |
| G25 | 所有ImportedAsset variant声明Complete/Unsupported/Error，不能静默空依赖 |
| G26 | dependency edge包含category、required、type/schema/revision和provenance |
| G27 | cycle、missing、type mismatch、version mismatch与optional unavailable分开报告 |
| G28 | `entries()`常见产品路径不分配并排序全表 |
| G29 | type/tag/path/package lookup使用二级index并有result/time budget |
| G30 | query返回missing/empty/ambiguous/stale/path-recovered等typed disposition |
| G31 | cursor/visitor持有generation lease，旧generation按reader lifetime安全retire |
| G32 | Runtime UI使用批量异步projection，不在session构建同步逐artifact加载 |
| G33 | Scene reference repair只在exact semantic identity可证明时发生 |
| G34 | EditorAssetIndex成为可重建投影，不再拥有独立runtime truth或持久authority |
| G35 | ProjectInfo暴露bounded structured diagnostics、generation与remediation摘要 |
| G36 | product open/import/reimport/watch/reopen/cook/Editor消费同一generation identity |
| G37 | symlink测试在权限不足时skip有machine-readable原因，required平台必须实际执行 |
| G38 | property/fuzz覆盖index invariant、delta/full等价、migration和malformed persistence |
| G39 | 10K/100K/1M与高fan-out基准记录CPU、RSS、I/O、allocation、p50/p95/p99和source hash |
| G40 | `git diff --check`、frontmatter path、link、count、fingerprint和共享worktree复核通过 |

## 12. 状态与限制

- `review_status: review_complete`只表示47个focused文件与列出的reference source已完成本轮静态审查；
- `implementation_status: pending`，2项P0、60项P1、16项P2均未实现；
- Runtime04的11项P1/1项P2、Runtime24/25的共享合同和Editor04三truth finding不在本篇重复累计；
- current product full generation已有projected inventory与multi-file journal，实施时必须保留，不得退回逐文件裸写；
- `dev/Graphics`只提供Unity Graphics package消费者合同，不包含Unity核心AssetDatabase源码；
- 本轮未运行Cargo或动态验证，所有性能结论仍是缺证据/待资格，不是已发生回归或“已优于Unreal”的声明；
- MVP 00完成前，本篇只允许继续做read-only review/test design，不授权production实现。
