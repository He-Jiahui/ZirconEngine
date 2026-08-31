---
title: Runtime Core Resource、Asset、Serialization、Load、Artifact、Pack 与 Persistence 当前源码复核
category: zircon_runtime
report_id: Runtime161
review_date: 2026-08-30
baseline_head: 189f72219eaf16a6d0db880b53f3f68b4f5ee15a
verification_head: 189f72219eaf16a6d0db880b53f3f68b4f5ee15a
canonical_owner: Runtime04
refreshes:
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
related_reports:
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/87-runtime-asset-reference-identity-locator-guid-subasset-redirector-rename-move-resolution-repair-migration-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/88-runtime-asset-watch-change-ingress-coalescing-rename-overflow-targeted-reimport-generation-reload-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99m-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99w-runtime-asset-registry-index-persistence-rebuild-incremental-query-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/160-runtime-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-current-source-review.md
related_code:
  - zircon_runtime/crates/zr_resource/src
  - zircon_runtime/src/core/resource
  - zircon_runtime/src/core/framework/asset.rs
  - zircon_runtime/src/asset
  - zircon_runtime_interface/src/resource
  - zircon_editor/src
  - zircon_app/src
tests:
  - zircon_runtime/crates/zr_resource/src
  - zircon_runtime/src/asset
  - zircon_runtime_interface/src/resource
reference_engines:
  - dev/bevy/crates/bevy_asset/src/handle.rs
  - dev/bevy/crates/bevy_asset/src/server/info.rs
  - dev/bevy/crates/bevy_asset/src/server/mod.rs
  - dev/bevy/crates/bevy_asset/src/loader.rs
  - dev/bevy/crates/bevy_asset/src/io/mod.rs
  - dev/bevy/crates/bevy_asset/src/saver.rs
  - dev/Fyrox/fyrox-resource/src/loader.rs
  - dev/Fyrox/fyrox-resource/src/manager.rs
  - dev/Fyrox/fyrox-resource/src/state.rs
  - dev/Fyrox/fyrox-resource/src/registry.rs
  - dev/Fyrox/fyrox-resource/src/graph.rs
  - dev/Fyrox/fyrox-resource/src/untyped.rs
  - dev/godot/core/io/resource_loader.h
  - dev/godot/core/io/resource_loader.cpp
  - dev/godot/core/io/resource.h
  - dev/godot/core/io/resource.cpp
  - dev/godot/core/io/resource_uid.h
  - dev/godot/core/io/resource_uid.cpp
  - dev/godot/core/io/resource_format_binary.cpp
  - dev/godot/core/io/resource_importer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/StreamableManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/StreamableManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/IO/IoDispatcher.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/AssetManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/AssetManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/AssetRegistryState.h
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/IAssetRegistry.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/PackageFileSummary.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Serialization/CustomVersion.h
  - dev/UnrealEngine/Engine/Source/Runtime/PakFile/Public/IPlatformFilePak.h
  - dev/UnrealEngine/Engine/Source/Runtime/PakFile/Private/IPlatformFilePak.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Lighting/ProbeVolume/ProbeReferenceVolume.Streaming.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Lighting/ProbeVolume/ProbeVolumeStreamableAsset.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResources.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/ResourceReloader.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: partial_foundation_contract_and_product_closure_incomplete
source_recheck_required: true
---

# Runtime161 · Core Resource、Asset、Serialization、Load、Artifact、Pack 与 Persistence

## 1. 结论

当前 Runtime 已经不是完全临时的资源系统。项目打开会先恢复未完成 generation，再加载或重建 registry；full/targeted import 会在 candidate generation 中准备 meta、registry、artifact 与 resource mutation，并通过共享 durable transaction 提交；`ResourceSnapshot<T>`、`ResourceLease<T>`、immutable registry generation、bounded staging、readiness shard、reverse dependency closure、artifact BLAKE3/zstd、render semantic manifest、watch reconciliation、pack source-change detection 和 plugin update promotion 都是可以保留的基础。报告不能把这些真实实现误判为“全部缺失”。

