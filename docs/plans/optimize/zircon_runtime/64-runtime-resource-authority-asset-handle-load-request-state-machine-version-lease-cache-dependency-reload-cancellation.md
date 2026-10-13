---
title: Runtime Resource Authority、Asset Handle、Load Request State Machine、Version Lease、Cache、Dependency、Reload、Cancellation 与 Product Integration 工程化差距
category: zircon_runtime
report_id: Runtime64
review_date: 2026-08-20
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime_interface/src/resource
  - zircon_runtime/src/core/framework/asset.rs
  - zircon_runtime/src/core/resource
  - zircon_runtime/src/asset/facade
  - zircon_runtime/src/asset/pipeline/manager/asset_manager
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager
  - zircon_runtime/src/asset/pipeline/manager/resource_sync
  - zircon_runtime/src/asset/project/manager/artifact_access.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract
  - zircon_runtime/src/graphics/scene/resources/resource_streamer
  - zircon_editor/src/ui/host/editor_asset_manager
  - zircon_editor/src/ui/retained_host/app/assets/refresh.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/scene.rs
tests:
  - zircon_runtime/src/core/resource/manager/tests
  - zircon_runtime/src/core/resource/tests.rs
  - zircon_runtime/src/asset/tests/facade
  - zircon_runtime/src/asset/tests/pipeline/manager
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime/tests.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/close_project.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/51-runtime-asset-registry-index-persistence-rebuild-incremental-query-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/54-runtime-scene-event-mirror-registration-subscription-cursor-backlog-overflow-reclaim-abi-consumer-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/StreamableManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/StreamableManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/AssetManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/AssetManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/Serialization/AsyncLoading2.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/Serialization/AsyncLoading2.cpp
  - dev/bevy/crates/bevy_asset/src/handle.rs
  - dev/bevy/crates/bevy_asset/src/server/mod.rs
  - dev/bevy/crates/bevy_asset/src/server/info.rs
  - dev/Fyrox/fyrox-resource/src/manager.rs
  - dev/Fyrox/fyrox-resource/src/state.rs
  - dev/Fyrox/fyrox-resource/src/loader.rs
  - dev/Fyrox/fyrox-resource/src/untyped.rs
  - dev/Fyrox/fyrox-resource/src/event.rs
  - dev/godot/core/io/resource_loader.h
  - dev/godot/core/io/resource_loader.cpp
  - dev/godot/core/io/resource.h
  - dev/godot/core/io/resource.cpp
  - dev/godot/core/io/resource_uid.h
  - dev/godot/core/io/resource_uid.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResources.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourceRegistry.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourcePool.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Common/ReloadAttribute.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/ResourceReloader.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 64 · Runtime Resource Authority、Asset Handle、Load Request State Machine、Version Lease、Cache、Dependency、Reload、Cancellation 与 Product Integration 工程化差距

## 1. 结论

Zircon 的资源底座已有值得保留的工程基础：resource mutation 会先preflight再提交，registry、payload、runtime slot与readiness共处同一authority，project generation能阻止过期artifact结果发布，event stream有entry/bytes/age上限与gap，reload失败能暂时保留last-good payload，旧lease也不能用旧token驱逐重注册后的payload。它不是空壳。

但当前合法公开API仍能制造不可接受的资源事实。`ResourceManager::store_payload`只校验resource id、revision和`Ready`状态，不校验payload精确类型；现有测试明确把`ShaderAsset`写入`Texture`记录并期待成功。写入后通用readiness看到“存在某个TypeId”便报告`Loaded`，typed facade却报告`NotLoaded`，而`ensure_resident`看到任意payload存在又直接返回，无法从artifact自愈。这是公开mutation成功后把同一资源拆成三种互相矛盾事实，本篇登记为P0。

第二个P0来自真实产品链。`ProjectAssetManager::load_*_asset`会在调用线程同步打开artifact、读取、解压/反序列化并深clone；render frame submission、material feature extract、camera target resolution、virtual geometry fallback和`ResourceStreamer`都直接调用该API。冷资源或reload后首次访问可以在提交帧时执行无界磁盘与CPU工作，还会串行加载material parent、shader和texture依赖。当前没有load ticket、priority、deadline、progress、cancel或“frame thread禁止等待”门，因此帧时延没有上界。

本轮登记 **2项P0、66项P1、17项P2和50项验收门禁**。目标不是再补几个`load_async`包装，而是建立`ExactAssetTypeCatalog + QualifiedAssetHandle + AssetLoadCoordinator + ResourceVersionSlot + VersionLease + CachePolicy + TypedDependencyGraph + ReloadCandidateTransaction + AssetPublicationReceipt`。Runtime04继续作为资产系统广义父报告；本篇拥有当前ResourceAuthority到产品consumer的具体纵向断裂。用户已要求暂停tooling优化，本篇没有新增脚本或tooling迁移任务。

本轮仅静态review和计划记录，没有修改production、tests、Cargo、ABI或参考源码，也没有运行Cargo、真实窗口、冷盘、100K资产、OOM/device-loss、跨项目stale handle、取消风暴或性能基准，因此不能宣称已经达到或超过Unreal。

## 2. 审查边界、规模与currentness

### 2.1 物理冻结

