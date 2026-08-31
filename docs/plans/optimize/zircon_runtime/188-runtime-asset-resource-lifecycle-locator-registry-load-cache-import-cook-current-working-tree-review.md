---
title: Runtime Asset/Resource 生命周期、Locator、Registry、Load、Residency、Cache、Import、Cook 与 Package 当前工作树工程化差距
category: zircon_runtime
report_id: Runtime188
review_date: 2026-08-30
baseline_head: working-tree
related_code:
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/facade
  - zircon_runtime/src/asset/pipeline/manager
  - zircon_runtime/src/asset/project
  - zircon_runtime/src/asset/registry
  - zircon_runtime/src/asset/importer
  - zircon_runtime/src/asset/artifact
  - zircon_runtime/src/asset/watch
  - zircon_runtime/src/asset/pack
  - zircon_runtime/crates/zr_resource/src
  - zircon_runtime_interface/src/resource
tests:
  - zircon_runtime/src/asset/tests
  - zircon_runtime/crates/zr_resource/src/tests.rs
  - zircon_runtime/crates/zr_resource/src/manager/tests
plan_sources:
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/87-runtime-asset-reference-identity-locator-guid-subasset-redirector-rename-move-resolution-repair-migration-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/88-runtime-asset-watch-change-ingress-coalescing-rename-overflow-targeted-reimport-generation-reload-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99m-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99w-runtime-asset-registry-index-persistence-rebuild-incremental-query-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/187-runtime-scene-ecs-world-archetype-query-schedule-generation-current-working-tree-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Developer/DerivedDataCache/Public/DerivedDataBuildDefinition.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/SoftObjectPath.h
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/AssetRegistryState.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/PrimaryAssetId.h
  - dev/bevy/crates/bevy_asset/src/server/mod.rs
  - dev/bevy/crates/bevy_asset/src/handle.rs
  - dev/bevy/crates/bevy_asset/src/loader.rs
  - dev/bevy/crates/bevy_asset/src/io/mod.rs
  - dev/bevy/crates/bevy_asset/src/meta.rs
  - dev/Fyrox/fyrox-resource/src/manager.rs
  - dev/Fyrox/fyrox-resource/src/loader.rs
  - dev/Fyrox/fyrox-resource/src/state.rs
  - dev/godot/core/io/resource_loader.cpp
  - dev/godot/core/io/resource.cpp
  - dev/godot/core/io/resource_uid.cpp
  - dev/godot/editor/file_system/editor_file_system.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Material/AssetReimportUtils.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Importers/ShaderGraphImporter.cs
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Runtime Asset/Resource 生命周期、Locator、Registry、Load、Residency、Cache、Import、Cook 与 Package 当前工作树工程化差距

## 1. 结论

当前 Runtime 已经有真实的资产底座：`ResourceLocator` 做了 scheme/path/label 规范化；`ResourceRegistry` 与 `AssetRegistryIndex` 能分别承担运行时记录和项目元数据查询；`ProjectManager` 能生成候选项目代，进行 full/targeted import，提交原子 journal；artifact store 有 bincode、zstd、BLAKE3、64 KiB chunk、manifest 上限和 chunk residency；watcher 有 debounce、最大批次延迟和 entry/byte 上限；importer registry 有 COW generation、full suffix/extension matcher、priority 和 plugin unload。

这些机制说明 Zircon 已超过“扫描文件、按扩展名读入一个 HashMap”的样例实现。但它们仍是多个平行 owner 的拼接，不是由一个 versioned `AssetBuildGraph` 驱动的 source-to-runtime 生命周期。`ProjectAssetManager` 同时持有项目代、source path index、watch activation、watch diagnostics、residency stripes、importer registry 和 `ResourceManager`（`zircon_runtime/src/asset/pipeline/manager/project_asset_manager/project_asset_manager.rs:97-124`）；加载路径又把同步 artifact I/O、typed decode、runtime payload publication 和 lease eviction 组合在调用线程中（`.../loading/ensure_resident.rs:11-102`）。这在小项目可工作，在大场景、跨平台 cook、热重载和 editor/runtime 并行访问时无法提供可证明的版本、优先级、预算、取消和恢复语义。

