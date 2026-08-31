---
title: Runtime Asset Registry、Index、Persistence、Rebuild、Incremental、Query 与 Product Integration Current Source Review
category: zircon_runtime
report_id: Runtime122
review_date: 2026-08-23
baseline_head: 1354e50da53db3dad1dc25a6c9e375942ba04d35
baseline_epoch: 368
supersedes:
  - docs/plans/optimize/zircon_runtime/51-runtime-asset-registry-index-persistence-rebuild-incremental-query-product-integration-review.md
related_code:
  - zircon_runtime/src/asset/registry
  - zircon_runtime/src/asset/mutation
  - zircon_runtime/src/asset/project/manager
  - zircon_runtime/src/asset/project/catalog_input_generation.rs
  - zircon_runtime/src/asset/project/generation_observation.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager
  - zircon_runtime/src/asset/pipeline/manager/resource_sync
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui.rs
  - zircon_runtime/src/scene/world/project_io/references.rs
  - zircon_editor/src/core/asset
  - zircon_editor/src/ui/host/editor_asset_manager
  - zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs
tests:
  - zircon_runtime/src/asset/tests/registry_index
  - zircon_runtime/src/asset/tests/pipeline/manager/watcher.rs
  - zircon_runtime/src/asset/tests/project/manager/catalog_input_generation.rs
  - zircon_runtime/src/asset/tests/project/manager/full_generation.rs
  - zircon_runtime/src/asset/tests/project/manager/targeted_import.rs
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/optimize/zircon_runtime/51-runtime-asset-registry-index-persistence-rebuild-incremental-query-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/optimize/zircon_runtime/51/2026-08-20-linear-dependency-owner-dedup.md
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
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 99w · Runtime Asset Registry Current Source Review

## 1. 结论

当前 Asset Registry 不是空壳。UUID/path/source/AssetId 反查、referencer 反向边、typed persistence error、duplicate 检测、projected sidecar inventory、multi-file durable journal、targeted generation、不可变 `ProjectCatalogInputGeneration`、delta publication 与 generation-bound reference repair 都是真实底座。当前 shared worktree 还新增了 Runtime-owned delete/relocation preflight、保留 UUID 的显式单文件 source relocation、ResourceRegistry locator rename 以及 Editor 对 Runtime mutation plan 的投影。这些能力应被吸收到唯一内容 authority，不应退回逐文件裸写或 Editor 自建写 authority。

但 Runtime51 的两个数据完整性 P0 都没有关闭。duplicate GUID 路径仍会随机 remint 并写回 `.zmeta`，没有完整 serialized inbound reference closure、redirect/tombstone、operator decision 与原子 migration receipt；显式 relocation 只证明“已知 UUID 的单文件移动可保留身份”，不能证明 collision remint 安全。`RecoveryDeferred` 仍在 ProjectManager/ProjectAssetManager 已安装 candidate、资源、watcher、catalog generation 或 renamed event 后才由 `ensure_durable()` 转为错误，继续形成 API 失败而 live generation 已推进的 unknown outcome。

Registry 本体仍是 JSON v1 项目扫描缓存，不是工程级内容数据库。row 没有 exact type/schema、artifact/cook identity、source/mount/provider、BuildSet、categorized dependency、availability、redirect/tombstone或 registry generation；合法陈旧快照仍可被接受；scan 无确定性目录顺序、预算、进度或取消；query 仍主要在 HashMap 上全表扫描、分配和排序。新的 sharded `ProjectCatalogInputGeneration` 改善了产品 catalog publication，却没有改变 `AssetRegistryIndex` whole-map clone 和无 generation lease 的事实。

产品闭环有真实但有限的进展。Scene reference 不再把错误 UUID 静默退回 path；Runtime reference resolver 会区分 path occupied、conflict 与可证明的 repair。Editor project sync 已消费 Runtime generation/delta，而不是自己扫描 source generation；dependency/referencer query 开始被 targeted import 和 Runtime mutation preflight 使用。可是 Runtime UI 仍同步 `entries()` 后逐 artifact 加载，Editor 仍维护复制的 catalog/index 状态，多数 public query 仍只有 tests caller，watch rename/multi-event 仍回退 full reconciliation。

本轮裁决为 **2 P0 Open；55 P1 Open、5 P1 Partial、0 Closed；16 P2 Open；36 Gate Fail、4 Gate Partial**。Partial 只代表当前源码出现可保留切片，不代表里程碑验收。目标仍是有 schema、source inventory 与 generation lease 的不可变 `AssetRegistryGeneration`，在 terminal durability 后原子发布 registry/resource/catalog/watcher/event 同一代，并用 indexed、cursor/visitor、bounded query 服务 Runtime、Editor 与 cook 产品链。

