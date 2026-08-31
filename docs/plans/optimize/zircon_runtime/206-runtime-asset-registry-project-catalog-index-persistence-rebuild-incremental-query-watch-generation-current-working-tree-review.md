---
title: Runtime Asset Registry、Project Catalog、Index Persistence、Rebuild、Incremental Query、Watch 与 Generation 当前工作树复审
category: zircon_runtime
report_id: Runtime206
review_date: 2026-08-31
baseline_head: working-tree
related_code:
  - zircon_runtime/src/asset/registry
  - zircon_runtime/src/asset/project/catalog_input_generation.rs
  - zircon_runtime/src/asset/project/generation_observation.rs
  - zircon_runtime/src/asset/project/manager
  - zircon_runtime/src/asset/watch
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager
  - zircon_runtime/crates/zr_resource/src
plan_sources:
  - docs/plans/optimize/zircon_runtime/99w-runtime-asset-registry-index-persistence-rebuild-incremental-query-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/88-runtime-asset-watch-change-ingress-coalescing-rename-overflow-targeted-reimport-generation-reload-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/204-runtime-filesystem-resource-io-path-atomic-transaction-recovery-security-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/205-runtime-resource-lifecycle-load-ticket-cache-residency-generation-reload-cancellation-current-working-tree-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/AssetRegistryState.h
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/IAssetRegistry.h
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetRegistryImpl.h
  - dev/bevy/crates/bevy_asset/src/server/mod.rs
  - dev/bevy/crates/bevy_asset/src/server/loaders.rs
  - dev/bevy/crates/bevy_asset/src/io/source.rs
  - dev/bevy/crates/bevy_asset/src/handle.rs
  - dev/Fyrox/fyrox-resource/src/manager.rs
  - dev/Fyrox/fyrox-resource/src/loader.rs
  - dev/Fyrox/fyrox-resource/src/state.rs
  - dev/godot/editor/filesystem/editor_file_system.h
  - dev/godot/editor/filesystem/editor_file_system.cpp
  - dev/godot/core/io/resource_uid.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Material/AssetReimportUtils.cs
doc_type: current_source_review
review_status: review_complete
implementation_status: pending
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Runtime Asset Registry、Project Catalog、Index Persistence、Rebuild、Incremental Query、Watch 与 Generation 当前工作树复审

## 1. 结论

当前工作树已经有真实的资产索引骨架，但仍不能称为工程级 Asset Registry。`AssetRegistryIndex` 具备 UUID/path/AssetId/依赖/referencer/source 反向索引，project catalog 已有 64 个 shard、不可回绕的 sequence、predecessor delta，watch ingress 和 activation queue 具备 entries/bytes 上限，full/targeted generation 已经在 candidate 上准备文件、registry、resource 与 catalog。上述底座应保留。

本轮逐文件追踪后，核心结论有五点：

1. durability fence 仍在 publication 之后。`ProjectManager::scan_and_import`、watch import、targeted import 先把 candidate 的 `registry/resource/catalog` 写入 live `self`，`ProjectFileCommitOutcome::RecoveryDeferred` 只在之后通过 `ensure_durable()` 变成错误；这仍允许 `Err + 已发布 live state`，所以 Runtime99w 的 `ASSETREG-P0-002` 继续 Open。
2. duplicate GUID 仍由 `AssetUuid::new()` 随机 remint，并立即写回 `.zmeta`。没有全仓 serialized reference closure、codec/schema coverage、redirect/tombstone、operator receipt 或 unknown-codec fail-closed；`ASSETREG-P0-001` 继续 Open。
3. registry persistence 仍是 `format_version = 1` 的 whole-file pretty JSON，只包含 `entries`。没有 project/root/source inventory、importer/BuildSet、artifact manifest、generation、row/edge/byte/time admission、last-good/quarantine 或 versioned migration。
4. scan/incremental 仍不是可证明的 deterministic delta engine。`read_dir` 顺序未冻结，多 root/case/Unicode collision policy 缺失；增量 helper clone 全索引、重扫所有 `.zmeta`、重跑 duplicate normalization 和全量 dependency refresh；rename/multi-event 只能回退 full reconciliation。
5. query API 仍以 `Vec<&AssetRegistryEntry>` 为中心，type/tag/path/package 查询每次扫描并按 URI 排序，没有 compiled query、cursor/visitor、result budget、generation lease 或统一 `Missing/Empty/Ambiguous/Stale/Recovered` disposition。catalog sequence 不能替代 registry generation，也没有证明 registry/resource/catalog/watcher/event 同代。

