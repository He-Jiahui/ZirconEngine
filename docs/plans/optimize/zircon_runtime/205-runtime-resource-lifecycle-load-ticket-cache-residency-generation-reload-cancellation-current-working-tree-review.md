---
title: Runtime Resource 生命周期、Load Ticket、Cache、Residency、Generation、Reload 与 Cancellation 当前工作树复审
category: zircon_runtime
report_id: Runtime205
review_date: 2026-08-31
baseline_head: working-tree
related_code:
  - zircon_runtime_interface/src/resource
  - zircon_runtime/crates/zr_resource/src
  - zircon_runtime/src/asset/facade
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/pipeline/worker_pool
  - zircon_runtime/src/asset/artifact
  - zircon_runtime/src/asset/importer
  - zircon_runtime/src/asset/watch
  - zircon_runtime/src/asset/pack
  - zircon_runtime/src/graphics/scene/resources/render_asset_residency
plan_sources:
  - docs/plans/optimize/zircon_runtime/99m-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/188-runtime-asset-resource-lifecycle-locator-registry-load-cache-import-cook-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/204-runtime-filesystem-resource-io-path-atomic-transaction-recovery-security-current-working-tree-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/StreamableManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/AssetManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/Serialization/AsyncLoading2.h
  - dev/UnrealEngine/Engine/Source/Developer/DerivedDataCache/Public/DerivedDataBuildDefinition.h
  - dev/bevy/crates/bevy_asset/src/server/mod.rs
  - dev/bevy/crates/bevy_asset/src/handle.rs
  - dev/bevy/crates/bevy_asset/src/loader.rs
  - dev/bevy/crates/bevy_asset/src/io/mod.rs
  - dev/Fyrox/fyrox-resource/src/manager.rs
  - dev/Fyrox/fyrox-resource/src/loader.rs
  - dev/Fyrox/fyrox-resource/src/state.rs
  - dev/godot/core/io/resource_loader.h
  - dev/godot/core/io/resource_loader.cpp
  - dev/godot/core/io/resource_uid.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourceRegistry.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Material/AssetReimportUtils.cs
doc_type: current_source_review
review_status: review_complete
implementation_status: pending
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Runtime Resource 生命周期、Load Ticket、Cache、Residency、Generation、Reload 与 Cancellation 当前工作树复审

## 1. 结论

当前工作树不能再概括为“没有工程化资源系统”。`zr_resource` 已形成单写提交、批量 preflight、typed handle、payload/snapshot/lease、management/readiness immutable generation、bounded event log 和冲突拒绝；Project asset 链已有 project generation token、prepare/commit fence、watch bounded ingress；artifact 层已有 content-addressed chunk、BLAKE3 校验、zstd/bincode 边界和 LRU byte budget；Render semantic residency 更进一步拥有 manifest/block 两级 ticket、single-flight、priority、deadline、I/O dispatch budget、retained-byte admission、取消和 owner close。

但这些能力仍是多个局部闭环。通用产品 API `ProjectAssetManager::load<T>`、`load_*_asset`、`acquire_*_asset` 最终仍同步执行 `ensure_resident -> prepare_artifact_read_by_id -> read/decode -> store_payload`；`AssetWorkerPool` 只接受独立的 `TextureSource/MeshSource`，不消费 `ResourceLocator`、project generation、record revision 或 `ResourceManager` publication；Render ticket 体系只接到 graphics semantic block residency。因而 Zircon 尚不存在一个覆盖 source/provider、artifact、CPU payload、dependency、GPU upload、reload 和 eviction 的统一 `ResourceLoadAuthority`。

本轮不新增独立 P0，继续继承 Runtime99m 的 2 个 RAR P0。对 Runtime188 的 36 个 AST4 P1 逐项复核后，当前状态为 **22 Open / 13 Partial / 1 Closed**；14 个 AST4 P2 为 **9 Open / 5 Partial**。唯一可关闭的是 AST4-P1-008：当前 `ResourceManager::prepare_commit` 已在生产路径强制校验 kind、locator ownership、explicit rename 和 revision conflict，不再依赖 unchecked insert。其余局部进展不能被提升为整个资源生命周期完成。

## 2. 审查范围与证据

### 2.1 当前生产文件清单

本轮对下列生产目录去除 `tests`、`*_tests.rs`、`optimization_tests.rs`、`zr_resource/src/io`（由 Runtime204 独立拥有）后建立 focused manifest：