本轮只做 current-source 静态审查与文档记录，没有修改 production、tests、Cargo、ABI 或参考源码；没有运行 Cargo、真实 project import/watch/Editor、fault injection、crash recovery、跨进程竞争、soak 或 benchmark。MVP `00` 仍为 `in_progress`，F0-F5 仍 blocked；本文不把 active shared-worktree candidate、ignored benchmark 或静态测试写成 accepted milestone，也不展开 tooling 优化。

## 2. 审查边界与物理冻结

| 范围 | 文件 / 行 / bytes / tests | fingerprint |
|---|---:|---|
| Registry production owner | 17 / 2,399 / 83,680 / 7 | `dd5dde98e5952de8eeb8d110566c5918088c2b581836b5c83b8899395b64560c` |
| focused direct tests | 10 / 2,926 / 103,765 / 55 | `93b1d98bf468a1cd0b215d362b8c40820f5e3185052e60549c1e106f61d0f91a` |
| Project/AssetManager product chain | 59 / 10,269 / 377,368 / 41 | `17c6dc6cc96d4d34f419cfea2fafaaf30292b4a981918d60a35fe7d9eecae4f8` |
| Runtime/Editor/App consumers | 16 / 3,804 / 133,885 / 22 | `499de5126374fec7feebfeeccaf95eedaeee4179a896efbe8d7bd0bfa81fcc27` |
| 五引擎参考实现 | 16 / 16,297 / 655,698 / 19 | `ca14b25504c159c7f27841dfdda363cf19a86a7a369ed97c0afc99756678c184` |

fingerprint 算法：仓库相对路径转 `/` 后转小写，以 ordinal 排序去重；每项编码为 `lowercase-path + NUL + lowercase per-file SHA-256`，以 LF 连接且末尾无 LF，再计算 UTF-8 SHA-256。逻辑行按 LF 分割计数。它只冻结本轮实际读取集合，不是 registry generation、artifact、BuildSet、ABI 或 release identity。

从 Runtime51 基线 `bea1acf91b909525ab1759e2c800858b0eda6528` 到本会话注册基线 HEAD `1354e50da53db3dad1dc25a6c9e375942ba04d35`，focused asset/project/editor 范围有 65 个文件变化、1,339 insertions 与 793 deletions；当前又有大量未提交及未跟踪的 registry relocation、Runtime mutation、ProjectManager、ProjectAssetManager、resource reconciliation 和 Editor refactor 代码。因此 Runtime51 的 47 文件 fingerprint 不可复用。最终验证时共享 `main` 前进到 `01d3ebc247f8f6027f4eacc47567a7ceb2a11621`，该提交只修改 `docs/plans/mvp/06/failure-2026-08-01-f5-evidence-package-incomplete.md`，没有触及五组冻结证据、本报告或两级索引。

共享实现 owner `optimize-runtime-full-post-main-integration-r3-01a00797-20260821` 仍覆盖 `registry/targeted.rs`、ProjectManager、ProjectAssetManager/watch 与 Runtime51 子计划路径。本报告只读其 current worktree 快照且不写这些 owner；`source_recheck_required` 置 true，任何实现开始前都必须重冻并重判，不能把本文 fingerprint 当 release identity。

## 3. 当前产品纵切面

```text
source/watch/editor request
  -> ProjectManager candidate
     -> ProjectedMetaInventory / targeted import / explicit relocation
     -> AssetRegistryIndex + ProjectCatalogInputGeneration
     -> durable journal(sidecar + artifact + registry)
  -> ProjectAssetManager installs project/resources/watchers
  -> publishes catalog generation / asset event
  -> ensure_durable()                 // P0: terminal durability仍然过晚

Runtime consumers
  -> Scene exact UUID resolution + bounded repair candidates
  -> mutation preflight via referencer query
  -> Runtime UI entries() + synchronous artifact loads

Editor consumers
  -> Runtime generation/delta projection
  -> copied Editor catalog + EditorAssetIndex
  -> Runtime-owned mutation/relocation request
```

独立 `rebuild_from_project` 和 public `AssetRegistryIndex::apply_watch_changes` 仍形成第二套 mutation/reconciliation 语义。前者会在 registry persist 前直接改多个 sidecar；后者无 product caller、clone 全 index、重扫全部 meta 并执行 duplicate normalization。正确收敛方向是 inspection 纯读，所有写入进入唯一 project generation transaction，full rebuild 只作为 reconciliation oracle。

## 4. 当前源码事实与状态变化