因此本轮不新增独立 P0，继承 `ASSETREG-P0-001`、`ASSETREG-P0-002` 两项唯一 Runtime registry P0；watch 的 `WATCH88-P0-001`、`WATCH88-P0-002`、`WATCH88-P0-003` 仍由 Runtime88 唯一计数。本轮对 Runtime99w 的 60 个 P1 重新核对为 **53 Open / 7 Partial / 0 Closed**，16 个 P2 为 **12 Open / 4 Partial / 0 Closed**；Runtime88 的 watch P1 与 P0 不重复计数。所有 Partial 只表示局部代码存在，不能作为里程碑通过。

## 2. 审查边界与证据

### 2.1 当前生产清单

本轮冻结 `asset/registry`、project catalog/generation、project manager、asset watch、project asset manager 的 Rust 生产代码；排除 `tests`、`*_tests.rs`、`optimization_tests.rs` 和测试专用 runtime 文件。

| 维度 | 当前值 |
|---|---:|
| Rust 生产文件 | 95 |
| 总行数 | 16,444 |
| 非空行 | 14,965 |
| 字节数 | 596,718 |
| path + SHA-256 manifest 指纹 | `b6d00897c71cbeb3ca265f2ce4fdd38ce0efe3b506458fa4447ec71e2f5b7532` |

### 2.2 证据等级

- **E3**：读取当前生产实现，沿 open、full import、targeted import、watch、persistence、resource publication、Editor catalog input 调用链追踪。
- **E2**：读取相邻测试和本地 Unreal、Bevy、Fyrox、Godot、Unity Graphics 源码，核对 API/数据结构/失败边界是否真实存在。
- **E1**：本轮没有运行 Cargo、真实大项目 scan、fault/soak、跨平台 reparse、restart parity 或 benchmark；静态测试声明不等于动态通过。
- **E0**：没有 10K/100K/1M asset corpus 的 RSS、I/O、allocation、p50/p95/p99、watch-to-visible latency 与 correctness 对照，不能声称性能优于 Unreal。

## 3. 当前应保留的底座

### 3.1 Registry 的局部索引

`AssetRegistryIndex` 当前保存 `entries_by_uuid`、`uuids_by_path`、`uuid_by_asset_id`、`referencers_by_uuid`、`entry_uuids_by_source`、`dependency_paths_by_uuid` 与 `referencers_by_path`。`AssetRegistryEntry::with_dependencies` 使用 `HashSet` 去重；UUID reference resolve 不再静默按 path fallback。上述是正确的 identity/query 起点。

### 3.2 Candidate preparation 与 project generation

full generation 从 loaded metadata 建 index、解析依赖、准备 artifact/sidecar/registry writes，再构造 `ProjectCatalogInputGeneration`。targeted generation 只替换一个 source 的 entries 并刷新受影响 owner；catalog 使用 shard + `Arc::make_mut`，有 `sequence`、`predecessor_sequence` 和 added/modified/removed/renamed delta。这比把 Editor catalog 直接当作 mutable global map 更接近不可变 generation。

### 3.3 Watch ingress 与 activation queue

`watch_loop` 对 ingress/pending 同时限制 entry/byte，debounce 与 max latency 由 options 明确控制；overflow 会清掉 partial changes 并设 `requires_reconciliation`。activation 还有 bounded error queue、single-flight worker、Pending/Draining/Active/Retired 生命周期和 coalescing。此处的容量和生命周期模型应成为统一 watcher authority 的内部实现，而不是继续暴露第二套 registry mutation API。

### 3.4 Durable transaction 轮廓