| 冻结组 | 文件 | 行 | bytes |
|---|---:|---:|---:|
| Public contracts and typed facade | 27 | 2,393 | 74,073 |
| Resource authority, readiness, lease and event stream | 57 | 11,559 | 391,154 |
| Project load, publication and lifecycle | 31 | 3,748 | 141,773 |
| Runtime and Editor product consumers | 79 | 16,281 | 607,986 |
| Focused external asset tests | 17 | 2,541 | 90,998 |
| 去重合计 | **205** | **35,248** | **1,257,681** |

Zircon冻结集fingerprint为SHA-256 `20e6da07298cb6d44ff069936c5e36dffffc3667e866e8334cf07c3f91a1d485`。算法与Runtime63一致：相对路径转`/`、排序去重，以`path|lowercase per-file SHA-256`逐行编码，LF连接且末尾不追加LF，再对UTF-8 payload计算SHA-256。

冻结时有3个working-tree文件已被其他工作修改：`zircon_app/src/bin/zircon_shader_pbr_viewer/scene.rs`正在从`SceneRenderer`迁移到`WgpuRenderFramework`，`project_deactivation.rs`只改测试锁型，`management.rs`只删空行。本篇按当前working copy计算fingerprint；P0证据位于未改动的ResourceAuthority/loading/render frame文件。任何实现前仍须recheck这3个文件及所有fingerprint变化。

参考集为25文件、40,525行、1,504,790 bytes，fingerprint为`7f9de19f4818ae34ebb33e8a2b4d4ea10e695bf6e954c9b55087e7108e6afa5d`：Unreal 6/25,929/947,521，Bevy 3/4,351/172,439，Fyrox 5/4,130/153,110，Godot 6/3,779/129,649，Unity Graphics 5/2,336/102,071。

### 2.2 本轮拥有与明确不拥有

- Runtime64拥有resource live authority、exact payload admission、typed handle实例语义、load request状态机、CPU payload version lease/cache、依赖就绪、reload candidate与Runtime/Editor产品接线。
- Runtime04继续拥有asset identity/artifact/import/load/reload/pack的广义父合同；本篇只扩展为当前源码纵向证据，并单独拥有两个可由当前公开/产品路径触发的P0。
- Runtime09D继续拥有GPU residency、upload、streaming和GPU retirement；本篇P0只覆盖CPU artifact I/O/decode/clone在frame-critical路径执行。
- Runtime24继续拥有通用ID/generation/owner epoch/耗尽；本篇只规定asset handle、version lease和project replacement如何采用这些合同。
- Runtime51继续拥有asset registry/index/persistence/rebuild/query；本篇拥有published registry row与live exact payload/load state的一致性。
- Runtime54继续拥有跨进程event mirror、cursor/backlog/resync；本篇只拥有本地asset publication事件的exact type/request/version语义，不重复其gap P0。
- Runtime59继续拥有通用task scheduler、cancellation和shutdown；本篇拥有asset load request如何声明owner、priority、deadline、cancel和terminal receipt。
- Plugins07拥有具体importer/source/subasset/artifact生成；Editor04拥有import/catalog/thumbnail/reference authoring UX。本篇不重复格式支持或Editor控件，只要求它们消费同一个Runtime load/version事实。
- 用户要求暂停tooling优化；本篇不新增Python、PowerShell、生成器或tooling性能工作。

## 3. 当前真实链路

```text
typed Handle<T>
  = ResourceId + PhantomData
  -> marker only contributes broad ResourceKind

load_*_asset(id)
  -> ensure_resident(id)
       -> lock one of 64 residency stripes
       -> any untyped payload exists? return Ok
       -> registry Ready?
       -> prepare project artifact read
       -> synchronous read/decode full ImportedAsset
       -> project generation recheck
       -> store_payload(id, revision, dyn ResourceData)
  -> downcast by requested Rust T
  -> deep clone T

render frame/product access
  -> material/model/texture/shader load_* calls
  -> cold artifact I/O and dependency chain may run inline

lease lifecycle
  -> acquire increments private slot ref_count
  -> get returns untracked Arc; load returns detached clone
  -> last tracked lease drop removes authority payload
  -> untracked Arc/clone may still outlive authority version

project close/open
  -> remove all project records/payload/runtime slots
  -> outstanding handles contain no project generation
  -> same locator derives same ResourceId in next generation
  -> stale handle can resolve a replacement asset silently
```

## 4. 当前应保留的能力

1. `ResourceMutationBatch`的preflight/apply边界、revision conflict与explicit rename检查是正确事务入口。
2. registry、payload、runtime与readiness由同一`ResourceAuthority`拥有，避免了完全独立manager间的任意撕裂。
3. project generation二次确认能拒绝在项目切换期间完成的旧artifact读取结果。
4. reload失败保留last-good payload和旧lease token隔离表明version slot方向已有局部基础。
5. readiness reverse dependency closure与immutable generation row可作为发布后查询投影继续演进。
6. event stream的entry/bytes/age预算、gap和diagnostics应保留，但需绑定可恢复publication generation。
7. `ResourceLease`把释放动作封装进Drop，适合作为未来strong version lease的迁移起点。
8. artifact读取先prepare再read，适合扩展为typed descriptor、section request与cancelable I/O。
9. Editor已能在resource gap时reconcile，而不是永久相信增量事件；该恢复意识应接到统一asset snapshot。
10. focused tests已覆盖revision conflict、旧lease不驱逐新payload、failed reload last-good和project close ordering；这些应升级而非删除。

## 5. 参考引擎事实与Zircon差异