| 维度 | 当前值 |
|---|---:|
| Rust 生产文件 | 236 |
| 总行数 | 46,515 |
| 非空行 | 42,445 |
| 字节数 | 1,636,772 |
| path + SHA-256 manifest 指纹 | `c710ff454a0e5aca18119b11fe1c3ffd08ce8d364c8432e6d25d131e5dd455e5` |

覆盖链包括：

1. interface identity：`ResourceLocator`、`ResourceId`、typed/untyped handle、record/event/state；
2. `zr_resource`：registry、mutation、receipt、payload、snapshot、lease、runtime slot、management/readiness generation、event stream；
3. product facade：`ProjectAssetManager`、typed load/acquire、generation fence、resource publication、watch；
4. execution/cache：`AssetWorkerPool`、artifact store/chunk residency、Render manifest/block loader；
5. source/build/package：importer contract/registry、watch、pack/install；
6. graphics consumer：Render semantic block residency 与 resource streamer 接线。

### 2.2 证据等级

- **E3**：读取当前生产实现并沿调用链追踪 owner、状态转换、publication 和 product consumer。
- **E2**：读取相邻测试声明与本地参考引擎源码，核对合同存在性；测试声明不等于动态通过。
- **E1**：本轮是 review-only，没有运行 Cargo、真实项目导入、故障注入、热重载风暴、内存压力或跨平台 package。
- **E0**：没有相同 workload 下的正确性和 p50/p95/p99/p99.9 证据，不能声称性能或表现优于 Unreal。

## 3. 当前可保留的工程底座

### 3.1 `zr_resource` authority

1. `ResourceManager::prepare_commit` 以 commit serial 固定 mutation 顺序，在只读 authority 上完成全部 preflight，再一次性应用 staged registry/payload/runtime/readiness 变化。
2. batch preflight 已拒绝 kind conflict、locator occupied、implicit rename、invalid transition、revision conflict 和 sequence exhaustion；这是生产一致性合同，不是 debug-only assertion。
3. `ResourceMutationReceipt` 同时给出 changed/removed record、management/readiness projection snapshot 和 event count；`ResourceSnapshot<T>` 固定 record revision 与 payload Arc。
4. management projection 已采用 1,024 id shard、1,024 locator shard和 ordered page；readiness generation 已采用 64 shard和 reverse dependency closure。
5. event stream 有 entry/byte/age 上限、sequence、lagged gap、coalescing 和 subscriber-local cursor。

### 3.2 project generation 与 publication

1. `ProjectAssetGenerationToken { project_root, catalog_sequence }` 能阻止候选项目代在被替换后提交。
2. `commit_resource_batch_after_dependencies` 先 reserve resource mutation，依赖文件/项目状态成功后才应用 resource commit，避免暴露 compound import 的前缀。
3. watch activation 有 Pending/Draining/Active/Retired 生命周期，能在 project close 时清队列并切断旧 watcher。
4. `ProjectCatalogInputGeneration` 已做 64 shard COW 和 predecessor delta，targeted import 不再必然全量复制 catalog input。

### 3.3 bounded worker、artifact 与 Render ticket

1. `AssetWorkerPool` 有 unique-request admission、waiter capacity、single-flight、request/completion max age、completion entry/byte budget、panic capture 和 bounded diagnostics。
2. artifact store 对 manifest/raw payload/chunk size设上限，使用 immutable chunks、BLAKE3、zstd、atomic manifest publication 和完整读后校验。
3. `ArtifactChunkResidency` 有 byte budget、LRU eviction、external Arc lease accounting、bounded retired-lease metadata 和 trim diagnostics。
4. Render manifest/block loader 有 per-entry/per-ticket/global retained-byte上限、priority/deadline frontier、ticket cancel/drop cancel、owner close、I/O/decode task scope与 typed terminal reason。

这些基础应被上提为公共资源加载合同，而不是删除后重写成另一套临时 loader。

## 4. 唯一 P0 owner

| Owner | 状态 | 本轮证据 |
|---|---|---|
| Runtime99m `RAR-P0-001` exact type/authority | Open | marker kind 只验证固定 `ResourceKind`；payload downcast 与 `load_imported_asset` 的 `or_else` 探测仍可把同 kind 的多种 schema 当成候选。 |
| Runtime99m `RAR-P0-002` product blocking load | Open | `ProjectAssetManager::load<T>` 与全部 `load_*_asset` 在调用线程进入同步 artifact read/decode；renderer/scene/text 等 consumer 仍可直接调用这些 API。 |