`commit_prepared_files` 已经有 journal owner 校验、fault point、commit disposition、recovery policy 与 restart recovery；registry/meta/artifact writes 能被放进同一 prepared write list。问题不是缺少任何事务代码，而是 live publication 和 durable terminal 的排序尚未收束。

## 4. 当前产品纵切面

```text
ProjectManager::open
  -> manifest / derived layout / asset roots / journal recovery
  -> AssetRegistryIndex::load_or_rebuild
  -> empty ProjectCatalogInputGeneration

open_prepared_project / scan_and_import / watch worker / targeted import
  -> clone candidate ProjectManager
  -> full or targeted metadata/import/dependency projection
  -> prepare sidecar/artifact/registry writes
  -> prepare ResourceRegistry mutation
  -> commit prepared files
  -> install candidate project/resource/catalog and publish events
  -> ensure_durable()
```

最后两步仍是错误边界：`ensure_durable()` 可能失败，而前一步已经安装 candidate。open 路径还把 `AssetRegistryIndex::load_or_rebuild` 当作可直接信任的 authoritative state，没有 currentness header 或 source inventory validation。

## 5. 参考引擎对照

### 5.1 Unreal Asset Registry

`FAssetRegistryState` 将 asset data、package data、dependency graph、referencer graph、mount/package 边界和序列化 options 视为同一个 registry state；接口同时提供 `GetAssets`、`EnumerateAssets`、`EnumerateAllAssets`、package path/package name 查询、mutable update/remove，以及 dependency category 的 add/clear/set。`FARCompiledFilter` 与 visitor-style enumerate 让查询能在不复制全表的情况下提前终止。Zircon 当前 `AssetRegistryEntry` 只有 `AssetKind`、tags、裸 UUID dependencies、digest；`get_assets` 每次遍历并排序，无法表达 package state、dependency category、cook filter、mount generation 或 query budget。

### 5.2 Bevy AssetServer

Bevy 将 typed `AssetId`/`Handle`、source id、reader/writer capability、loader/meta settings、dependency load 状态和 `AssetEvent` 分开建模；`AssetServer` 的 load path 与 processed/source reader 不是一个裸文件打开函数。Zircon 目前将 source URI、`.zmeta`、artifact locator、runtime resource id 和 project catalog 通过局部结构拼接，缺统一 `SourceInstanceId + AssetTypeId + SchemaId + ArtifactKey + LoadDisposition`，也没有从 discovery 到 loaded/cooked 的状态图。

### 5.3 Fyrox resource manager

Fyrox 的 resource manager/loader/state 组合把 loader identity、loading/error state、resource lifetime 和 async completion 放在 resource authority 内；Zircon 的 `ProjectCatalogInputGeneration` 只负责 Editor 输入投影，不能代表 registry 的 load state、artifact validity 或 resource residency。Runtime205 已记录通用资源生命周期缺少统一 authority，本报告不重复其 P0，而是要求 registry generation 作为该 authority 的输入版本。

### 5.4 Godot FileSystemDock、EditorFileSystem 与 ResourceUID

Godot 的 editor filesystem 扫描、import state、resource UID、dependency/import metadata 和 filesystem change notifications 是同一 editor database 的不同视图；UID 变更需要保持资源引用语义，importer 能报告 changed/failed/reimport 状态。Zircon 的 duplicate remint 仍只改 `.zmeta`，不遍历场景、材质、脚本和插件 serialized references；orphan `.zmeta` 也会被静默跳过。

### 5.5 Unity Graphics reimport boundary

Unity Graphics 的 `AssetReimportUtils` 只是在 Unity asset/import pipeline 上提供 reimport 辅助，不是完整 registry；它的可借鉴边界是 importer/version/settings 变化必须成为明确的 reimport key，并可回溯到 editor import state。Zircon 当前 `source_digest` 是唯一核心 row freshness 字段，缺 importer schema/version、settings hash、artifact/cook target 和 source capability。

## 6. P0 正确性阻断（继承，唯一计数）

### ASSETREG-P0-001：duplicate GUID remint 没有 reference closure