本轮没有新增独立 P0；Runtime85 的 external auxiliary source stale artifact、Runtime99m 的资源 authority/version/reload/cancel、Runtime108/109/110 的 runtime interface/serialization P0 仍然有效。本轮新增的是 36 项 P1、14 项 P2 和 30 个资格门，用来把已有零散能力收敛为一条可回放、可观测、可取消的资源生命周期。`tooling` 按用户要求不在本轮范围内。

## 2. 审查方法与证据边界

### 2.1 逐文件范围

本轮沿四条 owner 链读取生产代码和相邻测试：

| 链 | 关键当前文件 | 观察重点 |
|---|---|---|
| 身份与索引 | `zircon_runtime_interface/src/resource/{locator,resource_id,resource_record}.rs`、`zircon_runtime/src/asset/{reference_resolver.rs,registry}` | stable identity、label/subasset、scheme、generation、dependency/referencer |
| 管理与驻留 | `zircon_runtime/crates/zr_resource/src/{registry.rs,manager,runtime.rs,lease.rs}`、`asset/facade` | registry authority、payload、revision、lease、state、event |
| 导入与派生 | `asset/importer`、`asset/project/manager`、`asset/artifact` | source snapshot、recipe、dependency discovery、artifact key、atomic publication |
| 产品接线 | `asset/pipeline/manager/project_asset_manager`、`asset/watch`、`asset/pack` | sync/async、watch gap、reload、cook/package/install、project teardown |

### 2.2 证据等级

- **E3**：读取当前工作树生产实现并沿调用链追踪，不把类型声明当成功能完成。
- **E2**：读取 Runtime asset/resource 测试与参考引擎对应源码，核对 owner、字段和终态。
- **E1**：测试只证明测试意图；本轮没有运行 Cargo、真实项目导入、cook、跨平台 pack、故障注入或性能基准。
- **E0**：不能据此宣称“性能优于 Unreal”或宣称热重载、取消、恢复已经动态通过。

## 3. 当前可保留底座

1. `ResourceLocator` 以 scheme/path/label 表达 project、library、package、builtin、memory，并拒绝 path escape；这是继续加入 mount/provider identity 的正确入口。
2. `AssetImporterRegistry` 用 COW generation 和 matcher index 发布 importer，且能按 plugin 删除；不要退回每次扫描重建一张无版本表。
3. `AssetMetaDocument` 已有 format version、included files、artifact locator、importer id/version、config/source digest、dependency 和 subasset entry；问题是这些字段尚未成为一个不可变 build input contract。
4. `AssetRegistryIndex` 已维护 UUID/path/id、dependency/referencer、source-entry 反向索引；可作为查询 projection，但不能同时冒充 build graph authority。
5. `ArtifactStore` 的 staging、atomic manifest、chunk 校验、raw/manifest 上限和 LRU residency 是大型资源读取的必要内核；应补齐 provenance 和 target qualification，而不是重写为裸文件读取。
6. watcher 的 bounded ingress/pending batch、targeted/full 分流和 transaction watch echo 方向正确；需要把 source generation、event sequence 和 cancellation 传入 publication receipt。

## 4. 既有 P0 归属，不在本轮重复计数

| 父 owner | 仍开放的边界 | 本轮处理 |
|---|---|---|
| Runtime85 | auxiliary file 未进入 source identity、recipe/DDC/cook graph、完整 external dependency | 作为 source-build P0 引用；本轮只补生命周期和 consumer 影响 |
| Runtime99m | resource authority、typed handle、version lease、reload、cancellation | 本轮把观察延伸到 ProjectAssetManager 和 `zr_resource` 的真实接线 |
| Runtime108/109/110 | runtime interface、serialization、动态资源 API | 作为跨模块 P0 引用，不把同一 ABI 问题复制成资产 P0 |
| Runtime187 | world/ECS snapshot、generation、command/event retirement | 资源 publication 必须以 world/project generation fence 接入，不能另建一套隐式 epoch |
| Editor226/247 | catalog projection、asset workspace、play/document 与 runtime generation | Editor248 负责产品消费面，本报告只拥有 Runtime authority |