Runtime204 的 provider/mount/opened-root I/O owner 是这两个 P0 的前置依赖，但不在本报告重复计数。

## 5. Runtime188 P1 currentness 总账

### 5.1 Identity、catalog 与 authority

| ID | 状态 | 当前工作树证据 | 仍需重构 |
|---|---|---|---|
| AST4-P1-001 | Open | `ResourceHandle<T>` 与 facade `Handle<T>` 仍只保存 `ResourceId`。 | handle 加入 live instance / publication generation / exact type，并提供 typed stale 结果。 |
| AST4-P1-002 | Open | locator label 仍是任意字符串，display label、subasset stable identity 和 lineage 未分离。 | source UID、subasset UID、display label、redirect/remap 分层。 |
| AST4-P1-003 | Open | `ResourceRegistry`、`AssetRegistryIndex`、`ProjectCatalogInputGeneration`、`ProjectAssetManagementGeneration` 并行保存相交事实。 | 固定一个 catalog authority，其余全部是带 generation 的 projection。 |
| AST4-P1-004 | Partial | `zr_resource` 已有 ordered page/shard，Project catalog input 也有 64 shard；Editor/project `AssetRegistryIndex` 仍维护独立全量索引和复制查询。 | 将 page cursor、prefix/kind/tag query 移到唯一 catalog generation。 |
| AST4-P1-005 | Open | `AssetRegistryEntry::source_digest` 仍为无算法字符串；catalog input另存 mtime/meta，未形成 source snapshot identity。 | canonical `SourceFingerprint` 记录算法、size、digest、included files和 observation confidence。 |
| AST4-P1-006 | Open | `ResourceKind/AssetKind` 是固定枚举，plugin 类型没有 namespaced exact type/schema/opaque preservation。 | 建立 stable `AssetTypeId + schema + codec/provider descriptor`。 |
| AST4-P1-007 | Open | locator 只有 Res/Library/Package/Builtin/Memory scheme，没有 mount id/generation/capability/trust/health。 | 依赖 Runtime204 建立 provider/mount generation resolution receipt。 |
| AST4-P1-008 | **Closed** | commit preflight 已强制校验 kind、locator ownership、explicit rename、revision与状态转换，并返回 typed `ResourceRegistryError`。 | 保持 negative matrix；后续不得重新开放 unchecked live insert。 |

### 5.2 Load、payload 与 residency

| ID | 状态 | 当前工作树证据 | 仍需重构 |
|---|---|---|---|
| AST4-P1-009 | Partial | Render manifest/block loader 与 `AssetWorkerPool` 已有 ticket/single-flight/budget/cancel；通用 `ensure_resident` 仍同步。 | 将通用 resource load 接入同一 request graph，兼容同步 API只能等待既有 ticket。 |
| AST4-P1-010 | Open | `load_imported_asset` 对所有 kind 手写 match，并对 Texture/UI、UiLayout/V2、UiStyle/Theme 使用失败驱动 `or_else`。 | exact type descriptor 选择唯一 decoder，禁止拿解析失败作类型探测。 |
| AST4-P1-011 | Partial | `ResourceSnapshot<T>` 已固定 revision；只有少数 typed snapshot API，常规 facade仍 clone完整 payload。 | 默认返回 immutable snapshot/view；显式 clone记录 byte cost；支持 range/mip/LOD view。 |
| AST4-P1-012 | Open | `ResourceLeaseIdentity` 是空类型，lease没有 owner、purpose、priority、deadline、budget class或 pinned revision。 | lease identity结构化并绑定 owner scope、publication和 memory ledger。 |
| AST4-P1-013 | Open | 最后一个 lease Drop 会立即删除 payload；artifact chunk虽有 LRU byte budget，resource payload没有 warm retention或统一预算。 | CPU payload residency采用预算化 trim，不用 refcount==0 直接决定 eviction。 |
| AST4-P1-014 | Open | lease identity只用 process-local `Arc::ptr_eq`，未绑定 project generation、record revision或 load request。 | `(project, mount, record, request, payload revision)` 形成可诊断 token。 |
| AST4-P1-015 | Open | runtime state仍只有 Unloaded/Loading/Loaded/Error/Reloading，通用 load没有 queued/decode/upload/dependency-wait stage。 | 一个可序列化、可取消、可回放的阶段状态机拥有全部 transition。 |
| AST4-P1-016 | Partial | management/readiness read path已发布 immutable generation；registry/payload/lease/readiness commit仍共享一个 `RwLock<ResourceAuthority>` 和 commit mutex。 | 写路径按 page/shard 或 immutable root publication收口，证明 contention 上界。 |
| AST4-P1-017 | Partial | catalog input和 projections已有 shard/page COW；live registry仍为 `Arc<HashMap>` 根级 COW。 | live registry按 persistent page/shard增量发布并导出 changed-page receipt。 |
| AST4-P1-018 | Open | authority、registry、loader多处在 poisoned lock 后 `into_inner` 继续写。 | 写 owner进入 Poisoned/RepairRequired；只读降级和写拒绝必须分开。 |