证据：`zircon_runtime/src/asset/registry/rebuild.rs:122-190` 和 `zircon_runtime/src/asset/registry/targeted.rs:119-189` 在遇到重复 UUID 时调用 `AssetUuid::new()`，修改 root/subasset UUID 并直接保存 `.zmeta`。当前 owners 只来自 registry/path，不是全仓 inbound reference graph；`AssetRegistryDiagnostic::DuplicateGuidReminted` 只有 original/first/path/replacement，没有 operation id、generation、codec、receipt 或 redirect。

后果：任意未知或未扫描的 scene/material/script/plugin 文档仍可引用旧 UUID；重启后 registry 看似唯一，但语义引用已经指向不存在对象或错误对象。随机 UUID 还使 scan 顺序、机器和重试结果不可重现。

必须重构为：只读 collision candidate -> deterministic owner policy/operator approval -> registered codec closure scan -> unknown codec fail closed -> sidecar/serialized inbound/redirect/tombstone/registry 同一 transaction -> durable migration receipt -> restart verification。未有 closure proof 时不得写任何文件。

### ASSETREG-P0-002：RecoveryDeferred 在 live publication 之后才变成错误

证据：`zircon_runtime/src/asset/project/manager/scan_and_import.rs:95-116` 在 `prepared.commit()` 后执行 `*self = candidate`，随后才 `outcome.ensure_durable()`；`full_generation.rs:396-399` 和 `targeted.rs:529-544` 也在 prepare 阶段修改 candidate 的 registry/resource/catalog；`project_asset_manager/open_project.rs:45-80` 在 resource sync callback 中安装 project/watchers、发布 generation，最后才 `ensure_durable()`。因此 marker fsync unresolved 时，调用者得到 Err，但 live project、resource、watcher 或 catalog 可能已经换代。

必须重构为显式 `Durable` 或 `AcceptedRecoveryPending(OperationId)` terminal disposition。普通 API 不得返回 `Err` 同时留下已发布 candidate；live publication fence 必须只在 durable terminal 或可观察的 recovery-pending state 后打开，event/journal/restart recovery 共用 OperationId 和 generation。

## 7. P1 差距与当前状态

### 7.1 Identity、row、dependency graph

| ID | 当前状态 | 当前证据与重构要求 |
|---|---|---|
| ASSETREG-P1-001 | Open | `AssetRegistryEntry::type_marker` 仍为宽泛 `AssetKind`；加入稳定 `AssetTypeId` 与 provider registration。 |
| ASSETREG-P1-002 | Open | row 没有 `SchemaId`、codec/importer schema version 或 migration chain。 |
| ASSETREG-P1-003 | Open | 没有 artifact manifest hash、cook key、target/profile、toolchain identity。 |
| ASSETREG-P1-004 | Open | source URI 没有 source instance、mount/provider、physical identity、trust provenance。 |
| ASSETREG-P1-005 | Partial | catalog 有 process-local sequence，但 registry row/header 没有 project identity、BuildSet、registry generation。 |
| ASSETREG-P1-006 | Open | dependency 仍是裸 UUID；必须含 category、required/optional、expected type/schema/revision/provenance。 |
| ASSETREG-P1-007 | Open | dependencies 去重但没有 canonical binary sort、edge identity 与 duplicate diagnostic。 |
| ASSETREG-P1-008 | Open | rename/remint/delete 没有 redirect、tombstone、retirement epoch/expiry。 |
| ASSETREG-P1-009 | Open | row 不表达 discovered/importing/ready/stale/failed/missing-artifact。 |
| ASSETREG-P1-010 | Open | diagnostic 缺稳定 code/id、generation、source span、owner、operation、remediation。 |

### 7.2 Persistence、currentness、recovery