但这些基础还没有形成工程级权威链。资源 handle/record 没有持久 exact asset type 与 schema identity；generic typed load 仍同步执行 artifact read/decode，并允许调用方获得深 clone；依赖图只有裸 `ResourceId`，环又被 readiness 投影直接视为 Loaded；artifact read 没有以 resource、type、schema、cook profile 和 build key 做预期身份准入；canonical artifact 的 64 KiB chunk 只是压缩物理切片，除 render mesh/texture 外没有可调度 semantic section；production watch publication 绕过 reload candidate/last-good transaction；source schema migration 只产出摘要，没有转换输入；`.zrpack` 仍整包驻留、未成为 Runtime mount/source，且没有签名、密钥、rollback-resistant generation 或 crash-safe publication 协议。

因此当前实现不能声称达到 Unreal/Godot/Fyrox/Bevy 的资源生命周期完整度，更不能声称性能优于 Unreal。性能优越必须在 typed admission、异步 range I/O、semantic streaming、version lease、bounded cancellation、100K/1M asset scale、fault injection 与产品冷启动/热重载基准通过后才能成立。

本报告刷新 Runtime04 的 12 项 canonical finding，**不新增唯一 finding**。11 项 P1 当前为 **7 Open、3 Partial、1 Closed**；1 项 P2 当前为 **0 Open、1 Partial、0 Closed**。9 项工程门为 **4 Fail、5 Partial、0 Pass**。Runtime64/85/86/87/88/99w 等专项中的 P0 只作为继承阻断项引用，不在本报告重复计数。

## 2. 审查边界与证据

### 2.1 当前源码选择

统计口径为 UTF-8 physical lines、non-empty lines、bytes、精确 `#[test]` / `#[ignore]`，fingerprint 为排序后的 `path<TAB>SHA-256<LF>` 再做 SHA-256。跨产品消费者选择包含 Runtime/Editor/App 中实际调用 project manager、resource snapshot/lease 与 load/acquire 的文件；它是 owner-focused selection，不等于用关键词覆盖率代替语义复核。

| 选择集 | files | lines | nonempty | bytes | tests | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---|
| Core resource authority / interface | **93** | **17,310** | **15,701** | **591,902** | **203** | **4** | `a36a091dd5d36c6039c2a1ba485ffccf5f889ada5f14143bb208813940ac368f` |
| Full Runtime asset system | **692** | **133,844** | **122,464** | **4,703,967** | **1,350** | **108** | `14b3b6ed7be7f00e09367c51887a35d9da0ce01d757101ec3a2043914b760401` |
| Runtime / Editor / App product consumers | **269** | **102,894** | **95,729** | **3,827,984** | **1,000** | **35** | `cdfb0659e1fc3199dce63ff6c1939623b138348ead8acdb714d8bf7e2ce26ef7` |
| Deduplicated Zircon union | **1,053** | **254,001** | **233,850** | **9,121,606** | **2,553** | **147** | `113fa6557b5a2dfa6e5fdc034d75801d500f861b58ac65cefe7d9e8f5f9703f1` |
| Unreal / Godot / Bevy / Fyrox / Unity Graphics reference | **35** | **45,536** | **39,546** | **1,684,211** | n/a | n/a | `39f51c6276204aa75c4edd5eb4984aab00751a4bc4b43f85a5edb2206052f54e` |

本轮只做静态 review 与文档更新，没有运行 Cargo、Editor、产品进程、fault、fuzz、scale、soak 或动态 benchmark。Tooling 按用户要求排除，也没有查询、轮询、等待或实时跟踪协调器状态。

### 2.2 产品主链

当前项目资源链不是单一函数，而是以下 transaction/generation 管线：

```text
ProjectManager::open
  -> recover_project_generation
  -> AssetRegistryIndex::load_or_rebuild
  -> ProjectAssetManager::open_prepared_project
      -> prepare_full_generation
          -> scan/import candidate
          -> prepare artifact/meta/registry/resource writes
      -> synchronize resource candidate
      -> verify preparation epoch + project generation
      -> commit durable transaction
      -> publish project/resource generation
```