| 参考 | 代码事实 | Zircon应吸收的合同 |
|---|---|---|
| Unreal Streamable | `FStreamableHandle`可改priority、release、cancel、start stalled、绑定complete/cancel/update、查询count/progress/owning manager并组合handle；同步load接口明确警告会让game thread停顿数秒。 | canonical请求必须有owner、priority、progress、cancel、terminal callback/receipt；同步等待必须是显式受限wrapper，不能藏在普通`load_*`。 |
| Unreal AsyncLoading2/AssetManager | request ID映射package，priority可递归更新，event graph组织import依赖，I/O batch与cancel集合分离；AssetManager按primary asset/bundle管理active handle。 | 以请求和typed dependency graph驱动I/O/decode/publication，区分身份、请求、依赖和驻留owner。 |
| Bevy Asset | strong handle用`Arc<StrongHandle>`保持资产存活并携exact `TypeId`；`AssetServer::load`非阻塞、复用既有path handle、在IoTaskPool运行，外部handle提前drop可取消pending task。 | strong/weak/soft语义、exact type admission、single-flight pending task、真实load state与handle lifetime相连。 |
| Bevy load info | info保存pending tasks、waiting wakers、loading/failed dependency集合；错误和dependency state分开传播。 | `Loading`必须对应真实请求/等待者，dependency failure不能只压缩成一个枚举或任意payload存在。 |
| Fyrox Resource | `ResourceState::Pending`保存waker，manager在TaskPool spawn loader future，loader按data type UUID选择，完成后commit并广播Loaded/Reloaded。 | exact stable type、future/waker、异步loader与资源状态必须是一条链，而非投影状态和同步read分家。 |
| Godot ResourceLoader | threaded request复用path token，worker task有status/progress/condition variable，dependency progress递归聚合；CacheMode区分ignore/reuse/replace及deep policy。 | 明确request token、dedup、worker状态、progress与cache policy；cache存在不等于type/version正确。 |
| Unity Graphics RenderGraph | `ResourceHandle`同时编码index、version、resource type和execution validity；write产生新version，跨frame陈旧handle失效，共享persistent与transient资源分别管理。 | stable asset identity与live instance/version key分离；handle必须能拒绝跨project/frame/generation陈旧引用。 |
| Unity Graphics ResourceReloader | declarativereload field保留field exact type/path/package，invalid import与AssetDatabase暂不可用被区分并可延迟重试。 | reload需要typed candidate、retriable failure和显式publish，而不是任意payload覆盖或字符串Initialization错误。 |

## 6. P0：当前合法路径可破坏资源正确性或帧时延上界

### RAR-P0-001：公开payload mutation允许错误精确类型并形成不可自愈三重事实

`payload_ops.rs:11-40`公开`register_ready<TData>`与`store_payload<TData>`，泛型只要求`ResourceData`。`commit.rs:182-207`对`StorePayload`只检查record存在、revision相等、state为`Ready`，随后直接`PayloadMutation::Replace`；没有`AssetTypeId/SchemaId`，也没有`ResourceKind -> concrete payload`验证。

现有`asset/tests/facade/load_state_roots.rs:56-79`明确执行：先注册`TextureAsset`，再以相同id/revision存入`ShaderAsset`，`store_payload(...).unwrap()`成功。之后：

1. `readiness_projection.rs:271-290`看到record Ready且`payload_type_id.is_some()`，通用投影报告`Loaded`；
2. `Assets<TextureAsset>::get/load_state`downcast失败，typed facade报告`NotLoaded`；
3. `ensure_resident.rs:11-15`只检查`get_untyped(id).is_some()`便返回`Ok`，拒绝重新读取正确artifact；
4. `StorePayload`不改变record，`event_for_staged_resource`在`before == after`时返回`None`，consumer也收不到payload版本变化；
5. runtime slot仍是`Loaded/ref_count=0`，diagnostics和registry row均无法解释错误payload来自哪次操作。

这不是“错误类型最终返回None”的安全退化，而是一次成功公开调用永久破坏同一authority内registry、runtime state、typed facade和rehydration路径的一致性。必须先建立stable exact type/schema catalog，让record、handle、artifact descriptor、mutation operation和payload codec共享同一`ExactAssetTypeId`；所有ready/store/publish在staging阶段验证type/schema/revision/provider，失败零变更。错误payload测试必须改为期待typed admission error，并证明旧payload、runtime state、readiness generation和event sequence逐字节不变。

### RAR-P0-002：frame submission和ResourceStreamer可内联执行冷盘全量加载与深拷贝

`ensure_resident.rs:11-101`在residency stripe mutex内调用prepared artifact `.read()`；`load_typed.rs:9-26`随后把`Arc<T>`深clone为按值返回。代码中没有load request、future、priority、deadline、progress、cancel或task scheduling。

这条同步链已进入真实帧路径：

- `build_frame_submission_context/build.rs:75-110`在构建submission context时解析material、environment和camera target；`554-573`的automatic virtual geometry闭包直接`load_model_asset`；
- `material_feature_extract.rs:45-80`在frame extract中同步加载root及最多4层parent material；
- `target_resolution.rs:83-102`为camera texture target同步加载texture；
- `resource_streamer_ensure_material.rs:39-93,553-577,629-678,680-731`同步加载material/fallback/parent/shader/texture依赖；
- `resource_streamer_ensure_texture.rs:12-43`在GPU resource构建前同步加载完整texture；model cache命中仍clone整个`ModelAsset`。