### 5.3 Dependency、publication 与 reload

| ID | 状态 | 当前工作树证据 | 仍需重构 |
|---|---|---|---|
| AST4-P1-019 | Open | `dependency_ids: Vec<ResourceId>` 仍无 hard/soft/optional/source/runtime/editor phase。 | typed edge schema和phase-specific closure。 |
| AST4-P1-020 | Partial | readiness已有 reverse dependency closure，catalog input有 direct reference snapshot；edge provenance和target/build/runtime分层仍缺失。 | 唯一 dependency manifest与incremental reverse pages。 |
| AST4-P1-021 | Open | missing dependency主要表现为 record diagnostic，没有 unresolved node、mount-change retry trigger和bounded retry receipt。 | missing edge成为一等 graph state。 |
| AST4-P1-022 | Open | readiness递归聚合有visited保护，但没有统一 SCC build、cycle policy或partial-load contract。 | publication前计算 SCC并固定允许/拒绝策略。 |
| AST4-P1-023 | Partial | `ResourceProjectionSnapshot` 已同锁捕获 management+readiness；receipt仍不含 payload revision identity、project/mount generation和event sequence区间，`store_payload` 还丢弃receipt。 | 一个 publication receipt覆盖 record/payload/dependency/readiness/event/project generation。 |
| AST4-P1-024 | Open | event receiver有cursor/gap，但没有 ack、gap->snapshot handshake、consumer retirement或resync authority。 | 订阅协议显式返回 current snapshot及resume cursor。 |
| AST4-P1-025 | Open | record revision仍承载多种变化，source/build/payload/device revision未分层。 | 分层revision只触发受影响阶段。 |
| AST4-P1-026 | Partial | project generation token和commit fence已存在；token只有 root path+catalog sequence，candidate `ProjectManager` 仍clone复合状态。 | immutable build session输入与prepared output分离，token纳入mount/provider/source snapshot。 |

### 5.4 Import、artifact、watch 与 package

| ID | 状态 | 当前工作树证据 | 仍需重构 |
|---|---|---|---|
| AST4-P1-027 | Open | importer registry主要按 full suffix/extension、priority和capability status选取；无magic/MIME/schema probe receipt。 | deterministic sniff/candidate/winner pipeline。 |
| AST4-P1-028 | Open | importer注册和runtime typed loader仍是两套中心分派。 | 一个 exact type descriptor 同时注册import/load/cook/upgrade能力。 |
| AST4-P1-029 | Partial | `AssetImportContext` 已能携带 source file snapshots；主source仍是完整 `Vec<u8>`，部分复杂 importer仍拥有路径/外部读能力。 | 所有输入只能经 immutable snapshot reader，读操作产生 included-file receipt。 |
| AST4-P1-030 | Open | `LibraryCacheKey` 仍以 `DefaultHasher` 哈希 source/config/importer version，缺function/target/toolchain/engine ABI。 | versioned canonical cryptographic build key。 |
| AST4-P1-031 | Open | artifact manifest有schema/kind/revision/content hash/size/chunks，但缺producer、recipe、dependency closure、target和toolchain。 | self-describing build output manifest和compatibility decision。 |
| AST4-P1-032 | Open | full、targeted、compound、watch reconciliation仍保留多条prepare/commit分支。 | 共用build graph，只改变invalidation frontier。 |
| AST4-P1-033 | Partial | watch batch已有overflow/reconciliation和age/byte diagnostics；`AssetChange`仍只有kind/uri/previous_uri。 | change加入mount/source generation、digest、sequence、cause和producer。 |
| AST4-P1-034 | Partial | reload failure保留last-good payload，resource commit可在依赖成功后发布；没有load request generation、dependency closure和consumer原子切换receipt。 | prepare new closure -> atomic publish/retain last-good -> one qualified event。 |
| AST4-P1-035 | Partial | pack已有manifest/delta/dedup/trim、staging/promotion/receipt；未与唯一cook graph、mount authority、provider和Runtime204 durable install收口。 | pack只消费qualified outputs，并发布mount/install/rollback receipt。 |
| AST4-P1-036 | Partial | worker completion、artifact chunk、Render loader、resource lease和GPU streaming各有局部预算。 | hierarchical I/O/decode/CPU/GPU/cache ledger与跨阶段priority inheritance。 |