`ProjectManager::open`、`AssetRegistryIndex::load_or_rebuild`、`full_generation` 和 `durable_transaction` 已给出 recovery、candidate mutation、precondition、journal 与 durability 的真实骨架。`open_prepared_project` 中 `RecoveryDeferred` 与 publication 顺序的剩余 P0 由 Runtime51/99w 拥有，本报告不重复登记。

### 2.3 可保留基础

| 区域 | 当前事实 | 保留与收束方向 |
|---|---|---|
| Project persistence | open 前 recovery；registry 可 load-or-rebuild；meta/registry/artifact/resource mutation 共享 candidate generation | 保留 durable transaction，统一 publication fence 与 recovery receipt |
| Resource snapshots | `ResourceSnapshot<T>` 与 `ResourceLease<T>` 已存在，lease Drop 会释放 token | 作为 immutable version slot 的 API 基础，淘汰 clone-first 产品路径 |
| Registry/readiness | immutable generation、staging budget、readiness shard、reverse closure 已存在 | 将 payload/version 与 graph/query 分域，移除单 authority write-lock 热点 |
| Artifact integrity | BLAKE3、zstd、chunk table、schema/revision/broad kind 检查已存在 | 增加 exact type/schema/cook/build identity 和 semantic section manifest |
| Render streaming | manifest v3 已表达 texture mip/layer、mesh LOD/cluster page，支持优先级、deadline、ticket | 下沉为通用 section source；render 只是首个 consumer |
| Reference resolution | 缺失 label 返回 `DanglingSubasset` / `MissingAssetLabel`，不再静默回退 parent | 保留 fail-closed 行为，继续由 Runtime87 完成 GUID/path repair |
| Watch/import | bounded ingress、overflow reconciliation、targeted/full generation 与 source-change precondition 已存在 | 必须接入 candidate reload、last-good 与 dependency-aware invalidation |
| Pack/update | writer 会二次读取并检测 source change；native plugin update 已能调用 promotion | 改造成 mountable、signed、generation-qualified bundle transaction |

## 3. Runtime04 canonical finding 当前重判

### 3.1 P1