因此任意合法的冷resource、last lease eviction、reload或project首次帧都可能在render submission路径执行文件读取、解压、反序列化、多层依赖探测和大对象clone；64个stripe只能避免同id重复，并不提供时限。Unreal甚至在同步API注释中明确“可能卡game thread数秒”，而Zircon把同类工作藏在普通`load_*`返回值中。

必须建立`AssetLoadCoordinator`，冷load只返回`LoadTicket/VersionFuture`并由I/O/CPU lane执行；frame path只允许消费ready immutable `AssetVersionSnapshot`或显式fallback，结构测试禁止调用同步wait/read/decode。preload/readiness barrier要在scene activation、viewport admission或streaming budget阶段完成。只有冷盘、dependency fan-out、reload、cancel和pressure基准证明frame p99/p99.9无同步I/O/解码后，才能关闭此P0。

## 7. P1工程化差距

### 7.1 Exact type、schema与authority admission

| ID | 当前差距 | 所需收敛 |
|---|---|---|
| RAR-P1-001 | `ResourceRecord`只有宽泛`ResourceKind`，没有exact asset type、schema、codec或provider identity。 | 引入稳定`ExactAssetTypeId/SchemaId/CodecId/ProviderGeneration`并贯穿record、artifact、handle和receipt。 |
| RAR-P1-002 | `ResourceMarker`只有`const KIND`，不能证明Rust payload或插件schema。 | marker绑定stable exact type registration，运行时从catalog验证而非只看大类。 |
| RAR-P1-003 | Texture/UI icon、UI layout/v2 view、widget/v2 component、style/theme/v2 style共享marker。 | 每个可序列化payload有唯一exact type；共同kind只用于查询分组。 |
| RAR-P1-004 | `load_imported_asset`用`.or_else`依次尝试alias类型，把错误当类型发现控制流。 | artifact header/record在decode前直接选择唯一codec；禁止variant probing。 |
| RAR-P1-005 | `load_asset.rs`与`acquire_asset.rs`手写数十个映射，新增type易漏load/acquire/event三处。 | catalog生成/注册统一typed operation table，API按registration路由。 |
| RAR-P1-006 | `Handle<T>`和`ResourceHandle<T>`序列化后只剩`ResourceId`。 | soft handle持stable asset id + exact type；live handle另持owner/slot generation。 |
| RAR-P1-007 | `UntypedResourceHandle::typed()`只比较kind，alias或错误payload会提前宣称成功。 | typed conversion验证exact type/schema和owner policy，未知/旧schema返回typed error。 |
| RAR-P1-008 | `ResourceRecord`没有project/session/tenant owner generation。 | project-owned row携qualified owner，跨project引用必须显式external/global。 |
| RAR-P1-009 | `ResourceData: Any`与`get_untyped`允许绕过codec/catalog读取任意payload。 | erased payload只由authority内部持有，外部通过validated snapshot/type-erased descriptor访问。 |
| RAR-P1-010 | `ResourceSnapshot<T>`只有record + Arc，未绑定slot generation、schema或publication receipt。 | 统一为`AssetVersionSnapshot`，携exact type、content revision、slot generation与version lease。 |

### 7.2 Load request、状态机与并发

| ID | 当前差距 | 所需收敛 |
|---|---|---|
| RAR-P1-011 | canonical API没有`LoadRequestId/Ticket/Future`。 | 所有冷load返回共享ticket，可等待、订阅、取消和查询terminal receipt。 |
| RAR-P1-012 | `RuntimeResourceState::Loading`存在，但production residency从未在read前进入该状态。 | request admission原子发布Queued/Resolving/Reading/Decoding/WaitingDependencies/Publishing。 |
| RAR-P1-013 | `ResourceState::Pending`直接投影为Loading，即使没有实际task。 | state必须引用live request或明确catalog pending cause；无request不得伪报加载中。 |
| RAR-P1-014 | `ensure_resident`同步完成完整artifact read/decode。 | I/O、decode、validation和publication分lane，调用线程只做bounded admission。 |
| RAR-P1-015 | residency stripe在磁盘读取期间一直持锁，同stripe无关id也被阻塞。 | 锁内只安装/查找single-flight entry；慢工作锁外执行，publish做generation CAS。 |
| RAR-P1-016 | 固定64 stripe用`DefaultHasher`分配，缺少occupancy、wait和hot-key观测。 | 可配置shard policy并记录queue/wait/hold，按证据扩展而非固定常量。 |
| RAR-P1-017 | mutex只防重复执行，没有waiter fan-out、shared result或request identity。 | 以`(qualified id, exact type, revision, policy)`single-flight并共享terminal result。 |
| RAR-P1-018 | 没有priority、deadline、progress、stage、start-stalled或reprioritize。 | ticket暴露这些合同，并映射到Runtime59 scheduler与I/O backend。 |
| RAR-P1-019 | 没有request owner/cancel-on-project-close/module-shutdown。 | owner lease退出时取消未发布请求，drain callback后再退休provider/project。 |
| RAR-P1-020 | load错误全部包装成`CoreError::Initialization(name, String)`。 | 区分Missing/Type/Schema/Io/Corrupt/Dependency/Cancelled/Superseded/Timeout/Budget/Provider。 |
| RAR-P1-021 | 失败没有retry classification、backoff或negative cache，同一帧可反复失败。 | terminal receipt标记retriable和retry-after，cache按input generation失效。 |
| RAR-P1-022 | 没有各stage bytes/items/time、queue wait、dedup waiter与wasted work统计。 | 每个request输出结构化stage timeline和预算/取消/废弃原因。 |