合计：**36 = 22 Open + 13 Partial + 1 Closed**。

## 6. Runtime188 P2 currentness

| ID | 状态 | 当前说明 |
|---|---|---|
| AST4-P2-001 | Partial | 64/1,024 shard与ordered page减少部分全量复制，但重复HashMap/String authority仍存在。 |
| AST4-P2-002 | Partial | `zr_resource`已有ordered pages；`AssetRegistryIndex`和部分产品查询仍复制/排序。 |
| AST4-P2-003 | Open | import settings仍是untyped `toml::Table`，typed schema/migration telemetry未收口。 |
| AST4-P2-004 | Open | build key和readiness fingerprint仍使用`DefaultHasher`，无跨进程schema。 |
| AST4-P2-005 | Open | residency stripe只按id散列，无size/priority/deadline调度。 |
| AST4-P2-006 | Partial | chunk/worker/Render loader可报告局部bytes；CPU resource与GPU没有统一账本。 |
| AST4-P2-007 | Open | lease Drop closure持有完整manager，缺owner teardown/leak report。 |
| AST4-P2-008 | Partial | snapshot减少部分clone，常规load仍clone且没有clone-byte counter。 |
| AST4-P2-009 | Open |通用asset错误仍大量依赖文本，稳定code/action/input sequence不完整。 |
| AST4-P2-010 | Open | importer capability无health TTL/quarantine/crash attribution。 |
| AST4-P2-011 | Open | ignored microbenchmark不能代替大项目、故障、soak与cross-machine gate。 |
| AST4-P2-012 | Partial | watch已有batch age/bytes/overflow diagnostics，但缺长期profile和gap/rescan cost。 |
| AST4-P2-013 | Open | artifact 4 MiB/2 GiB等上限仍为源码常量，而非versioned target policy。 |
| AST4-P2-014 | Open | builtin residency仍线性扫描`builtin_resources()`。 |

合计：**14 = 9 Open + 5 Partial**。

## 7. 当前实现新增的收口风险

以下不是另起重复backlog，而是决定上述owner如何重构的当前证据：

| ID | 当前风险 | 归属 |
|---|---|---|
| RLC-D01 | `ResourceManager::store_payload` 执行完整commit后只返回`()`，丢失projection snapshot和event publication证据。 | AST4-P1-023 |
| RLC-D02 | `acquire`是读语义API，却取得authority写锁、改成Loaded并刷新readiness；读路径会制造publication副作用。 | AST4-P1-015/016 |
| RLC-D03 | lease Drop无法返回stale release、poison或eviction错误，失败只能静默。 | AST4-P1-012/014/018 |
| RLC-D04 | readiness dependency fingerprint用process-local `DefaultHasher`，revision/fingerprint计数多处`saturating_add`，耗尽时会静默固定。 | AST4-P1-023/025，AST4-P2-004 |
| RLC-D05 | `AssetLoadState::Loading`可由record状态投影，但通用facade没有创建/持有load ticket，状态不能回答queue、progress或cancel owner。 | AST4-P1-009/015 |
| RLC-D06 | `AssetWorkerPool::cancel(request)`终止observer/publication，但底层task closure仍可能自然执行；没有cooperative cancellation token进入decode。 | AST4-P1-009/036 |
| RLC-D07 | `AssetWorkerPool` request identity是Path/Builtin的Texture/Mesh二元枚举，与ResourceLocator/catalog/artifact体系平行。 | AST4-P1-003/009/028 |
| RLC-D08 | Render manifest loader和block loader各自复制ticket registry/deadline/close/admission状态机，尚未抽成公共request kernel。 | AST4-P1-009/036 |
| RLC-D09 | Render request key绑定resource/revision/platform，但未绑定project/mount/provider generation；project切换后的stale completion只能靠上层ticket retirement纪律。 | AST4-P1-007/014/026 |
| RLC-D10 | chunk residency预算只约束cache-owned Arc；caller持有的external lease可继续超出budget，bounded tracker甚至允许覆盖live记录并只报overflow。 | AST4-P1-013/036，AST4-P2-006 |
| RLC-D11 | Project generation token以root path+catalog sequence识别代；相同root下provider/mount/cook target变化没有独立身份。 | AST4-P1-007/026 |
| RLC-D12 | `load_imported_asset`通过`or_else`区别同kind payload，首个decoder的真实corruption可能被误报为另一schema失败。 | RAR-P0-001，AST4-P1-010 |