| 主题 | 当前证据 | 裁决 |
|---|---|---|
| duplicate identity | `normalize_duplicate_guids`、targeted/projected normalization 仍生成随机 UUID；没有 reference closure/redirect/receipt | `ASSETREG-P0-001 Open` |
| durability/publication | open、full/targeted、watch、relocation 均可先 install/publish，再 `ensure_durable()` | `ASSETREG-P0-002 Open` |
| persistence | `PersistedAssetRegistry { format_version, entries }` 仍为 v1 whole JSON；无 currentness header、budget、migration/quarantine | P1 保持 Open |
| scan/rebuild | `read_dir` 未排序；orphan meta 静默跳过；随机 duplicate owner；单 link 可中止全 scan | P1 保持 Open |
| index/query | UUID/path/source/referencer HashMap；type/tag/path/package扫描；`entries()`分配排序；无 cursor/generation | P1 保持 Open |
| explicit relocation | 单文件 `res://` relocation 校验 digest、保留 UUID、移动 source/sidecar/registry/resource locator 并发布 generation | `P1-036 Partial`，不关闭 duplicate P0 |
| dependency owner dedup | `HashSet` 替代 `Vec::contains`，有行为测试和 ignored benchmark | `P1-039 Partial`；managed validation/release performance 未完成 |
| reference resolution | GUID 成为 authoritative；错误 GUID 不再 path fallback；只对同 GUID/subasset 安全 hint 生成 repair | `P1-046 Partial`、`G33 Partial` |
| product query reachability | targeted import 使用 dependency query，Runtime mutation preflight 使用 referencer query | `P1-050 Partial`；多数 query 仍 test-only |
| Editor projection | project sync 消费 Runtime `ProjectCatalogInputGeneration` 与 delta | `P1-054 Partial`、`G34 Partial`；复制 catalog/index 仍存在 |
| watch delta | exactly one Added/Modified/Removed 可 targeted；rename/previous URI/multi-event 回退 full reconciliation | `G23 Partial` |
| typed query disposition | UUID resolve 更严格，但 dependency/referencer missing 与 empty 仍相同 | `G30 Partial` |

`docs/zircon_runtime/asset/registry.md` 当前声称 candidate 会先持久化、成功后才交换 live state。该陈述对普通 Durable 路径描述了意图，但对 `RecoveryDeferred` 不成立：ProjectAssetManager 已经可能安装并发布 candidate 后才返回 durability error。修复 P0-002 前，模块文档不得被当作已满足的 publication contract。

## 5. 五引擎参考差异

| 参考 | 已核对能力 | Zircon 应吸收的合同 | 边界 |
|---|---|---|---|
| Unreal AssetRegistry | package/path/class/tag 次级索引；category/property dependency query；serialization options；gatherer cache/progress/async result/event/memory telemetry | immutable indexed state、typed edge/query、可配置持久 schema、scan progress/cache/内存预算 | 不复制 UObject/package 历史债务或全局 singleton |
| Bevy AssetServer | typed path、load/dependency/recursive states、loader dependency hash、dependents、change watch；named source 的 reader/writer/watch capability | discovery/loaded/cooked state 分层，source capability 与 typed failure | `TypeId`/进程内 server 不能当长期持久 schema |
| Godot EditorFileSystem/ResourceUID | UID/type/mtime/import mtime/md5/destination/validity/dependency；scan/import progress/change action；UID↔path persistent cache | currentness inputs、typed scan action、stable identity reverse mapping 与 cache update | 不复制 EditorFileSystem 单体或裸路径全局 authority |
| Fyrox ResourceManager/Registry/Graph | registry UUID→path persistence、event broadcaster、watcher、显式 move validation、reflection-style dependency graph | async event、move validation、schema-driven dependency discovery | 轻量 UUID map 只作二级证据，不是目标上限 |
| Unity Graphics consumer | ShaderGraph importer 声明 source/artifact/custom dependencies 与 subassets；pipeline hash invalidation；batch reimport progress | source/artifact/custom edge、subasset identity、environment hash 与 batch scope | 本地 Graphics 不含 Unity 核心 AssetDatabase，不能反推其内部实现 |

共同工程底线是 registry state 可证明 current、identity migration 保持语义目标、dependency 可分类、scan/import 可预算与取消、query 可索引且 generation-bound、publication 与 durability 属于同一 operation。Zircon 可以用紧凑 Rust layout、immutable generation 和批量 visitor 争取更低 CPU/RSS，但只有同 corpus、同硬件、同正确性门禁下的动态证据才能支持“优于 Unreal”。

## 6. Canonical owner 边界