### 7.3 Handle、version lease与cache policy

| ID | 当前差距 | 所需收敛 |
|---|---|---|
| RAR-P1-023 | `ResourceManager::get`/`Assets::get`返回Arc但不增加slot ref_count。 | 所有长期payload访问都持version lease；裸Arc不越过authority边界。 |
| RAR-P1-024 | tracked lease drop可删除authority payload，而未跟踪Arc仍存活，随后可加载第二份同revision对象。 | slot维护active/retired version census，旧Arc只能由对应lease拥有。 |
| RAR-P1-025 | 最后一个lease释放就立即卸载，不看cache budget、reuse、pin或cost。 | residency由policy/budget/LRU/priority决定；refcount为0只是可驱逐，不是强制驱逐。 |
| RAR-P1-026 | handle没有strong/weak/soft区别，Copy handle看起来像live引用却不保活。 | 明确SoftAssetRef、WeakVersionHandle、StrongVersionLease三类语义。 |
| RAR-P1-027 | locator派生稳定ResourceId；close/reopen或remove/re-register后旧Copy handle可别名新实例。 | stable asset id与`OwnerEpoch + SlotGeneration`分离，live resolve拒绝stale generation。 |
| RAR-P1-028 | `ResourceLease`只有id、u64 token和Arc，没有revision/type/project/provider。 | lease携qualified instance key、exact type、content revision和owner epoch。 |
| RAR-P1-029 | residency token用`wrapping_add().max(1)`，最终可ABA复用。 | 采用checked generation/exhaustion合同，禁止静默wrap。 |
| RAR-P1-030 | `slot.ref_count += 1`无checked overflow。 | checked acquire；耗尽返回typed admission failure且状态不变。 |
| RAR-P1-031 | payload Replace直接插入新slot并把ref_count重置0，忽略旧lease census。 | active/candidate/retired版本分槽，publish不覆盖旧版本的live ownership。 |
| RAR-P1-032 | 没有retired generation、quiescence或版本上限，reload期间生命周期不可审计。 | 每slot限制retired versions/bytes并在lease清零后回收。 |
| RAR-P1-033 | CPU payload没有estimated/actual bytes、entry budget、LRU、pin或pressure policy。 | cache admission/eviction按bytes、cost、priority、world和frame budget执行。 |
| RAR-P1-034 | `Asset: Clone`与`load_typed<T: Clone>`把深clone变成基础合同。 | 默认返回`AssetVersionSnapshot<T>`；只对确需复制的小value提供显式clone。 |
| RAR-P1-035 | `ResourceCacheIdentity`只有revision/state，忽略exact type、slot generation、artifact/provider。 | cache key绑定qualified asset version、schema、provider和relevant dependency generation。 |

### 7.4 Dependency、reload与publication

| ID | 当前差距 | 所需收敛 |
|---|---|---|
| RAR-P1-036 | dependency只有裸`ResourceId`数组。 | edge携required/optional、hard/soft、exact type/schema、version和provenance。 |
| RAR-P1-037 | readiness遇到cycle返回direct/recursive Loaded。 | registry candidate发布前做SCC；非法required cycle失败，允许环需显式edge policy。 |
| RAR-P1-038 | dependency source有任意payload TypeId就视为Loaded，不验证期望类型。 | readiness消费validated exact version，TypeMismatch为结构化失败。 |
| RAR-P1-039 | dependency readiness不是实际request graph，不能驱动调度、取消或priority inheritance。 | coordinator构建有界DAG/SCC plan并让parent ticket聚合child receipts。 |
| RAR-P1-040 | 缺失、失败、optional unavailable和version mismatch被压成少数枚举。 | report保留edge/path、cause chain、retry disposition与fallback policy。 |
| RAR-P1-041 | production reload未形成active/candidate双版本事务。 | source invalidation只启动candidate；验证成功后单次publish，失败保留active。 |
| RAR-P1-042 | failed reload把record/runtime设Error，但`get/acquire`仍可返回last-good，状态语义矛盾。 | active version readiness与candidate failure分字段发布。 |
| RAR-P1-043 | last-good仅因`Reloading/Error`特殊分支保留，不是正式version policy。 | candidate transaction显式引用active fallback与retirement条件。 |
| RAR-P1-044 | payload replacement若record不变不产生event。 | publication event由version publish驱动，携before/after version与operation receipt。 |
| RAR-P1-045 | typed asset event只按broad kind过滤，alias类型会收到彼此事件。 | event携exact type/schema；typed receiver严格匹配registration。 |
| RAR-P1-046 | event没有request、owner、candidate/active、failure、cache或terminal disposition。 | 定义typed lifecycle event/receipt，观察者不再猜record revision。 |
| RAR-P1-047 | Added/Updated coalesce会移除旧sequence并让已订阅consumer立即Lagged。 | Runtime54 owner统一publication cursor/resync；coalesce保留可解释range/epoch。 |
| RAR-P1-048 | global commit serial一直持有到所有event publication完成。 | immutable candidate构建锁外，锁内swap，event journal在同receipt后有界发布。 |
| RAR-P1-049 | project resource prepare先持global commit serial，再执行任意filesystem/project closure。 | 两阶段transaction用reservation/CAS而非跨慢I/O持全局serial。 |