| ID | 当前状态 | 当前源码证据 | 与参考实现的差距 | 重构要求 |
|---|---|---|---|---|
| `R04-P1-01` | **Open** | `ResourceRecord` 仅保存 broad kind、locator、artifact、revision/state、裸 dependency id 与 importer metadata；selected source 中没有 `AssetTypeId`、`SchemaId`、`QualifiedAssetHandle` 或等价持久合同。payload 只在运行时用 `TypeId` downcast | Bevy untyped handle 持有 type id；Fyrox loader/manager 以 data type UUID 准入；Unreal `FPrimaryAssetId` 至少具有 type+name | 建立 `AssetTypeCatalog`、`QualifiedAssetKey/Handle<T>`、schema/version compatibility；record、reference、artifact、registry row 与 load request 使用同一身份 |
| `R04-P1-02` | **Partial** | snapshot/lease 已实现，但 `load_typed<T: Clone>` 仍从 snapshot `.clone()`；当前 production 选择有约 70 个 legacy `load_*_asset` call site、14 个 snapshot call site、0 个 acquire call site | Bevy strong handle 维持生命周期；Fyrox typed resource 共享状态，不以深复制作为默认消费 | 产品 API hard-cut 到 lease/snapshot；clone 仅允许显式 detached copy，并计量 bytes/time；大对象禁止隐式 clone |
| `R04-P1-03` | **Open** | generic `ensure_resident` 在调用线程持 residency lock、读取 prepared artifact、decode 并 publish；typed/snapshot/acquire 都进入该路径。专用 render loader 的 priority/deadline/ticket 没有接入 generic authority | Unreal StreamableHandle 支持 priority/cancel/combined request；Godot threaded request/status；Fyrox 使用 async task pool | 建立 `AssetLoadCoordinator`，让 I/O、decode、dependency admission、publish 分阶段异步执行；所有 stage 有 cancel/deadline/budget/receipt |
| `R04-P1-04` | **Open** | `ResourceRecord.dependencies` 是 `Vec<ResourceId>`；readiness 遇到回访节点直接投影 Loaded，没有 edge type、requiredness、category、SCC admission | Unreal AssetRegistry 区分依赖 category/property；Bevy 区分 direct/recursive dependency load state | 建立 immutable typed dependency graph generation；显式 hard/soft/editor-only/runtime/cook edge、SCC policy、cycle diagnostic 与 closure receipt |
| `R04-P1-05` | **Open** | artifact header 只有 schema、broad kind、revision、content hash、raw/compressed bytes 与 chunks；read 只接收 path/URI，不接收 expected resource/type/schema/cook descriptor | Unreal package/chunk identity 与 AssetManager/registry/cook target 相互约束；Godot binary loader持 format/type/resource contract | 建立 `CookArtifactKey` 与 `ExpectedArtifactRead`：Project/BuildSet、resource id、exact type/schema、source/importer/config/build graph、platform/quality、section manifest 全部参与准入 |
| `R04-P1-06` | **Partial** | canonical artifact 把整份 bincode payload 压成单帧再切 64 KiB chunk，read 会完整解压/反序列化。render manifest v3 已支持 mip/layer/LOD/cluster page，但 scene/animation/audio 等仍是 full object | Unity Graphics probe streaming 以 offset/size 异步读取、取消、双缓冲 staging 与 budget/eviction 工作；Unreal IoDispatcher 按 chunk/range 调度 | 提炼 `SemanticArtifactManifest + SectionSource`；将 mesh/texture 实现下沉并扩展 scene cell、animation segment、audio block、shader library 等 |
| `R04-P1-07` | **Open** | production watch publication 主要调用 `upsert_lazy`；未发现 product path 驱动 `start_reload -> candidate -> complete_reload/fail_reload`。当前 reload state machine 没有成为唯一入口 | Godot cache replace/deep replace 与 threaded status；Unreal streamable request/version replacement 都保留可观察请求状态 | 建立 `ReloadCandidateTransaction`：active/last-good/candidate 三槽、dependency validation、safe-point swap、failure retention、generation-qualified event |
| `R04-P1-08` | **Closed** | `reference_resolver` 对带 label locator 做精确匹配，缺失时返回 `DanglingSubasset` 并列出 candidate；artifact access 返回 `MissingAssetLabel`，没有 parent fallback | 与 mature engine 的 fail-closed subresource resolution 方向一致 | 保持关闭；Runtime87 的 GUID/path rebinding 与 repair 问题是不同 finding，不在此重开 |
| `R04-P1-09` | **Open** | `migrate_source_schema` 只接收 source version 并返回 summary，不接收或返回 source bytes/document；production 没有迁移调用 | Unreal CustomVersion/package summary 和 Godot format version 都把版本准入绑定到实际反序列化/迁移路径 | 建立 `AssetMigrationCatalog` 与 typed migration pipeline；输入/输出 schema、deterministic receipt、reference rewrite、round-trip/unknown preservation、rollback 都必须可验证 |
| `R04-P1-10` | **Partial** | pack manifest v1 只有 path/hash/size；reader 持有完整 `Vec<u8>` 并逐 chunk 验证、clone asset bytes；writer 有两遍读取/source change detection，但 assembler 仍构造完整输出 buffer。Runtime 没有 `pack://`/mount source | Unreal IoDispatcher 可 mount backend 并做异步 range read；Pak 有 mount/index/block 等运行时合同 | 建立 `AssetSourceMountRegistry` 与 seek/range reader；bundle index、chunk/section offset、compression/encryption、priority/cancel、mount generation 与 cache policy 必须进入 Runtime source authority |
| `R04-P1-11` | **Open** | pack tree 没有 signature/public key/key id/revocation/trust/fsync/active generation；promotion 以 rename/copy 和整包读取完成，restore error 还可能被忽略 | Unreal IoDispatcher/Pak 提供 signature error、encryption key 与 mount lifecycle | 建立 `SignedBundleGeneration` 与 `BundlePublicationTransaction`：签名链、key rotation/revocation、anti-rollback、staged verify、directory durability、active pointer CAS、crash recovery、native update admission |