| 事实 | Canonical owner | Runtime122 纵切面 |
|---|---|---|
| exact asset type/schema、artifact/cook identity、typed dependency、last-good | Runtime04 | registry row/index/persistence 承载与查询 |
| stable UUID、owner/domain/generation/exhaustion、跨 schema migration | Runtime24 | duplicate/remint reference closure、redirect 与 transaction |
| filesystem/source/mount、root collision、watch、durable writer | Runtime25 | scan plan、source inventory 与 publication 消费 |
| Editor catalog/index/import/reference workflow truth | Editor04 | Runtime immutable generation 作为唯一上游 authority |
| product transaction、unknown outcome、operation disposition | shared transaction/operation owner | `RecoveryDeferred` 晚于 live publication 的具体 P0 |
| whole-index clone/query/Editor projection 性能 | `PERF-MVP-556` | query/index/cursor 的 correctness 与产品资格 |

Runtime122 不重复统计父 owner 的 finding，也不为 Editor 创建第二套持久 asset authority。显式 relocation、delete preflight 与 reference repair 必须消费同一 identity/schema/transaction 合同。

## 7. P0 差距（2 项）

| ID | 状态 | 当前证据与必须达到的修复合同 |
|---|---|---|
| ASSETREG-P0-001 | Open | duplicate normalization 仍随机 remint 并写 sidecar，未扫描/迁移所有 serialized inbound UUID reference。必须先生成只读 collision report，由确定性 policy/operator 选择 owner，构建全 repository closure，原子提交 sidecar、文档、registry、redirect/tombstone 与 receipt；unknown codec 使 publish fail closed。 |
| ASSETREG-P0-002 | Open | `RecoveryDeferred` 可在 project/resource/watcher/catalog/event 已安装发布后才变 Err。必须以 publication fence 先取得 `Durable` 或显式 `AcceptedRecoveryPending(OperationId)`；API、event、live state 与 restart recovery 共享 disposition，普通失败不得留下不可见的已发布 candidate。 |

## 8. P1 差距（60 项）

### 8.1 Identity、row 与 dependency graph

| ID | 状态 | 差距与硬切目标 |
|---|---|---|
| ASSETREG-P1-001 | Open | row 只有宽泛 `AssetKind`；加入稳定 exact `AssetTypeId`。 |
| ASSETREG-P1-002 | Open | 缺 `SchemaId`、codec/importer schema version 与 migration chain。 |
| ASSETREG-P1-003 | Open | 缺 artifact manifest hash、cook key、target/profile 与 toolchain identity。 |
| ASSETREG-P1-004 | Open | 缺 source instance、mount/provider、physical identity 与 trust provenance。 |
| ASSETREG-P1-005 | Open | 缺 package revision、registry generation、BuildSet 与 project identity。 |
| ASSETREG-P1-006 | Open | dependency 仍是裸 UUID；加入 category、required、expected type/schema/revision 与 provenance。 |
| ASSETREG-P1-007 | Open | edge 顺序未冻结 canonical sort 与 duplicate diagnostic。 |
| ASSETREG-P1-008 | Open | rename/remint/delete 缺 redirect、tombstone、retirement epoch 与 expiry policy。 |
| ASSETREG-P1-009 | Open | row 不表达 discovered/importing/ready/stale/failed/missing-artifact。 |
| ASSETREG-P1-010 | Open | diagnostic 缺稳定 ID、generation、source span、owner、operation 与 remediation。 |

### 8.2 Persistence、currentness 与 recovery

| ID | 状态 | 差距与硬切目标 |
|---|---|---|
| ASSETREG-P1-011 | Open | 合法陈旧 registry 不核对 sidecar/source/artifact currentness。 |
| ASSETREG-P1-012 | Open | header 不绑定 project/root/config/importer set/BuildSet/catalog generation。 |
| ASSETREG-P1-013 | Open | whole-file JSON read/decode 无 byte/entry/time/cancel admission。 |
| ASSETREG-P1-014 | Open | tag/dependency/string/path/diagnostic 无 per-row/total budget。 |
| ASSETREG-P1-015 | Open | I/O、permission、decode、version、duplicate 仍共享自动 rebuild disposition。 |
| ASSETREG-P1-016 | Open | corrupt/unsupported 文件无 quarantine、backup、hash、raw evidence/operator action。 |
| ASSETREG-P1-017 | Open | unsupported version 直接 rebuild，无 versioned migration 与 reader/writer window。 |
| ASSETREG-P1-018 | Open | `CorruptPersistenceRebuilt` 不进入持久 recovery receipt。 |
| ASSETREG-P1-019 | Open | save clone/sort 全部 row 并 pretty JSON 全量编码，无 RSS/写放大预算。 |
| ASSETREG-P1-020 | Open | load/rebuild 无 read-only/degraded/last-good mode，权限错误仍可能触发副作用。 |

### 8.3 Scan、rebuild 与 root policy