### 7.5 Runtime、Renderer、Editor与project lifecycle产品接线

| ID | 当前差距 | 所需收敛 |
|---|---|---|
| RAR-P1-050 | neutral `core::framework::asset::ResourceManager`只提供metadata/revision/subscribe。 | 增加模块中立的typed resolve/request/snapshot contract，不泄漏具体manager。 |
| RAR-P1-051 | `AssetManager` trait只覆盖project/import/catalog/watch，不含runtime load/acquire。 | 区分AuthoringAssetService与RuntimeAssetService，声明owner和lifecycle。 |
| RAR-P1-052 | graphics/text/scene直接解析并调用具体`ProjectAssetManager`。 | 产品依赖中立`RuntimeAssetService`handle，支持provider replacement与capability。 |
| RAR-P1-053 | material链多处重复同步加载fallback/parent/shader/texture并深clone。 | scene activation产生dependency load plan；frame只消费同代ready snapshots。 |
| RAR-P1-054 | model cache命中仍clone完整`ModelAsset`，cache只比source revision。 | cache保存version snapshot/lease并包含dependency/cook identity。 |
| RAR-P1-055 | camera target、IBL、material feature和virtual geometry在submission构建期间解析资产。 | 提前编译frame resource set；missing/cold只选择显式fallback或skip receipt。 |
| RAR-P1-056 | scene/project/viewport没有preload barrier或required asset admission receipt。 | activation定义required/optional set、budget、deadline与ready/fallback结果。 |
| RAR-P1-057 | Editor从runtime project clone出独立`EditorAssetState/catalog`，靠事件后手动refresh。 | Editor消费同代immutable catalog + load/version projection，不复制authority。 |
| RAR-P1-058 | Editor API不暴露load request、progress、cancel、version lease或candidate failure。 | details/preview引用Runtime ticket/snapshot并显示真实terminal state。 |
| RAR-P1-059 | close_project删除authority row，但outstanding handle无project generation；下个项目可复用id。 | close退休owner epoch、取消request、保留retired lease并让旧handle永久stale。 |

### 7.6 测试、错误与可运维性

| ID | 当前差距 | 所需收敛 |
|---|---|---|
| RAR-P1-060 | wrong-payload测试把成功写入和typed NotLoaded当预期。 | 改为mutation原子拒绝，并验证所有generation/事件/payload不变。 |
| RAR-P1-061 | 没有same-locator close/reopen、remove/re-register的stale handle/lease产品测试。 | 覆盖owner epoch、slot generation、retired lease和永久stale handle。 |
| RAR-P1-062 | 没有frame path“零文件I/O/零decode/零深clone”结构与运行门。 | instrumentation fail test + cold/warm frame benchmark共同验收。 |
| RAR-P1-063 | 没有priority inversion、cancel race、deadline、waiter drop和shutdown drain测试。 | deterministic scheduler/fake I/O覆盖每个request transition与terminal exactly-once。 |
| RAR-P1-064 | 没有CPU payload bytes、eviction、pressure、OOM和retired version规模门。 | 1K/100K资产、large texture/model、reload storm与budget fault benchmark。 |
| RAR-P1-065 | load failure被字符串化，测试无法断言cause/disposition/provenance。 | typed error DTO、cause chain和stable error code进入unit/integration/product测试。 |
| RAR-P1-066 | 没有read/decode/dependency/publish各阶段fault injection与零半提交证明。 | 每一stage注入失败/取消/supersede，验证active version、cache和receipt一致。 |

## 8. P2完善项

| ID | 当前不足 | 完善方向 |
|---|---|---|
| RAR-P2-001 | `ResourceAuthority`单RwLock的hold/wait分布未知。 | 建立100K/1M row与并发reader/acquire/reload benchmark后再决定分片。 |
| RAR-P2-002 | registry staging会clone整张HashMap，规模成本未量化。 | candidate immutable generation/structural sharing，并记录copy/allocation。 |
| RAR-P2-003 | shard选择依赖`DefaultHasher`，跨进程不稳定且难重放。 | 若shard成为artifact/diagnostic identity，改用固定版本stable hash。 |
| RAR-P2-004 | readiness/event generation多处`wrapping_add`。 | 统一checked exhaustion/epoch rollover和consumer resync。 |
| RAR-P2-005 | poison lock恢复后继续执行，未发布poison diagnostics。 | 记录fault domain并根据authority完整性决定fail-stop或rebuild。 |
| RAR-P2-006 | event bytes为近似值，未计Vec/String capacity与共享owner。 | 定义可重复的retained byte accounting与budget误差界。 |
| RAR-P2-007 | event 4,096/4MiB/60秒为固定全局策略。 | 按consumer class配置并绑定resync cost与backlog SLO。 |
| RAR-P2-008 | `load`命名隐藏同步I/O与clone成本。 | 过渡期改名`load_blocking_cloned`并限制调用线程，最终删除。 |
| RAR-P2-009 | `ResourceData`强制`Debug`，大型payload debug能力与运行权威耦合。 | diagnostics由typed descriptor提供bounded summary。 |
| RAR-P2-010 | `ResourceRuntimeInfo`重复record/runtime字段但没有统一snapshot generation。 | 由authority publication生成不可变同代view。 |
| RAR-P2-011 | `ResourceLease`不可clone且没有downgrade/upgrade，调用者易退回裸Arc。 | 在明确strong count语义后提供clone/downgrade/try-upgrade。 |
| RAR-P2-012 | alias probing顺序是隐式兼容策略，新增类型可能改变结果。 | exact type迁移表显式版本化，删除顺序依赖。 |
| RAR-P2-013 | dependency fingerprint基于DefaultHasher且cycle hash与健康聚合混用。 | published graph使用稳定canonical fingerprint和独立cycle diagnostic。 |
| RAR-P2-014 | readiness row同时混合record、payload TypeId与递归状态，snapshot较重。 | 分离identity/version row与派生aggregate，按consumer投影。 |
| RAR-P2-015 | `contains`只看kind，产品可能把错误exact type视为存在。 | 提供exact contains/resolve report，broad kind查询仅用于catalog筛选。 |
| RAR-P2-016 | load/acquire方法列表未从catalog生成一致性测试。 | registration inventory验证每type codec/load/acquire/event/migration齐全。 |
| RAR-P2-017 | 当前没有统一资源状态可视化，Editor和profiler各自投影有限字段。 | 输出bounded request/version/cache/dependency diagnostic snapshot。 |