## 8. 参考引擎差异

| 参考 | 可核对合同 | Zircon当前差异 |
|---|---|---|
| Unreal StreamableManager | `FStreamableHandle`区分active/released/cancelled/completed，支持priority调整、cancel/release、combined handle、timeout wait和managed lifetime。 | Zircon通用asset load没有handle；这些能力只在Render专用ticket局部出现。 |
| Unreal AsyncLoading2 / AssetManager | package request、dependency、priority、async completion与primary asset policy位于统一加载系统，产品API不需要每种asset手写同步函数。 | Zircon ProjectAssetManager仍兼有project/watch/importer/residency，typed load按kind展开。 |
| Unreal DDC | build function、constants、input builds/files/hashes组成immutable build definition和serialized key。 | `LibraryCacheKey`仍是三个字段的process-local hash，artifact manifest不能解释生产来源。 |
| Bevy AssetServer | server统一source、loader、mode、meta check、load state、events和dependency tracking；handle区分strong/weak与typed/untyped身份。 | Zircon source/provider缺失，ResourceHandle只有id，load execution与registry authority分离。 |
| Fyrox ResourceManager | manager明确拥有ResourceIo、loaders、registry、task pool和watcher；pending state与waker/data绑定。 | Zircon generation/transaction更强，但通用Pending/Loading没有request/waker owner。 |
| Godot ResourceLoader | threaded request可复用token、查询progress和dependency；cache mode、UID、dependency rename是公共合同。 | Zircon没有通用threaded token/progress/cache policy；rename只改registry locator，不代表typed dependency repair。 |
| Unity Graphics | RenderGraph handle包含version/validity，registry按imported/transient/shared管理lifetime；reimport helper有batch/finally boundary。 | 可参考version handle和resource lifetime，但Unity Graphics不是通用asset authority；Zircon不能用局部Render ticket替代全局生命周期。 |

## 9. 目标架构与依赖序

```text
ResourceProviderRegistry (Runtime204)
  -> ResolvedResourceIdentity (source + mount/provider generation + exact type)
  -> ResourceLoadAuthority
       -> RequestKernel (ticket / priority / deadline / cancellation / progress)
       -> Build/Artifact Reader (qualified manifest / range / provenance)
       -> Dependency Scheduler (typed edges / SCC / retry)
       -> Payload Publication (record + payload + readiness + event receipt)
       -> Residency Ledger (artifact / CPU / GPU / physics budgets)
  -> Product Snapshot/Lease API
```

1. **M205.0 exact identity**：先关闭 RAR-P0-001；定义 exact type/schema、mount/provider generation、live instance与stale error，禁止继续扩展固定 kind + downcast探测。
2. **M205.1 request kernel**：从Render loader提取公共ticket/admission/frontier/deadline/cancel/close内核；保留现有经过测试的算法，删除两套复制状态机。
3. **M205.2 generic async load**：`ProjectAssetManager::load<T>`改为创建/合并通用request；同步API只能显式等待ticket，并有main/render thread阻塞guard。
4. **M205.3 publication**：`store_payload`返回完整receipt；record/payload/dependency/readiness/event/project generation一次发布，acquire不再修改authority状态。
5. **M205.4 dependency/reload**：typed edge、SCC、last-good、stale completion、bounded retry和dependent switch纳入同一request graph。
6. **M205.5 residency ledger**：统一artifact/CPU/GPU/physics bytes、priority和owner scope；external Arc lease也必须计费，trim有reason和receipt。
7. **M205.6 import/build/package**：source snapshot、canonical build definition、qualified artifact和pack mount/install收口，依赖Runtime204 durable provider边界。
8. **M205.7 product migration**：renderer/scene/text/UI/animation删除普通帧路径同步load和payload深clone，改用preload plan + snapshot/lease + fallback。