## 5. P1 工程化差距（36 项）

### 5.1 身份、locator 与 registry

| ID | 当前证据与差距 | 必须重构为 |
|---|---|---|
| AST4-P1-001 | `AssetId` 直接别名 `ResourceId`；`ResourceHandle` 只存 id，旧 handle 没有 generation/revision 检查。 | `AssetHandle { identity, generation, expected_type }`，旧代访问返回 typed stale 结果而非读到新 payload。 |
| AST4-P1-002 | `ResourceLocator` 的 label 是任意字符串，subasset identity 仍由 importer label 和 index 约定。 | source UID、subasset UID、display label 分离；label 变更产生 redirect/remap record。 |
| AST4-P1-003 | `ResourceRegistry`（`Arc<HashMap>`）与 `AssetRegistryIndex`（多张 HashMap）各自保存 id/path/dependency 事实。 | 单一 immutable `AssetCatalogGeneration`，runtime/editor/package 都消费带 generation 的 projection。 |
| AST4-P1-004 | `AssetRegistryIndex::entries()` 每次复制并按 path 排序；大 catalog 查询退化为全量分配。 | 持久排序页、kind/tag/path trie 或 keyed query cursor，禁止 UI/pack 热路径重建 Vec。 |
| AST4-P1-005 | `AssetRegistryEntry` 只有 String `source_digest`，没有算法、size、mtime confidence、included-file digest 和 source snapshot id。 | `SourceFingerprint { algorithm, bytes, digest, files, observed_at }`，字段可验证且可序列化。 |
| AST4-P1-006 | `ResourceKind` 是固定枚举；未知 plugin/importer 类型不能保留 opaque type。 | namespaced `AssetTypeId + schema/version + opaque payload`；`ResourceKind` 仅作内置能力分组。 |
| AST4-P1-007 | scheme 能表达 package id，但没有 mount generation、provider capability、trust/read-only/offline health。 | `AssetMountId`、mount generation、reader/writer/watch/cook capability 和 health 进入 locator resolution。 |
| AST4-P1-008 | `ResourceRegistry::insert_unchecked` 依赖 debug assertion，生产冲突必须由上层 staging 预防。 | commit 层强制验证 id/kind/locator uniqueness，并返回可恢复的 conflict receipt。 |

### 5.2 Load、payload 与 residency

| ID | 当前证据与差距 | 必须重构为 |
|---|---|---|
| AST4-P1-009 | `ensure_resident` 在调用线程读取 artifact、解码 zstd/bincode 并 `store_runtime_payload`；没有 future、I/O queue 或 backpressure。 | `LoadRequest`/`LoadTicket`/`LoadFuture`，I/O、decode、GPU/physics upload 分阶段调度并可暂停。 |
| AST4-P1-010 | `load_imported_asset` 对每个 `AssetKind` 手写 match；Texture/UI icon、UiLayout/V2、UiStyle/Theme 通过 `or_else` 试解析，错误语义不稳定。 | importer/asset type 注册表提供 typed loader、schema probe 和唯一 winner；失败只返回明确类型错误。 |
| AST4-P1-011 | facade `load<T>` 最终 clone typed payload；大 mesh/texture/model 无 zero-copy snapshot 或 streaming view。 | immutable payload snapshot、range/mip/LOD view、显式 clone policy；CPU/GPU 资源分离。 |
| AST4-P1-012 | `ResourceLease` 只保存 id/token/Arc payload，Drop 时降低 refcount；没有 owner、priority、deadline、purpose。 | lease metadata 记录 consumer、priority、budget class、deadline 和 revision pin。 |
| AST4-P1-013 | refcount 归零立即从 `payloads` 移除（`lease_ops.rs:49-69`）；没有全局 byte budget、warm retention、streaming priority 或 eviction reason。 | budgeted residency manager，按 bytes/priority/age/scene demand 驱动 trim，保留 last-good policy。 |
| AST4-P1-014 | residency token 只是递增 u64，未与 project generation、record revision、load request 绑定。 | token 由 `(project_generation, asset_revision, request_id)` 构成，旧 completion 只能得到 stale。 |
| AST4-P1-015 | `RuntimeResourceState` 只有 Unloaded/Loading/Loaded/Error/Reloading；没有 queued/blocked/evicting/uploading/dependency-wait。 | 可序列化状态机，明确 transition owner、retry budget、last-good payload 和 terminal diagnostic。 |
| AST4-P1-016 | `ResourceManager` 使用全局 `RwLock<ResourceAuthority>` 与 `Mutex` commit serial；读 generation、payload、lease 和事件共享大锁。 | sharded authority 或 immutable publication + lock-free read snapshot；commit 只锁受影响 shard。 |
| AST4-P1-017 | `Arc<HashMap>` COW 使每次 registry mutation 可能复制整张 map；`Arc::make_mut` 不能替代分段/增量索引。 | persistent B-tree/segment index 或 sharded copy-on-write page，记录 changed pages。 |
| AST4-P1-018 | poisoned lock 统一 `into_inner` 继续运行，可能把 panic 后半更新状态当成合法 authority。 | commit journal/poison state/repair path；只读查询可降级，写入必须拒绝未修复 authority。 |