| ID | 状态 | 差距与硬切目标 |
|---|---|---|
| ASSETREG-P1-021 | Open | `read_dir`/meta path 未排序，owner 与 diagnostic 顺序依赖 filesystem enumeration。 |
| ASSETREG-P1-022 | Open | 多 root 同 AssetUri 缺 root priority/collision report。 |
| ASSETREG-P1-023 | Open | 缺 case-fold、Unicode normalization 与 canonical URI collision policy。 |
| ASSETREG-P1-024 | Open | 缺 include/exclude/hidden/vendor/generated/source-control-ignore policy schema。 |
| ASSETREG-P1-025 | Open | 单 symlink/reparse 可中止全 scan，缺 per-entry typed disposition。 |
| ASSETREG-P1-026 | Open | orphan `.zmeta` 仍静默跳过，无 tombstone/diagnostic/recovery action。 |
| ASSETREG-P1-027 | Open | rebuild 信任 sidecar digest，不验证 source/artifact 对应关系。 |
| ASSETREG-P1-028 | Open | scan 无 entries/depth/bytes/time/deadline/cancel/progress budget。 |
| ASSETREG-P1-029 | Open | metadata load/decode 串行，无 bounded I/O/CPU pipeline 或 deterministic merge。 |
| ASSETREG-P1-030 | Open | public rebuild 在 registry persist 前直接改多份 sidecar，未进入唯一 generation transaction。 |

### 8.4 Incremental、watch 与 closure

| ID | 状态 | 差距与硬切目标 |
|---|---|---|
| ASSETREG-P1-031 | Open | public `apply_watch_changes` 无 product caller，却形成第二套 mutation authority。 |
| ASSETREG-P1-032 | Open | helper 对小变更 clone 整张 index。 |
| ASSETREG-P1-033 | Open | helper 重扫全部 meta 并 duplicate-normalize，不是真正 delta apply。 |
| ASSETREG-P1-034 | Open | empty change 不验证 overflow、root generation 或 source drift。 |
| ASSETREG-P1-035 | Open | targeted 单源更新仍 clone 整个 `AssetRegistryIndex`。 |
| ASSETREG-P1-036 | Partial | 显式单文件 relocation 已形成 transactional delta 并保留 UUID；watch rename、previous URI、multi-event 与 compound source 仍回退 full reconciliation。 |
| ASSETREG-P1-037 | Open | affected set 无可证明 transitive dependency/readiness/diagnostic closure。 |
| ASSETREG-P1-038 | Open | dependency path retarget 未提供全量 canonical dedup 与 typed conflict。 |
| ASSETREG-P1-039 | Partial | owner 刷新已用 `HashSet` 去重并有行为测试/ignored benchmark；managed validation 与 release corpus performance 尚未完成。 |
| ASSETREG-P1-040 | Open | source/affected owner/diagnostic 发布顺序仍未全面规范化。 |

### 8.5 Query、index 与 snapshot

| ID | 状态 | 差距与硬切目标 |
|---|---|---|
| ASSETREG-P1-041 | Open | type/tag/path/package 查询仍无二级索引。 |
| ASSETREG-P1-042 | Open | `entries()` 每次分配 Vec 并排序，无稳定 iterator/visitor。 |
| ASSETREG-P1-043 | Open | query 无 cursor/pagination/result budget/deadline/early stop。 |
| ASSETREG-P1-044 | Open | filter 无 compiled predicate/query plan/invalid-filter error。 |
| ASSETREG-P1-045 | Open | missing asset 与 present-with-zero-edge 仍都返回空集合。 |
| ASSETREG-P1-046 | Partial | UUID resolve 已不再静默 path fallback，并返回 not-found/conflict/repair 语义；全部 query 尚未统一 `Exact/Missing/Empty/Ambiguous/Stale/Recovered` disposition。 |
| ASSETREG-P1-047 | Open | referencer sort 仍通过 UUID string 分配，缺 binary sort key contract。 |
| ASSETREG-P1-048 | Open | borrowed index/snapshot 不携 generation lease。 |
| ASSETREG-P1-049 | Open | inspection/query/resolve/rebuild error model 未统一 availability。 |
| ASSETREG-P1-050 | Partial | dependency query 已有 targeted product caller，referencer query 已被 Runtime mutation preflight 消费；多数 public query 仍 test-only，API 尚未完成产品资格化。 |

### 8.6 Product、Editor、extractor 与 qualification