## 9. 目标架构与硬切边界

### 9.1 Authority与identity

`ResourceRegistryGeneration`保存稳定asset identity、exact type/schema、owner/project epoch、typed dependency graph和artifact descriptor；`ResourceVersionSlot`保存active/candidate/retired payload version。soft identity不保活，live handle必须包含slot generation，跨project或replacement后fail-stale。

### 9.2 Request与publication

`AssetLoadCoordinator`按qualified asset version single-flight。ticket状态至少为Queued、Resolving、Reading、Decoding、WaitingDependencies、Validating、Publishing、Ready、Failed、Cancelled、Superseded；每个terminal只有一次receipt。慢工作锁外执行，publish在authority内做generation/type/schema/revision CAS并一次交换active version。

### 9.3 Lifetime与cache

`VersionLease<T>`是payload长期访问的唯一owner；active版本可有零lease但仍由cache policy保留，retired版本由旧lease保活。compressed section、decoded CPU payload与GPU residency使用不同预算和owner，Runtime09D只消费已发布CPU version及upload descriptor。

### 9.4 Dependency与reload

candidate registry先做typed edge验证和SCC，再生成load plan。reload构建candidate version，失败只发布candidate failure，active继续服务；成功发布before/after version receipt，旧lease稳定观察旧版本。event只是receipt journal的投影，不再反推authority事实。

### 9.5 Product integration

scene/project/viewport activation提交required/optional asset set和预算，获得admission receipt；render frame只读取ready snapshot或显式fallback。Editor details/preview消费同一个ticket/version/catalog generation。project close取消owner request、退休owner epoch并等待受管callback，不能让stale handle落到下一项目。

硬切要求：

1. exact type catalog落地后删除只按`ResourceKind`成功的typed conversion、alias probing和public arbitrary `store_payload`。
2. product迁移后删除按值clone的`load_*_asset/load_typed`；frame path禁止同步wait/read/decode。
3. 裸Arc不再越过authority；所有长期访问必须持version lease。
4. 删除last-lease立即卸载规则，改由显式cache policy与budget驱逐。
5. live handle加入owner epoch/slot generation后，不保留把stale handle静默解析到新slot的shim。
6. dependency cycle不再返回Loaded；非法candidate graph不得发布。
7. reload不再用record Error同时表示“active可用、candidate失败”；拆成两套事实。
8. project/filesystem慢commit不得持global resource commit serial。
9. Editor和renderer停止依赖具体`ProjectAssetManager`，改用模块中立service contract。

## 10. 依赖序重构里程碑

| 里程碑 | 先写失败测试 | 实现范围 | 退出条件 |
|---|---|---|---|
| M64.1 Exact type admission | wrong payload/register/store/type alias negative matrix | ExactAssetTypeCatalog、typed record/artifact/mutation | RAR-P0-001关闭，失败零变更 |
| M64.2 Qualified handle | close/reopen、remove/re-register stale handles | owner epoch、slot generation、soft/weak/strong handle | stale引用永久拒绝且无ABA |
| M64.3 Version slot/lease | raw Arc、old/new lease、replacement、overflow | active/candidate/retired slot、checked counts | payload lifetime与accounting一致 |
| M64.4 Load coordinator | dedup/waiter/cancel/priority/deadline/fault tests | ticket state machine、I/O/CPU lanes、terminal receipt | 所有state对应真实request |
| M64.5 Typed dependency | cycle/type/version/optional/fan-out corpus | typed edges、SCC、request plan、cause chain | 非法graph不发布 |
| M64.6 Cache policy | byte pressure、LRU/pin、retired version/OOM | CPU cache admission/eviction与diagnostics | budget内稳定且无live version驱逐 |
| M64.7 Reload transaction | success/fail/cancel/supersede with live leases | candidate build、last-good、atomic publication | active与candidate状态不矛盾 |
| M64.8 Product migration | render cold path instrumentation | renderer/scene/text改snapshot/fallback/preload | RAR-P0-002关闭 |
| M64.9 Editor/project lifecycle | project switch、preview cancel、gap/resync | neutral service、Editor projection、owner retirement | 无第二authority和跨项目别名 |
| M64.10 Scale/soak | 100K assets、reload/cancel storm、cold/warm frames | metrics、budgets、fault/soak | 通过全部50项门禁 |