### 5.3 Dependency、readiness 与 publication

| ID | 当前证据与差距 | 必须重构为 |
|---|---|---|
| AST4-P1-019 | `ResourceRecord.dependency_ids` 只有 Vec<ResourceId>，不区分 source、artifact、runtime、optional、hard、editor-only。 | typed edge `{kind,strength,phase,target,owner}`，构建、加载、cook 分层。 |
| AST4-P1-020 | registry 依赖路径索引是 `Vec<AssetUri>`，每次 source replacement 才重建；没有 edge provenance。 | immutable dependency manifest 和 reverse edge pages，单边修改可定向失效。 |
| AST4-P1-021 | resolve dependency 未找到时把诊断附加到 record，但没有 unresolved node/tombstone 和 retry trigger。 | 缺失依赖是 graph node 状态，可在 mount/import generation 改变时自动重试。 |
| AST4-P1-022 | cycle 主要由 import/manifest 诊断发现，runtime readiness 没有统一 SCC/partial-load policy。 | graph build 阶段计算 SCC；循环边带 policy，禁止 load 侧递归栈溢出或静默 NotLoaded。 |
| AST4-P1-023 | readiness generation 与 management generation 分离；asset facade 只能把三种 load state 组合，缺 publication receipt。 | 一个 `ResourcePublication` 同时携带 registry/revision/payload/dependency/readiness sequence。 |
| AST4-P1-024 | event stream 有 gap diagnostics，但订阅者没有 ack/currentness cursor 和 resync authority。 | per-consumer cursor、gap->snapshot handshake、project teardown retirement 和 bounded replay。 |
| AST4-P1-025 | record revision 在 upsert 时递增，但没有 source/build/runtime/GPU revision 分层。 | `SourceRevision`, `BuildRevision`, `PayloadRevision`, `DeviceRevision` 分层比较，避免无关 reload。 |
| AST4-P1-026 | `ProjectManager` clone 包含 registry、artifact、importer、diagnostics 等整套状态；候选代边界靠调用者纪律。 | immutable build session 输入 + explicit prepared output，候选不能共享可变 watcher/lease owner。 |

### 5.4 Import、artifact、cook、watch 与 package