| ID | 当前状态 | 当前证据与重构要求 |
|---|---|---|
| ASSETREG-P1-011 | Open | `load_or_rebuild` 成功读取 JSON 就接受，不核对 sidecar/source/artifact currentness。 |
| ASSETREG-P1-012 | Open | `PersistedAssetRegistry` header 只有 `format_version`，不绑定 project/root/config/importer/BuildSet/catalog generation。 |
| ASSETREG-P1-013 | Open | `fs::read` + `serde_json::from_slice` 没有 bytes/rows/edges/time/cancel admission。 |
| ASSETREG-P1-014 | Open | tags/dependency/path/diagnostic/string 没有 per-row/total budget。 |
| ASSETREG-P1-015 | Open | I/O、permission、decode、version、duplicate 共用 rebuild disposition，权限错误可能触发改写。 |
| ASSETREG-P1-016 | Open | corrupt/unsupported 文件没有 quarantine、backup、hash、raw evidence、operator action。 |
| ASSETREG-P1-017 | Open | version 不是 migration window；unsupported version 直接 rebuild。 |
| ASSETREG-P1-018 | Open | `CorruptPersistenceRebuilt` 只是内存 diagnostic，没有 durable recovery receipt。 |
| ASSETREG-P1-019 | Partial | writer 改为借用 entry，减少 deep clone；仍是全量 pretty JSON、全量 sort/encode，无 RSS/写放大预算。 |
| ASSETREG-P1-020 | Open | 没有 read-only/degraded/last-good mode，不能保证 stale/corrupt 期间不产生副作用。 |

### 7.3 Scan、rebuild、root policy

| ID | 当前状态 | 当前证据与重构要求 |
|---|---|---|
| ASSETREG-P1-021 | Open | `collect_meta_paths` 直接消费 `fs::read_dir`，未排序；diagnostic/owner 依赖 filesystem enumeration。 |
| ASSETREG-P1-022 | Open | `asset_roots` 可产生同 URI 多 root collision，没有 priority/schema/collision report。 |
| ASSETREG-P1-023 | Open | URI/path 没有 case-fold、Unicode normalization、canonical collision policy。 |
| ASSETREG-P1-024 | Open | include/exclude/hidden/vendor/generated/source-control-ignore 未进入 SourceInventory identity。 |
| ASSETREG-P1-025 | Partial | link/reparse 被 fail closed；但仍是单条错误中止全 scan，没有 per-entry typed disposition。 |
| ASSETREG-P1-026 | Open | `.zmeta` 对应 source 不存在时 `scan_meta_paths` 静默 `continue`，没有 orphan tombstone/diagnostic。 |
| ASSETREG-P1-027 | Open | rebuild 信任 sidecar `source_digest`，不验证 source/artifact 内容对应关系。 |
| ASSETREG-P1-028 | Open | scan 没有 entries/depth/bytes/time/deadline/cancel/progress receipt。 |
| ASSETREG-P1-029 | Open | metadata load/decode 串行；没有 bounded I/O/CPU pipeline 与 deterministic merge。 |
| ASSETREG-P1-030 | Open | public rebuild/normalization 会在 registry persist 前直接写 `.zmeta`，不在唯一 generation transaction 内。 |

### 7.4 Incremental、watch、closure

| ID | 当前状态 | 当前证据与重构要求 |
|---|---|---|
| ASSETREG-P1-031 | Open | `AssetRegistryIndex::apply_watch_changes` 仍是可调用的第二 mutation authority，无 product caller qualification。 |
| ASSETREG-P1-032 | Open | incremental path `let mut candidate = self.clone()`，小变更复制整张 index。 |
| ASSETREG-P1-033 | Open | 每批 changes 调 `scan_project_metas`，重扫全部 meta 并 duplicate-normalize。 |
| ASSETREG-P1-034 | Open | empty change 直接 return，不验证 watcher overflow、root generation、source drift。 |
| ASSETREG-P1-035 | Open | targeted source replacement clone HashMap/HashSet 和相关 entry，缺 persistent immutable segment/COW budget。 |
| ASSETREG-P1-036 | Partial | 显式 single-source relocation 能保留 UUID并 transactional；watch rename/previous URI/multi-event/compound source 仍 full fallback。 |
| ASSETREG-P1-037 | Open | affected set 只刷新直接 owner，缺 transitive dependency/readiness/diagnostic closure oracle。 |
| ASSETREG-P1-038 | Open | dependency path retarget 未 canonical sort/dedup 并报告 typed conflict。 |
| ASSETREG-P1-039 | Partial | owner refresh 使用 `HashSet`；仍缺 managed validation、release corpus 和 graph fan-out benchmark。 |
| ASSETREG-P1-040 | Open | source/affected owner/diagnostic 发布顺序未形成跨运行 canonical contract。 |