| ID | 状态 | 差距与硬切目标 |
|---|---|---|
| ASSETREG-P1-051 | Open | Runtime UI 同步 `entries()` 并逐 artifact 打开，缺 batch/budget/async progress。 |
| ASSETREG-P1-052 | Open | `ProjectManager::open` 可接受合法陈旧 registry，full activation 前可观察旧 state。 |
| ASSETREG-P1-053 | Open | registry 无 generation，不能证明与 catalog/resource/event 同代。 |
| ASSETREG-P1-054 | Partial | Editor sync 已消费 Runtime generation/delta；Editor 仍复制 catalog/index 并维护另一套 import state，projection contract 未收敛。 |
| ASSETREG-P1-055 | Open | ProjectInfo 只有 diagnostic count，无 typed severity/code/generation/remediation。 |
| ASSETREG-P1-056 | Open | dependency extractor 仍只覆盖 Scene/Material/Model handwritten variant。 |
| ASSETREG-P1-057 | Open | 其他 ImportedAsset variant 静默空依赖，无法区分 Complete 与 Unsupported。 |
| ASSETREG-P1-058 | Open | extractor 无 schema/reflection/provider registration/coverage manifest/unknown-field policy。 |
| ASSETREG-P1-059 | Open | symlink test 权限不足时直接 return，通过不等于目标平台 policy 已验证。 |
| ASSETREG-P1-060 | Open | 缺 durability-deferred-after-publication、multi-sidecar remint fault、reference closure、valid-stale persistence 与 restart product parity 失败测试。 |

## 9. P2 差距（16 项）

| ID | 状态 | 候选优化与前置条件 |
|---|---|---|
| ASSETREG-P2-001 | Open | path/tag/type/source string intern 与 dictionary encoding；先冻结 schema/memory baseline。 |
| ASSETREG-P2-002 | Open | compact columnar row/edge storage；先完成 immutable generation。 |
| ASSETREG-P2-003 | Open | mmap/segmented snapshot 与 section validation；先完成 bounded binary persistence。 |
| ASSETREG-P2-004 | Open | bounded parallel walk/decode/extract；先完成 deterministic merge/source capability。 |
| ASSETREG-P2-005 | Open | WAL/delta segment + checkpoint；先完成 crash protocol/generation transaction。 |
| ASSETREG-P2-006 | Open | bitmap/sorted posting index；先以 representative workload 测量。 |
| ASSETREG-P2-007 | Open | zero-allocation visitor/batch query；先冻结 lifetime/cancel/reentrancy。 |
| ASSETREG-P2-008 | Open | compiled query cache/cost model；先有 filter schema/index/eviction budget。 |
| ASSETREG-P2-009 | Open | sharded candidate builder/RCU publish；先证明 single-generation correctness/retirement。 |
| ASSETREG-P2-010 | Open | SCC/topological cache incremental invalidation；先完成 typed graph/closure oracle。 |
| ASSETREG-P2-011 | Open | retained/index bytes、fan-out、clone telemetry；先冻结 metrics schema。 |
| ASSETREG-P2-012 | Open | scan/import/query progress projection；先有 operation/budget/event generation。 |
| ASSETREG-P2-013 | Open | offline collision/reference repair inspector；先关闭 P0-001。 |
| ASSETREG-P2-014 | Open | quarantine explorer/minimal reproducer export；先有 evidence/redaction policy。 |
| ASSETREG-P2-015 | Open | property/fuzz model覆盖 index、delta、migration、path normalization。 |
| ASSETREG-P2-016 | Open | 10K/100K/1M/high-fan-out competition benchmark；correctness/memory/failure/BuildSet 先过门。 |

## 10. 目标架构

```text
Source/Mount Generation + SourceInventory
  -> bounded ScanPlan / NormalizedDelta
  -> Identity + Import + Dependency candidate
  -> AssetRegistryGeneration {
       Header(project, BuildSet, schema, source inventory, operation),
       Rows(exact type/schema/source/artifact/state),
       CategorizedEdges + SecondaryIndexes,
       Diagnostics + Redirects + Tombstones
     }
  -> ProjectGenerationTransaction {
       sidecars + serialized reference migration + artifacts + registry + receipt
     }
  -> terminal durability disposition
  -> atomic publish(registry + resource + catalog + watcher + events)
  -> leased cursor/visitor query -> Runtime / Editor projection / cook
```

`AssetRegistryGeneration` 是 Runtime 唯一 authority，reader 持 `Arc` generation lease；跨 generation 组合必须显式 rebase 或失败。Editor 只维护可丢弃、可重建的视图缓存。load/residency/cook state 与 discovery row 分层，但共享 asset/generation/schema identity。

duplicate、rename、delete 和 reference repair 必须进入同一 mutation planner。prepare 纯读并生成 closure/precondition；commit 在 journal 内完成全部文件与 receipt；只有 terminal durability 后 publication fence 才交换同一代。full rebuild 是 normalized delta 的 reconciliation oracle，不是常规 rename 默认路径。

query 编译为 index intersection/union plan，返回 generation-bound cursor/visitor 与 typed disposition；常见 lookup 不全表分配排序。实现性能优化前，先建立结果等价、生命周期、budget、cancel、reentrancy 与 retirement 合同。