| ID | 当前证据与差距 | 必须重构为 |
|---|---|---|
| AST4-P1-027 | importer selection 以 extension/full suffix 为主；没有 magic/MIME/schema probe 和 conflict evidence。 | sniff -> candidate -> deterministic winner，winner receipt 可重放。 |
| AST4-P1-028 | `AssetImporter` builtin 注册和 `load_imported_asset` 分派是手写函数表；新增格式要修改中心 match。 | `AssetTypeDescriptor` + loader/processor/cook vtable，插件只注册能力和 schema。 |
| AST4-P1-029 | importer 接收完整 `Vec<u8>` 或直接从 source path 打开外部文件；source snapshot 不封闭。 | immutable `AssetSourceSnapshot` reader，所有 included files、random access、read receipt 都受 build session 控制；Runtime85 P0 关闭前不得宣称完成。 |
| AST4-P1-030 | `LibraryCacheKey::fingerprint` 使用 `DefaultHasher`，只含 source/config/importer version（`artifact/cache_key.rs:1-27`）。 | cryptographic canonical build key，加入 function/recipe/input graph/target/toolchain/engine ABI/platform。 |
| AST4-P1-031 | artifact manifest 只有 schema/kind/revision/content hash/size/chunks；没有 producer、recipe、dependency closure、target、toolchain。 | self-describing `BuildOutputManifest`，每个 output 可追溯输入和兼容性。 |
| AST4-P1-032 | full scan 与 targeted watch 在 `scan_and_import.rs` 维护两套 prepare/commit 分支，rename/batch 又回退 full。 | 同一 build graph + input set；scope 只改变 invalidation frontier，不改变 publication protocol。 |
| AST4-P1-033 | watch 事件有 bounded batch，但 change 只有 URI/kind，缺 source generation、content fingerprint、producer 和 causality。 | qualified `AssetChange` 带 mount/source/event sequence、digest、reason、overflow/gap。 |
| AST4-P1-034 | reload 通过 `start_reload/fail_reload` 改 record 状态，未定义 last-good 与 dependents 的原子可见顺序。 | two-phase reload：prepare new payload/closure -> publish or retain last-good -> emit one receipt。 |
| AST4-P1-035 | `ZrPackWriter`/artifact store 能写压缩内容，但 pack 输入、cook target、mount table、sign/encrypt/provenance 不属于同一 graph。 | target cook session 产出 qualified artifacts，pack compiler 只消费已验证 output，install 有 mount/rollback receipt。 |
| AST4-P1-036 | importer worker、artifact chunk LRU、resource residency、GPU upload 各有预算，没有跨阶段 admission。 | hierarchical budget ledger（I/O/decode/CPU/GPU/cache）和 priority-aware scheduler，拒绝必须有 typed reason。 |

## 6. P2 性能、质量与维护

1. **AST4-P2-001**：`ResourceRegistry` 与 `AssetRegistryIndex` 的 `HashMap`/String key 在 100k/1M 资产下会产生重复 allocation；建立 page/query benchmark 和 peak RSS 基线。
2. **AST4-P2-002**：`AssetRegistryIndex::entries()` 每次排序；增加 stable page cursor、prefix/kind/tag index，禁止 UI/pack 直接调用全量 entries。
3. **AST4-P2-003**：`AssetMetaDocument.import_settings` 使用 `toml::Table`，typed recipe schema、canonical key order 和 migration telemetry 不完整。
4. **AST4-P2-004**：`DefaultHasher` cache fingerprint 不适合跨进程/跨平台；即使后续替换，也要保留 key schema version 和 migration report。
5. **AST4-P2-005**：`ensure_resident` 的 64 stripe 只减少同 id 竞争，不能按 asset size/priority 避免大资源 head-of-line blocking。
6. **AST4-P2-006**：chunk residency 有预算但 resource payload 没有统一 byte accounting；诊断无法回答“artifact/cache/CPU/GPU 谁占内存”。
7. **AST4-P2-007**：`ResourceLease` Drop 回调捕获 `ResourceManager` Arc，长寿命 lease 可拖住整个 authority；需要 owner scope 和 teardown leak report。
8. **AST4-P2-008**：ready facade 复制 payload，clone 成本没有 profiling counter；应区分 clone、Arc snapshot、range view 的 p95/p99。
9. **AST4-P2-009**：同步 `AssetImportError` 文本诊断缺 error code、actionability、input/output sequence；编辑器无法稳定聚合/本地化。
10. **AST4-P2-010**：importer registry availability rank 只基于 capability status；需要 probe TTL、provider health、quarantine 和 crash attribution。
11. **AST4-P2-011**：现有 ignored performance tests 不能替代真实 clean/incremental/cook/large artifact soak；补 deterministic fixture 与 fault matrix。
12. **AST4-P2-012**：watcher overflow/reconciliation 没有持续 telemetry（batch age、queue bytes、gap rate、rescan cost）；补入 runtime profile schema。
13. **AST4-P2-013**：artifact manifest max 4 MiB、raw payload max 2 GiB 是常量而非 target policy；移入 platform/cook profile 并版本化。
14. **AST4-P2-014**：内置资源通过 `builtin_resources()` 线性查找；构建时生成 typed lookup，避免每次 residency 扫描所有 builtin。