P1 合计：**7 Open、3 Partial、1 Closed**。

### 3.2 P2

| ID | 当前状态 | 当前源码证据 | 重构要求 |
|---|---|---|---|
| `R04-P2-01` | **Partial** | `ResourceManager` 仍以单一 `Arc<RwLock<ResourceAuthority>>` 持 registry、management、payload、runtime 与 readiness，另有 commit mutex。COW registry、immutable generation、readiness shard、reverse closure 与 bounded staging 已减轻部分竞争，但 100K/1M asset lock wait、event fanout、resident churn 和 soak 没有 accepted evidence；多项规模/性能测试仍 `#[ignore]` | 拆成 `ResourceAuthoritySnapshot`、version/payload slots、graph/query generations 与 admission/commit domains；定义 p50/p95/p99 lock wait、load latency、bytes、queue age、eviction、reload stall 指标与产品预算 |

P2 合计：**0 Open、1 Partial、0 Closed**。

## 4. 继承阻断项，不重复计数

| Owner | 继承事实 | 本报告边界 |
|---|---|---|
| Runtime64 / Runtime112 | public payload admission 仍可形成 exact type 错配；frame path 同步冷加载/decode/clone | 本报告以 `R04-P1-01..03` 描述父缺口，不复制专项 P0 |
| Runtime85 | auxiliary glTF/font source 没有进入 source digest/watch reverse owner，可复用 stale artifact | 由 import/build graph owner 修复 |
| Runtime86 | ImportedAsset 类型、dependency extractor 与 exact type/schema catalog 未闭合 | 由 asset schema owner 修复 |
| Runtime87 | GUID/path/subasset rename-move repair 与 rebinding 未闭合 | `R04-P1-08` 仅关闭 parent fallback，不代表 reference identity 完成 |
| Runtime88 | watcher error/reconciliation/compound source owner 与 reload generation 未闭合 | 本报告只拥有 resource reload transaction 父接口 |
| Runtime51 / Runtime99w | `RecoveryDeferred` publication ordering、registry row identity/currentness 未闭合 | 由 project registry/persistence owner 修复 |
| Runtime160 | mount/source provider、secure path/range I/O/direct-fs governance 未闭合 | 本报告消费其 provider，不重复定义 filesystem finding |

## 5. 参考引擎逐项对照

| 参考源码 | 可吸收的工程合同 | Zircon 当前事实 | 采用边界 |
|---|---|---|---|
| Bevy Asset | typed/untyped handle identity、strong handle residency、direct/recursive dependency load state、async reader/saver | snapshot/lease 有底座，但 handle 不保存 exact type，generic load 同步，dependency edge 无类型 | 吸收身份和状态分层，不复制 ECS API 表面 |
| Fyrox Resource | loader data type UUID、typed request check、async loader/task pool、resource state/reload | Zircon 只有 runtime downcast 和 broad kind，异步 worker 未成为 generic owner | 采用 type UUID/catalog admission 与 async state machine |
| Godot ResourceLoader | UID/path cache、threaded request/status、cache replace policy、binary format/version、external/internal resource offsets | Zircon 有 GUID/locator 和 partial migration，但没有统一 cache/reload policy 与真实 source transform | 采用 request/cache/version语义；稳定 identity 继续由 Runtime87 拥有 |
| Unreal Streamable/IoDispatcher/AssetManager/Registry/Package/Pak | request handle/priority/cancel、mountable async chunk/range I/O、primary asset type、typed dependency category、package/custom version、signature/encryption | Zircon 的 specialized render tickets、pack/artifact/registry 是局部底座，尚未连成 product authority | 作为目标资格线；不直接移植宏大 UObject/Package 实现 |
| Unity Graphics probe/render resources | offset/size async read、cancel、double-buffer staging、per-cell budget/eviction、resource reload utility | Zircon render semantic manifest 接近局部方向，但 canonical artifact 和其他资产仍 full-object | 仅作为 Graphics streaming consumer 证据，不把 Unity Graphics 冒充完整 AssetDatabase |