## 11. 验收门禁

| Gate | 验收内容 |
|---|---|
| RAR-G01 | 每个published record携stable exact type/schema/provider和owner epoch |
| RAR-G02 | public ready/store对wrong payload在preflight拒绝且authority逐字节不变 |
| RAR-G03 | alias asset各有唯一exact type，不再依靠marker或probe order |
| RAR-G04 | typed handle转换在payload decode前验证exact type/schema |
| RAR-G05 | same locator close/reopen后旧live handle永久Stale |
| RAR-G06 | remove/re-register、provider reload和generation exhaustion无ABA |
| RAR-G07 | soft/weak/strong语义和serialization roundtrip明确且测试覆盖 |
| RAR-G08 | 所有payload长期访问持version lease，authority外无裸Arc逃逸 |
| RAR-G09 | replacement不重置旧version lease census |
| RAR-G10 | refcount/token/generation耗尽返回typed failure且零变更 |
| RAR-G11 | last lease释放只使version可驱逐，不强制卸载 |
| RAR-G12 | cache按bytes/items/cost/priority/owner有界admission和eviction |
| RAR-G13 | active/candidate/retired版本及bytes可查询并有上限 |
| RAR-G14 | coldload只创建ticket，不在调用线程读盘/decode/clone |
| RAR-G15 | ticket有stable request id、owner、priority、deadline和debug source |
| RAR-G16 | request状态对应真实task/queue/waiter，不存在伪Loading |
| RAR-G17 | single-flight按qualified version去重并共享terminal result |
| RAR-G18 | waiter drop、owner close和explicit cancel exactly-once终止 |
| RAR-G19 | reprioritize与dependency priority inheritance有确定性测试 |
| RAR-G20 | 每stage progress、bytes、time、queue wait和cancel cause可观测 |
| RAR-G21 | missing/type/schema/I/O/corrupt/dependency/budget/cancel错误可区分 |
| RAR-G22 | retry/backoff/negative cache按input generation正确失效 |
| RAR-G23 | I/O、decode、validation、publication全部有fault injection |
| RAR-G24 | superseded project/artifact request不发布且不污染cache |
| RAR-G25 | dependency edge携kind、exact type、required policy和provenance |
| RAR-G26 | required cycle在candidate registry publication前失败 |
| RAR-G27 | optional unavailable不会伪装required Loaded或吞cause |
| RAR-G28 | parent ticket聚合child progress/failure并可取消未需要分支 |
| RAR-G29 | reload失败保留active version，candidate failure独立发布 |
| RAR-G30 | reload成功一次交换active，旧lease稳定读取旧version |
| RAR-G31 | payload-only publish产生before/after version event/receipt |
| RAR-G32 | typed event receiver不会收到同kind不同exact type事件 |
| RAR-G33 | event gap可按publication generation完整resync |
| RAR-G34 | event/request receipt含owner、request、version、terminal disposition |
| RAR-G35 | resource global serial不跨filesystem、decode或callback持有 |
| RAR-G36 | authority lock内只做有界CAS/swap，hold/wait进入profile |
| RAR-G37 | scene/project/viewport activation有required/optional preload receipt |
| RAR-G38 | render submission结构测试证明无blocking load/wait API |
| RAR-G39 | cold frame instrumentation证明零文件I/O、零decode、零深clone |
| RAR-G40 | missing/cold frame只走显式fallback/skip并记录receipt |
| RAR-G41 | material parent/shader/texture依赖在frame前完成plan/admission |
| RAR-G42 | CPU snapshot与Runtime09D GPU upload version一致 |
| RAR-G43 | Editor details/preview消费Runtime ticket/version而非复制load state |
| RAR-G44 | project close取消请求、退休epoch、drain callback并保留retired lease |
| RAR-G45 | provider/module shutdown后无late publish或callback进入已卸载代码 |
| RAR-G46 | 100K资产与高fan-out reload的commit/read p95/p99有门限 |
| RAR-G47 | large texture/model、reload storm和OOM下CPU cache不越预算 |
| RAR-G48 | cancel/deadline/priority storm下queue有界且低优先级不永久饥饿 |
| RAR-G49 | cold/warm frame p50/p95/p99/p99.9、I/O、CPU、RSS按同workload记录 |
| RAR-G50 | focused tests、product tests、fault/soak、Markdown/link/index验证全部通过 |

## 12. 状态与产出记录

| 项目 | 状态 | 日期 | 证据 |
|---|---|---|---|
| Runtime64 static source review | review_complete | 2026-08-20 | 205文件、35,248行、1,257,681 bytes，fingerprint `20e6da...d485` |
| Reference comparison | review_complete | 2026-08-20 | 25文件、40,525行、1,504,790 bytes，五套参考源码 |
| Severity/acceptance inventory | review_complete | 2026-08-20 | 2 P0 / 66 P1 / 17 P2 / 50 gates |
| Production/tests/Cargo mutation | pending | - | 本篇只review；MVP M0.3未绿前不实施advanced refactor |

后续实现必须从M64.1开始，先关闭公开wrong-payload半事实，再建立qualified handle/version lease和真实load request，最后迁移frame product路径。禁止先加异步包装却保留任意payload admission、裸Arc、按值clone和同步frame fallback，否则只会把同一缺陷藏到更多线程。