## 7. 参考引擎对照

| 参考 | 直接可核对的工程合同 | Zircon 当前差异 |
|---|---|---|
| Unreal DDC | `DerivedDataBuildDefinition.h:70-84` 将 build function、constants、input builds、bulk、files、hashes 收敛为 immutable definition，并由 serialized key 唯一标识。 | `LibraryCacheKey` 只有三个字符串/版本字段且用 `DefaultHasher`；artifact manifest 没有 function/target/toolchain/dependency provenance。 |
| Unreal SoftObjectPath / PrimaryAssetId | `SoftObjectPath.h:48-52` 明确 package/top-level asset/subobject；`PrimaryAssetId.h:129-176` 把 type/name 作为可查询稳定身份。 | `ResourceLocator` 有 label，但 label 与 subasset UUID/lineage 分离不足；没有 primary type/name 查询合同。 |
| Unreal AssetRegistryState | `AssetRegistryState.h:60-108` 提供依赖、package data、tag filter、mount/prune 和 cooked/development serialization 选项。 | Zircon registry 主要是内存 COW HashMap；tags/dependencies 没有 target-specific prune/serialization profile。 |
| Bevy AssetServer | `bevy_asset/src/server/mod.rs:50-80` 把 server、loader、source、mode、meta check 和 event channel 统一；`handle.rs` 区分 strong/index/UUID handle。 | Zircon ProjectAssetManager 把 project/watch/residency/importer/resource authority 混在一个结构中，handle 只有 ResourceId；没有 source mode/strong handle/UUID handle 分层。 |
| Bevy loader/io/meta | `loader.rs:97-155` 提供 extension、native conversion、async load、import settings；`io/mod.rs` 与 `meta.rs` 把 source/processed reader、meta transform 和 dependency 作为可替换合同。 | Zircon loader 主要是同步 typed function，source path 可被 importer 直接读取；meta、artifact、runtime payload 没有统一 async reader contract。 |
| Fyrox ResourceManager | `fyrox-resource/src/manager.rs:86-125` 明确 loaders、constructors、builtins、ResourceIo、registry、task pool、watcher 的 owner；`loader.rs:97-155` 有 async load/convert/import options；`state.rs:132-182` 把 Pending/Error/Ok 与 waker/data 绑定。 | Zircon 具有更多 generation/transaction 结构，但 `ensure_resident` 仍在调用线程做 I/O/decode；状态更细的 readiness 没有对应统一 request/waker/queue owner。 |
| Godot ResourceLoader/FileSystem/UID | `resource_loader.cpp:62-128,165-203` 支持 type、extension、UID、dependencies、rename_dependencies、cache mode；`editor_file_system.cpp:260-300` 扫描 UID/导入缓存；`resource_uid.cpp:127-212` 维护 path<->UID reverse cache。 | Zircon 有 locator/UUID/referencer，但缺统一 UID cache、type-aware dependency rename 和 scan/import cache validity contract。 |
| Unity Graphics importer | `AssetReimportUtils.cs:17-49` 用 batch boundary、progress、delegate、finally cleanup；`ShaderGraphImporter.cs:108-142` 声明依赖，`230-260` 生成 primary/subassets 并把 artifact dependency 纳入导入上下文。 | Zircon 有 batch/targeted import，但 recipe/dependency/artifact publication 仍分散；没有统一 per-output subasset receipt 和 progress/cancel contract。 |

## 8. 目标架构与重构顺序