## 11. 重构里程碑

| Milestone | 内容 | 退出证据 |
|---|---|---|
| M122-0 · Freeze/fail-close | 固定 row/query/persistence/publication/caller inventory；为两项 P0 建 RED repro | duplicate/reference 与 deferred-publication 测试在旧代码稳定失败；owner/deletion matrix 完成 |
| M122-1 · Identity mutation transaction | collision report、deterministic owner、codec closure、redirect/tombstone、operator receipt；统一 relocation/delete/repair | multi-codec semantic target 不变；unknown codec 零 publish |
| M122-2 · Publication transaction | OperationId、terminal durability、publication fence、idempotency/recovery/event ordering | 每个 fault/kill point下 API/live/event/restart disposition 一致 |
| M122-3 · Registry schema v2 | exact type/schema/source/artifact/state/generation header 与 bounded reader/writer | v1 migration、newer/unknown fail-close、valid-stale reject |
| M122-4 · Scan/source authority | SourceInventory、root/case/ignore/orphan/link policy、budget/progress/cancel、deterministic merge | 跨平台多 root/case/link/orphan/huge-tree golden 一致 |
| M122-5 · Dependency graph | provider/reflection extractor、categorized edge、coverage manifest、SCC/closure、typed diagnostics | 所有 ImportedAsset 明确 Complete/Unsupported/Error；full/targeted graph 等价 |
| M122-6 · Index/query/generation | immutable generation、secondary indexes、cursor/visitor、typed disposition、retirement | 10K/100K/1M query 不全表 scan且与 reference model 等价 |
| M122-7 · Product convergence | Runtime UI、Scene、cook、ProjectInfo、Editor 投影迁移到同一 generation | open/import/watch/reopen/cook/Editor 报告同一 generation/BuildSet |
| M122-8 · Dynamic qualification | fault/crash/cross-process/soak、CPU/RSS/I/O/allocation/p95/p99 同 corpus 基准 | correctness gate 全绿后才比较参考引擎性能，不以静态设计宣称超越 |

没有任何 M122 milestone 在本文中被标记 accepted；active shared-worktree 代码必须进入其 managed plan，带基线、validator 与 release evidence 后才能改变该状态。

## 12. 验收门禁（40 项）