### 7.5 Query、index、snapshot

| ID | 当前状态 | 当前证据与重构要求 |
|---|---|---|
| ASSETREG-P1-041 | Open | type/tag/path/package 没有二级 posting/path/package index；filter 每次扫 entries。 |
| ASSETREG-P1-042 | Open | `entries()` 每次分配 Vec 并按 path 排序；catalog shard 不能替代 registry visitor。 |
| ASSETREG-P1-043 | Open | query 没有 cursor、pagination、result budget、deadline、early stop。 |
| ASSETREG-P1-044 | Open | filter 没有 compiled predicate/query plan/invalid-filter error。 |
| ASSETREG-P1-045 | Open | missing asset 与 present-but-zero-edge 都返回空 `Vec`。 |
| ASSETREG-P1-046 | Partial | UUID resolve 已区分 dangling reference 和 path hint；dependency/referencer/filter 尚未统一 disposition。 |
| ASSETREG-P1-047 | Open | referencer 仍按 UUID `ToString` 排序，缺 binary key contract。 |
| ASSETREG-P1-048 | Open | borrowed index/snapshot 没有 generation lease，旧 generation retire 无读者证明。 |
| ASSETREG-P1-049 | Open | inspection/query/resolve/rebuild errors 没有统一 availability model。 |
| ASSETREG-P1-050 | Partial | targeted dependency/referencer caller 已存在；大多数 query 仍 test-only，未形成产品 API 资格。 |

### 7.6 Product、Editor projection、extractor、qualification

| ID | 当前状态 | 当前证据与重构要求 |
|---|---|---|
| ASSETREG-P1-051 | Open | Runtime UI/consumer 仍可使用全量 `entries()`，没有 batch/async/progress contract。 |
| ASSETREG-P1-052 | Open | `ProjectManager::open` 读取合法但 stale registry，不在 full activation 前验证 currentness。 |
| ASSETREG-P1-053 | Partial | catalog 有 sequence/predecessor delta；registry/resource/watcher/event 没有共同 generation identity。 |
| ASSETREG-P1-054 | Partial | Editor 消费 catalog generation/delta；Editor 仍维护可复制的 import/index projection。 |
| ASSETREG-P1-055 | Open | `ProjectInfo` 仍只有有限 diagnostic count，缺 typed severity/code/generation/remediation。 |
| ASSETREG-P1-056 | Open | handwritten dependency extractor 仍集中于 Scene/Material/Model。 |
| ASSETREG-P1-057 | Open | unsupported ImportedAsset variant 可能静默空依赖，无法区分 Complete/Unsupported/Error。 |
| ASSETREG-P1-058 | Open | extractor 无 schema/reflection/provider registration/coverage manifest/unknown-field policy。 |
| ASSETREG-P1-059 | Open | symlink/reparse 权限不足的测试不能证明目标平台 policy；需要 machine-readable skip 或 required-platform fail。 |
| ASSETREG-P1-060 | Open | 缺 recovery-deferred-after-publication、multi-sidecar remint fault、reference closure、valid-stale persistence、restart parity tests。 |

## 8. P2 长期能力

当前 P2 为 **12 Open / 4 Partial**。Partial 仅指已有局部方向：catalog shard/COW（P2-009）、借用序列化减少 clone（P2-011）、generation observation counters（P2-012）和 watch bounded queue（P2-015 对应 watch 侧能力）。仍未完成的工程能力包括：

1. versioned bounded binary/columnar persistence、section checksum、mmap/segmented snapshot；
2. string/tag/path interning、bitmap/posting index、zero-allocation visitor 和 compiled query cache；
3. WAL/delta segment + checkpoint、RCU candidate builder、generation lease retirement；
4. bounded parallel scan/decode/extract、SCC/topological dependency cache、large-corpus benchmark 与 fuzz/property model；
5. collision inspector、quarantine explorer、minimal reproducer/evidence redaction；
6. watch-to-visible telemetry、retry/backoff/fairness、cross-process/VCS/network provider qualification。