```text
AssetMountRegistry
  -> AssetSourceSnapshot (read receipt / included files / mount generation)
  -> AssetBuildGraph (typed edges / recipe / target / toolchain)
  -> BuildScheduler (priority / budgets / cancellation / single-flight)
  -> ContentAddressedArtifactStore (manifest / provenance / chunks)
  -> ResourcePublication (record + payload + readiness + last-good)
  -> ResidencyManager (lease scope / CPU-GPU budget / eviction)
  -> QualifiedAssetEventStream (cursor / gap / resync / teardown)
```

1. **M188.0 先收口身份**：在 interface 定义 `AssetTypeId`、source/subasset identity、mount generation、typed handle 和 stale receipt；兼容层只能显式标记 deprecated。
2. **M188.1 source snapshot**：导入器禁止绕过 snapshot 直接打开路径；included/external/auxiliary 文件建立 content digest 与 reverse owner，完成 Runtime85 P0 的动态测试。
3. **M188.2 build graph/DDC**：将 recipe、function、dependency closure、target、toolchain、engine ABI canonicalize 成 cryptographic key；full/targeted/watch 共用一个 graph。
4. **M188.3 async load**：建立 load ticket、I/O/decode/upload stages、priority/cancel/deadline、single-flight 和 failure/last-good state machine；`load<T>` 只保留 facade 兼容包装。
5. **M188.4 publication/readiness**：把 registry/payload/dependency/readiness/revision 合并为一次 publication receipt，并让 world/ECS/editor 消费同一 generation fence。
6. **M188.5 residency**：把 artifact chunk、CPU payload、GPU/physics residency 纳入层级 budget ledger；lease 按 scope/priority/revision pin，eviction 可观测。
7. **M188.6 cook/package/install**：target cook session 只接收 validated graph outputs，pack/compiler 生成 mount/index/sign/encrypt/rollback receipt；运行时只读取 qualified package。
8. **M188.7 scale/verification**：100k/1M catalog、10 GiB texture/mesh、并发 scene load、hot reload storm、watch gap、provider crash、clean/incremental/cook cross-machine determinism 全部纳入门禁。

## 9. 资格门（30 个）

- **Identity**：旧 generation handle 永不读到新 payload；rename/redirect/subasset remap 有稳定 receipt；package/builtin/memory mount identity 不碰撞。
- **Registry**：id/path/kind/locator 冲突在 commit 层拒绝；查询使用 generation/page cursor；plugin unload 等待 in-flight reader/loader drain。
- **Source**：所有 importer 只能读 immutable snapshot；external buffer/image/font/OBJ companion 变化能定向失效父 asset；缺失/权限/overflow 有 typed terminal reason。
- **Build key**：recipe、function、source files、dependency closure、target、toolchain、ABI 任何一项变化都会改变 key；两台机器 clean/incremental hash 相同。
- **Dependency**：hard/soft/optional/runtime/editor-only edge 可查询；cycle 产生 SCC diagnostic；unresolved edge 触发 bounded retry，不静默变 NotLoaded。
- **Load**：API 不在主线程做未预算 I/O/decode；request 可取消、限时、合并、重试；completion 带 project/record/request generation。
- **Residency**：CPU/GPU/artifact bytes 统一计费；eviction 遵守 priority/last-good；lease 泄漏、超预算和 stale release 可诊断。
- **Publication**：record、payload、dependency readiness、revision 和 event receipt 原子可见；reload 失败保留 last-good 并明确错误。
- **Watch**：event 有 source/mount/sequence/digest/reason；gap 触发 bounded rescan；project close/plugin unload 后不再回调旧 owner。
- **Cook/package**：cook target/profile、compression/encryption/signing/mount index 和 artifact provenance 可回放；install/promotion/rollback 有 durable receipt。
- **Performance**：100k/1M query p95/p99、并发 load throughput、peak RSS、decode/eviction/upload latency、hot-reload soak 有基线，不能以单元测试替代。

## 10. Review-only 交付规则

本报告只记录当前工作树事实和后续重构计划，没有修改 Runtime、Editor 或 tooling 生产代码。本报告引用的旧报告仍需在每个里程碑开始前重新导出 focused manifest、fingerprint 和 dirty path；实现阶段不得把本报告的静态证据写成“已通过性能/恢复验证”。