## 6. 目标架构与所有权

### 6.1 固定包边界

| 包 | 必须拥有 | 明确禁止 |
|---|---|---|
| `zircon_runtime` | resource/type/schema identity、load coordinator、version lease、dependency graph、artifact/section source、migration、bundle mount/publication、metrics | 不把 Runtime authority 上移给 Editor；不新增第四个顶层 server/package |
| `zircon_editor` | importer/authoring UI、repair proposal、save/build workflow、diagnostic projection | 不持有第二套 registry、payload cache、bundle mount 或 reload truth |
| `zircon_app` | host composition、Runtime creation/shutdown、fatal durability/update result consumption | 不解析 artifact/pack，不成为资源业务 owner |

### 6.2 Runtime 内部权威脊柱

```text
AssetTypeCatalog
  -> QualifiedAssetKey / QualifiedAssetHandle<T>
  -> AssetLoadCoordinator
      -> TypedDependencyGraphGeneration
      -> ExpectedArtifactRead(CookArtifactKey)
      -> SemanticArtifactManifest / SectionSource
      -> ResourceVersionSlot / ResourceVersionLease
      -> ReloadCandidateTransaction(active, candidate, last-good)

AssetSourceMountRegistry
  -> SignedBundleGeneration
  -> BundlePublicationTransaction

AssetMigrationCatalog
  -> typed source/document/reference migration receipts

ResourceAuthoritySnapshot
  -> immutable query/readiness/diagnostic generations
```

`ResourceManager` 应成为组合 facade，而不是继续让一个 write lock 同时拥有 registry projection、payload、readiness 与 runtime bookkeeping。load coordinator 必须只通过不可变 descriptor 和 generation-qualified commit 接触各域；Editor 只能订阅 snapshot/receipt。

## 7. 依赖顺序重构计划

| Milestone | 工作 | 完成证据 |
|---|---|---|
| M0 · Contract freeze | 冻结 Runtime04/64/85/86/87/88/99w finding owner；定义 type/schema/cook/load/version/dependency/bundle ID grammar | schema 文档、compile-time type tests、旧/新 API inventory；没有双真相 compatibility facade |
| M1 · Exact identity | 实现 `AssetTypeCatalog`、qualified key/handle、record/registry/reference/artifact exact identity admission | wrong-type/wrong-schema 必须在 publish 前拒绝；跨进程/跨平台 corpus 稳定 |
| M2 · Versioned loading | 实现 load coordinator、request handle、priority/deadline/cancel、version slot/lease；产品 load hard-cut | caller thread 不执行冷盘 read/decode；cancel/timeout/late completion 不污染 active version |
| M3 · Typed graph | 引入 typed edge、requiredness/category、SCC admission、closure receipt 与 generation snapshot | cycle、missing hard dependency、soft dependency、recursive/direct state 全部 deterministic |
| M4 · Artifact qualification | 引入 cook key、expected read、semantic manifest/section hash；迁移 render manifest | wrong BuildSet/platform/profile/source/importer/schema artifact fail-closed；section corruption 可定位 |
| M5 · Semantic streaming | 将 mesh/texture loader 下沉为通用 section source，扩展 scene/animation/audio/shader consumer | range read、budget、priority、cancel、eviction、re-request 和 no-full-decode tests |
| M6 · Reload transaction | watch/import publication 统一走 active/candidate/last-good；dependency validation 后 safe-point swap | failed reload 保留 last-good；gap/overflow/reconcile/rename 具有 generation receipt |
| M7 · Migration/persistence | source/document/reference migration 进入真实 decode/import/save/reopen；统一 transaction publication fence | golden corpus、unknown preservation、N-2/N-1/current、crash/fault/reopen、idempotent recovery |
| M8 · Bundle authority/scale | `.zrpack` 改为 signed mountable bundle；实现 crash-safe active generation；拆分 authority hot locks | tamper/key rotation/revocation/rollback/crash matrix；100K/1M asset scale、soak 与产品 benchmark 达标 |