## 10. 资格门（24项）

| Gate | 状态 | 验收条件 |
|---|---|---|
| RLC-G01 exact type admission | Fail | wrong schema/provider payload在任何public mutation前被拒绝。 |
| RLC-G02 versioned handle | Fail | stale project/mount/payload handle永不读到新代。 |
| RLC-G03 locator resolution receipt | Fail | locator解析返回mount/provider generation和capability。 |
| RLC-G04 mutation conflict | **Partial** | live commit已拒绝kind/locator/revision conflict；还缺exact schema/provider冲突。 |
| RLC-G05 generic load ticket | Fail | 每种resource都通过同一ticket API。 |
| RLC-G06 single-flight | Fail | 通用resource按qualified identity合并；不是仅Texture/Mesh或Render block。 |
| RLC-G07 priority inheritance | Fail | dependency和合并request继承最高有效priority。 |
| RLC-G08 deadline/cancel | **Partial** | Render ticket已具备；通用resource/import/decode/upload尚未贯通。 |
| RLC-G09 cooperative cancellation | Fail | worker停止未需要的I/O/decode/upload，而非只丢completion。 |
| RLC-G10 progress/stage | Fail | queue/read/decode/dependency/upload可查询且单调。 |
| RLC-G11 generation fence | **Partial** | project commit有token；load/render completion未统一绑定provider/mount/project代。 |
| RLC-G12 atomic publication | Fail | record/payload/readiness/event sequence由一个receipt证明。 |
| RLC-G13 read purity | Fail | get/snapshot/acquire不修改authority或load state。 |
| RLC-G14 last-good reload | Fail | reload失败、成功和dependent切换有原子可见测试。 |
| RLC-G15 typed dependency | Fail | hard/soft/optional与source/runtime/editor edge分层。 |
| RLC-G16 SCC policy | Fail | cycle检测和partial-load策略可重复。 |
| RLC-G17 event resync | Fail | lagged consumer通过snapshot+cursor恢复currentness。 |
| RLC-G18 artifact provenance | Fail | manifest含producer/recipe/inputs/target/toolchain/ABI。 |
| RLC-G19 source snapshot closure | Fail | importer不能绕过snapshot读取外部文件。 |
| RLC-G20 chunk integrity/budget | **Partial** | chunk校验和cache budget已存在；external lease与全局ledger未完成。 |
| RLC-G21 CPU/GPU unified residency | Fail |所有resident bytes按owner/priority/revision计费。 |
| RLC-G22 package mount/install | Fail | pack通过provider mount并有durable install/rollback receipt。 |
| RLC-G23 product no-blocking-load | Fail | frame/render/scene/text路径无未预算I/O/decode/clone。 |
| RLC-G24 scale/fault/performance | Fail | 100k/1M catalog、并发load、reload storm、cancel、OOM/provider fault和同workload percentile门通过。 |

合计：**20 Fail / 4 Partial / 0 Pass**。

## 11. 禁止的临时修补

1. 禁止再为新asset kind增加一组`load_xxx_asset/acquire_xxx_asset`并把它称作loader扩展机制。
2. 禁止用`or_else`解析失败区分exact asset type。
3. 禁止把Render专用ticket或Texture/Mesh worker描述为通用资源加载完成。
4. 禁止让同步兼容API在主线程、render线程或frame extract中偷偷读文件/解压/反序列化。
5. 禁止以`Arc::strong_count == 1`替代budgeted eviction policy。
6. 禁止用process-local pointer、`DefaultHasher`或root path冒充跨进程/跨代identity。
7. 禁止新增另一套deadline、cancel、ticket registry；应提取并复用request kernel。
8. 禁止在Runtime204 provider/mount边界未完成前让importer/loader继续扩散裸`PathBuf/File`。
9. 禁止以ignored microbenchmark或局部单测宣称优于Unreal。

## 12. Review-only 边界

本报告只复核当前工作树并收敛 owner/status，没有修改 Runtime、Editor、tooling、Cargo、ABI或测试。实现每个里程碑前必须重新生成focused manifest和dirty-path清单；本报告中的Partial只能表示局部生产合同存在，不能表示端到端功能通过。

## 状态与产出记录

| 里程碑 | 状态 | 日期 | 证据 |
|---|---|---|---|