这些 P2 必须等 P0 和 schema/currentness P1 完成后实施，不能以 micro-benchmark 代替数据模型和失败语义。

## 9. 目标架构与 hard cut

目标不是继续给 `AssetRegistryIndex` 加字段，而是收束为下列 owner：

```text
SourceInventoryAuthority
  -> SourceInstance / Mount / Provider / Trust / IgnorePolicy / RootGeneration

AssetIdentityAuthority
  -> AssetTypeId / SchemaId / AssetUuid / SubassetId / Redirect / Tombstone

AssetRegistryGeneration
  -> immutable rows + typed edges + artifact/build state + diagnostics + source inventory hash

RegistryPersistenceService
  -> bounded reader/writer + header validation + migration/quarantine/last-good

RegistryMutationCoordinator
  -> full scan / targeted delta / watch fold / import / reimport / relocation
  -> one prepared generation transaction and one Durable/RecoveryPending outcome

RegistryQueryService
  -> compiled filter + indexed candidate sets + cursor/visitor + generation lease + disposition

ProjectCatalogProjection
  -> Runtime-owned projection consumed by Editor; no second import truth
```

必须 hard cut：

- 删除或收紧 public `AssetRegistryIndex::apply_watch_changes`，所有外部变化进入 `RegistryMutationCoordinator`；
- 禁止 scan/inspection/normalization 在没有 operation receipt 的情况下写 `.zmeta`；
- `AssetUuid::new()` 不得作为 duplicate repair 的默认 policy；
- `AssetRegistryIndex` 不再把 `AssetKind` 作为长期 schema identity；
- `entries()` 全表分配 API 仅允许 diagnostic/admin scope，产品路径使用 leased visitor/query page；
- `ProjectCatalogInputGeneration` 必须携带 `RegistryGenerationId`、`ResourceGenerationId` 和 publication disposition，而不是独立 process-local counter。

## 10. 依赖有序重构里程碑

| 里程碑 | 范围 | 完成判定 |
|---|---|---|
| M206-0 Freeze/RED | 固定 source inventory、P0 repro、owner/consumer matrix | duplicate closure 与 RecoveryDeferred publication 测试在旧实现稳定失败。 |
| M206-1 Identity/schema v2 | exact type/schema/source/artifact/state/edge/diagnostic model | row/edge/header 可 round-trip；unknown codec/type fail closed。 |
| M206-2 Collision and migration | read-only collision report、codec closure、redirect/tombstone、receipt | remint 不再随机或静默写；所有 inbound reference closure 有证据。 |
| M206-3 Persistence/currentness | bounded binary sections、header identity、migration window、quarantine、last-good | stale/corrupt/permission/version 各有 typed disposition，权限错误不自动改写。 |
| M206-4 Mutation coordinator | full/targeted/watch/import/relocation 统一 prepare/commit/publication fence | live publication 只发生在 Durable 或可观察 RecoveryPending；restart parity 通过。 |
| M206-5 Incremental closure | event fold、source-level delta、transitive dependency/readiness/diagnostic closure | normalized delta 与 full rebuild state hash 相同；rename/overflow 有界恢复。 |
| M206-6 Query service | secondary indexes、compiled filter、cursor/visitor、lease、result/deadline budget | million-row corpus 无全表复制，missing/empty/stale/recovered 可区分。 |
| M206-7 Product convergence | Runtime UI、Editor catalog、cook/package、resource manager 消费同一 generation | 无第二 import truth，所有 event/receipt/catalog/resource generation 可关联。 |
| M206-8 Qualification | fault/restart/cross-platform/soak/benchmark/fuzz | correctness gates 全绿后才允许与参考引擎比较性能。 |

## 11. 验收门禁