M0-M3 是 F1/F3/F4 项目、资产、持久化 MVP 的先决条件；不能为了先做高级 streaming 而继续保留不精确身份和同步 generic load。

## 8. 工程资格门

| Gate | 状态 | 当前缺口 | 通过标准 |
|---|---|---|---|
| Unit / contract | **Partial** | 局部单测多，但 exact identity/load request/migration/bundle trust 合同不完整，且有 ignored release gate | 所有新 ID/schema/state transition 有正反例与 compatibility corpus，无关键 ignored |
| Transaction / reopen | **Partial** | project durable transaction 是真实底座，但 resource publication、reload、bundle active generation 尚未统一 | prepare/commit/publish/durable/recover 状态机可重放，任意阶段 crash 后 reopen 唯一收敛 |
| Concurrency | **Partial** | candidate generation 与部分 single-flight 已有；generic residency 同步且 authority lock 粗 | cancel/race/late completion/reload/evict/lease/parallel project tests 通过，锁顺序有模型 |
| Fault injection | **Fail** | artifact read、decode、reload swap、bundle promotion/key failure 没有端到端故障矩阵 | I/O short read/corrupt/ENOSPC/rename failure/process kill/key revoke 全部 fail-closed/last-good |
| Scale / performance | **Fail** | 没有 accepted 100K/1M asset、lock wait p95/p99、queue/bytes/soak | 冻结硬件与数据集，给出相对 Unreal/reference baseline、预算、回归阈值和原始 artifact |
| Streaming / memory | **Partial** | render mesh/texture 有 semantic manifest；canonical artifact 与其他资产 full-object | section range read、bounded staging/residency/eviction、no hidden clone/full decode |
| Migration / reference | **Partial** | subasset label fail-closed；真实 source transform、GUID repair/redirect 与 serialized reference closure 未闭合 | N-2/N-1/current golden corpus、rename/move/subasset/unknown fields round-trip、migration receipt |
| Security / update | **Fail** | pack 无 signature/key/revocation/anti-rollback/crash-safe active generation | signed manifest/chunks、key policy、rollback protection、mount admission、durable atomic activation |
| Product integration | **Fail** | production 仍以 clone load、同步 residency、upsert reload 与非 mount pack 工作 | App/Runtime/Editor 全链只走新 authority；冷启/热载/save-reopen/package-update scenario 通过 |

Gate 合计：**4 Fail、5 Partial、0 Pass**。

## 9. 实施约束与验收交接

1. 先写 architecture contract 与 failure model，再改代码；不得以更多 facade、type alias 或空 driver 代替 owner 收束。
2. Runtime04 是本报告 12 项 finding 的 canonical owner；专项报告可以拆实现，但不得复制计数。
3. 迁移采用 hard cutover：新 typed handle/load/artifact/bundle authority 产品接通后，旧 clone-first、sync residency、raw pack reader 生产入口必须删除。
4. 每个 milestone 都要提供底层单元、transaction/reopen、并发/fault、scale/product 四层证据；上层 scenario 不能替代底层失败测试。
5. “优于 Unreal”必须是测量结论，不是设计目标文字。至少冻结同硬件、同资产集、同冷/热缓存、同压缩质量、同线程预算和相同正确性门，再比较 cold load、stream stall、memory、reload latency 与 package mount/update。
6. 本报告只完成 review；没有修改 Runtime/Editor/App 生产代码，也没有把未运行的动态验证写成通过。

下一实施会话应从 M0/M1 开始，把 Runtime04、Runtime64、Runtime86 的身份合同合并为一个可编译设计记录；在 exact identity 通过前，不应先扩建更多 asset-specific loader。