| Gate | 状态 | 验收条件 |
|---|---|---|
| ASSETREG-G01 | Fail | duplicate GUID 只生成 collision candidate，inspection/scan 不写文件。 |
| ASSETREG-G02 | Fail | remint 覆盖全部 registered serialized codec，semantic target 不变。 |
| ASSETREG-G03 | Fail | unknown/unavailable codec 使 migration fail closed且 live/disk 零变化。 |
| ASSETREG-G04 | Fail | redirect/tombstone 有 generation/owner/reason/expiry/receipt。 |
| ASSETREG-G05 | Fail | duplicate owner 与 enumeration/thread/platform 无关。 |
| ASSETREG-G06 | Fail | `RecoveryDeferred` 不产生普通成功 event 或 Err+已发布 live 组合。 |
| ASSETREG-G07 | Fail | API/event/journal/restart recovery 共享 OperationId。 |
| ASSETREG-G08 | Fail | commit 每个 fault point 都是零变化或明确可恢复 terminal。 |
| ASSETREG-G09 | Fail | registry/resource/catalog/watcher/event 只发布同一 generation。 |
| ASSETREG-G10 | Fail | failed/cancelled/superseded/pending/durable 为互斥 typed disposition。 |
| ASSETREG-G11 | Fail | v2 header 绑定 project/BuildSet/schema/source inventory/root/config/operation。 |
| ASSETREG-G12 | Fail | valid-but-stale sidecar/source/artifact 在 publish 前被识别。 |
| ASSETREG-G13 | Fail | corrupt quarantine 保留 evidence；permission error 不自动改写项目。 |
| ASSETREG-G14 | Fail | reader 限制 bytes/rows/edges/tags/strings/depth/time/allocation。 |
| ASSETREG-G15 | Fail | writer RSS/写放大有预算，不依赖 pretty JSON whole clone。 |
| ASSETREG-G16 | Fail | legacy v1/newer/unknown/partial file 有明确 migration/error。 |
| ASSETREG-G17 | Fail | scan/row/edge/diagnostic 顺序跨运行确定。 |
| ASSETREG-G18 | Fail | 多 root URI/case-fold/Unicode/canonical collision fail closed。 |
| ASSETREG-G19 | Fail | ignore/include/hidden/vendor/generated policy 进入 SourceInventory identity。 |
| ASSETREG-G20 | Fail | link/orphan meta 产生 typed disposition，不静默 pass 或误报全树成功。 |
| ASSETREG-G21 | Fail | scan 有 entries/depth/bytes/time/deadline/cancel/progress/receipt。 |
| ASSETREG-G22 | Fail | targeted delta 与 full rebuild 对 normalized corpus 生成相同 state hash。 |
| ASSETREG-G23 | Partial | 显式单文件 relocation 已是 transaction delta；watch rename/multi-event/unknown fold 仍缺有界 reconcile。 |
| ASSETREG-G24 | Fail | transitive dependency/readiness/diagnostic closure 有 reference oracle。 |
| ASSETREG-G25 | Fail | 每个 ImportedAsset 声明 Complete/Unsupported/Error，不能静默空依赖。 |
| ASSETREG-G26 | Fail | edge 包含 category/required/type/schema/revision/provenance。 |
| ASSETREG-G27 | Fail | cycle/missing/type/version/optional unavailable 分开报告。 |
| ASSETREG-G28 | Fail | `entries()` 常见产品路径不分配排序全表。 |
| ASSETREG-G29 | Fail | type/tag/path/package lookup 使用 index 且有 result/time budget。 |
| ASSETREG-G30 | Partial | GUID resolve 已区分 not-found/conflict/repair；dependency/referencer 等 query 尚未统一 missing/empty/ambiguous/stale/recovered。 |
| ASSETREG-G31 | Fail | cursor/visitor 持 generation lease，旧 generation 安全 retire。 |
| ASSETREG-G32 | Fail | Runtime UI 使用 batch async projection，不同步逐 artifact 加载。 |
| ASSETREG-G33 | Partial | repair 只在同 GUID/subasset hint 可证明时生成；完整 codec closure 与持久 mutation receipt 尚缺。 |
| ASSETREG-G34 | Partial | Editor 已消费 Runtime generation/delta；复制 catalog/index/import truth 仍未完全降为可重建投影。 |
| ASSETREG-G35 | Fail | ProjectInfo 暴露 bounded structured diagnostics/generation/remediation。 |
| ASSETREG-G36 | Fail | open/import/reimport/watch/reopen/cook/Editor 消费同一 generation identity。 |
| ASSETREG-G37 | Fail | symlink test 权限不足时 machine-readable skip，required 平台必须实际执行。 |
| ASSETREG-G38 | Fail | property/fuzz 覆盖 index invariant、delta/full、migration、malformed persistence。 |
| ASSETREG-G39 | Fail | 10K/100K/1M/high-fan-out 记录 CPU/RSS/I/O/allocation/p50/p95/p99/source hash。 |
| ASSETREG-G40 | Fail | managed validation、frontmatter/link/count/fingerprint/shared-worktree/release evidence 全通过。 |

## 13. 当前状态与首个实施切片

- Review：`review_complete`。
- Implementation：`pending`。
- Finding：`2 P0 Open`；`55 P1 Open`、`5 P1 Partial`、`0 Closed`；`16 P2 Open`。
- Gate：`36 Fail`、`4 Partial`。
- 可保留进展：显式 UUID-preserving source relocation、generation-bound strict reference repair、dependency owner HashSet dedup、Runtime mutation preflight query caller、Editor Runtime-generation projection。
- 未关闭底线：duplicate remint semantic closure 与 durability-before-publication 两项 P0。
- 首个实施切片：执行 M122-0，先建立两项 P0 的 RED repro、唯一 owner/caller/deletion matrix 与 publication ordering trace；不得先扩充 query facade、再建 Editor authority 或以 relocation happy path 代替 collision migration。
- `P1-039` 的源码实现和 ignored benchmark 只构成 Partial；managed validation、同 corpus release performance 与计划验收完成前不得写 Closed。
- 本轮没有 accepted milestone output；`docs/plans/optimize/zircon_runtime/51/2026-08-20-linear-dependency-owner-dedup.md` 的 pending validation 状态保持权威。

## 14. 限制与 currentness

- `source_recheck_required: true`：本轮审查的是包含 active shared-worktree 修改与 untracked candidate 的瞬时快照；实现和验收前必须重新计算五组 fingerprint。
- 未运行 Cargo 或动态产品验证，所有功能状态来自逐文件静态证据；Partial 不等于 passing。
- 当前模块文档的 durability-before-swap 陈述强于 `RecoveryDeferred` 真实实现，需由 P0-002 owner 后续同步修正文档或实现。
- `dev/Graphics` 只提供 Unity Graphics package consumer 证据，不含核心 AssetDatabase；本文没有凭缺失源码推断其内部性能或事务实现。
- 本报告不授权 production 修改；MVP `00` 完成前，advanced work 仍限于 read-only review/test design 与已登记 owner 的 managed implementation。