| Gate | 当前 | 必须证明 |
|---|---|---|
| ASSETREG206-G01 | Fail | duplicate scan 只产生 collision candidate，不写 sidecar。 |
| ASSETREG206-G02 | Fail | registered codec closure 覆盖所有 inbound UUID reference，semantic target 不变。 |
| ASSETREG206-G03 | Fail | unknown/unavailable codec 使 migration fail closed，live/disk 零变化。 |
| ASSETREG206-G04 | Fail | redirect/tombstone 含 generation、owner、reason、expiry、receipt。 |
| ASSETREG206-G05 | Fail | owner policy 与 scan enumeration/thread/platform 无关。 |
| ASSETREG206-G06 | Fail | RecoveryDeferred 不产生 Err+已发布 live state。 |
| ASSETREG206-G07 | Fail | API/event/journal/restart recovery 共享 OperationId 与 generation。 |
| ASSETREG206-G08 | Fail | registry/resource/catalog/watcher/event 只发布同一 generation。 |
| ASSETREG206-G09 | Fail | v2 header 绑定 project/root/config/importer/BuildSet/source inventory。 |
| ASSETREG206-G10 | Fail | valid-but-stale sidecar/source/artifact 在 publish 前识别。 |
| ASSETREG206-G11 | Fail | corrupt/permission/unsupported version 有 quarantine/last-good/typed action。 |
| ASSETREG206-G12 | Fail | reader/writer 具 bytes/rows/edges/tags/strings/depth/time/cancel budget。 |
| ASSETREG206-G13 | Fail | scan/row/edge/diagnostic 顺序跨运行确定。 |
| ASSETREG206-G14 | Fail | multi-root/case-fold/Unicode/canonical collision fail closed。 |
| ASSETREG206-G15 | Fail | orphan/link/meta 产生 typed disposition，不静默跳过。 |
| ASSETREG206-G16 | Fail | full/targeted/watch normalized state hash 一致，overflow/rename 可恢复。 |
| ASSETREG206-G17 | Fail | transitive dependency/readiness/diagnostic closure 有 oracle。 |
| ASSETREG206-G18 | Fail | query 使用 index/compiled filter/cursor/visitor，并有 result/deadline budget。 |
| ASSETREG206-G19 | Fail | generation lease 保证 snapshot 退休安全，旧 reader 不观察混代。 |
| ASSETREG206-G20 | Partial | UUID resolve 已 fail closed；所有 query disposition 尚未统一。 |
| ASSETREG206-G21 | Partial | catalog 有 sequence/delta；registry/resource/watcher/event 尚无共同 generation。 |
| ASSETREG206-G22 | Fail | Runtime UI/Editor/cook 不同步逐项加载或维护第二 import truth。 |
| ASSETREG206-G23 | Fail | ProjectInfo/diagnostic 暴露 typed severity/code/generation/remediation。 |
| ASSETREG206-G24 | Fail | fault/restart/cross-platform/large corpus/fuzz evidence 全部存在。 |

## 12. 首个实施切片

当前只允许进入实现计划，不直接在本轮改生产代码：

1. 为 `ASSETREG-P0-002` 添加最小 RED：在 commit marker durability unresolved 后断言 project/resource/catalog/watcher/event 都没有发布 candidate，或返回显式 RecoveryPending receipt；
2. 为 `ASSETREG-P0-001` 添加只读 duplicate collision fixture，证明旧 UUID inbound references、unknown codec 和 deterministic owner policy；
3. 定义 `RegistryGenerationId`、`OperationId`、`SourceInventoryHash` 和 `PublicationDisposition` 的 interface 级合同；
4. 将 full/targeted/watch 的 prepared state 改为不可见 candidate，只有 coordinator 在 durable terminal 后一次性提交；
5. 先做排序、root collision、orphan meta、stale registry 的 currentness tests，再开始 binary persistence/query index 优化。

## 13. 当前限制

本篇是 review-only。没有运行 Cargo、真实项目导入、跨进程 recovery、GPU/Editor product loop、Windows reparse required-platform、large corpus benchmark 或参考引擎动态基准；任何“性能和表现优于 Unreal”的结论都必须留到正确性门禁与同内容 workload 证据之后。
