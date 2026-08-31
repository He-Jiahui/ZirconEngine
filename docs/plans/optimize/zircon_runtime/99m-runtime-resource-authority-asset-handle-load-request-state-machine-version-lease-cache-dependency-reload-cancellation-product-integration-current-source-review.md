---
title: Runtime Resource Authority、Asset Handle、Load Request State Machine、Version Lease、Cache、Dependency、Reload、Cancellation 与 Product Integration 当前源码工程化差距复核
category: zircon_runtime
report_id: Runtime112
review_date: 2026-08-22
baseline_head: bee4c707b714738346b49bba15c59468b8bd9b39
baseline_epoch: 339
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
  - zircon_runtime/src/graphics/scene/resources/resource_streamer
  - zircon_editor/src/ui/host/editor_asset_manager
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/51-runtime-asset-registry-index-persistence-rebuild-incremental-query-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/54-runtime-scene-event-mirror-registration-subscription-cursor-backlog-overflow-reclaim-abi-consumer-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/64/2026-08-21-copy-on-write-registry-staging.md
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/87-runtime-asset-reference-identity-locator-guid-subasset-redirector-rename-move-resolution-repair-migration-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/88-runtime-asset-watch-change-ingress-coalescing-rename-overflow-targeted-reimport-generation-reload-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/92-runtime-texture-image-cubemap-array-volume-format-sampler-mip-compression-upload-streaming-residency-budget-eviction-virtual-texture-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/93-runtime-mesh-geometry-section-lod-instancing-skinning-morph-deformation-bounds-collision-streaming-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/57-editor-asset-workspace-content-browser-folder-source-tree-selection-open-create-import-rename-move-delete-history-collection-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/StreamableManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/StreamableManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/AssetManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/AssetManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/Serialization/AsyncLoading2.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/Serialization/AsyncLoading2.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Tests/Loading/AsyncLoadingTests_Timeouts.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Tests/Loading/AsyncLoadingTests_RecursiveLoads.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Tests/Loading/AsyncLoadingTests_Flushes.cpp
  - dev/bevy/crates/bevy_asset/src/handle.rs
  - dev/bevy/crates/bevy_asset/src/server/mod.rs
  - dev/bevy/crates/bevy_asset/src/server/info.rs
  - dev/bevy/crates/bevy_asset/src/server/loaders.rs
  - dev/bevy/crates/bevy_asset/src/assets.rs
  - dev/bevy/crates/bevy_asset/src/event.rs
  - dev/Fyrox/fyrox-resource/src/manager.rs
  - dev/Fyrox/fyrox-resource/src/state.rs
  - dev/Fyrox/fyrox-resource/src/loader.rs
  - dev/Fyrox/fyrox-resource/src/untyped.rs
  - dev/Fyrox/fyrox-resource/src/event.rs
  - dev/Fyrox/fyrox-resource/src/entry.rs
  - dev/Fyrox/fyrox-resource/src/registry.rs
  - dev/godot/core/io/resource_loader.h
  - dev/godot/core/io/resource_loader.cpp
  - dev/godot/core/io/resource.h
  - dev/godot/core/io/resource.cpp
  - dev/godot/core/io/resource_uid.h
  - dev/godot/core/io/resource_uid.cpp
  - dev/godot/tests/core/io/test_resource.cpp
  - dev/godot/tests/core/io/test_resource_uid.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResources.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourceRegistry.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourcePool.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Common/ReloadAttribute.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/ResourceReloader.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/RenderGraphTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/PathTracing/ResourceCacheTests.cs
doc_type: review-and-refactor-plan
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# 112 · Runtime Resource Authority、Asset Handle、Load Request State Machine、Version Lease、Cache、Dependency、Reload、Cancellation 与 Product Integration 当前源码工程化差距复核

## 1. 结论

Runtime64 的最高风险在当前源码中没有关闭。公开 `ResourceManager::store_payload` 仍只校验资源存在、revision 与 `Ready`，不校验 exact payload type；现有测试仍把 `ShaderAsset` 写入 `Texture` record 并期待成功。随后 generic readiness 因“存在任意 `TypeId`”报告 `Loaded`，typed facade 因 downcast/type mismatch 报告 `NotLoaded`，`ensure_resident` 又因“存在任意 payload”直接返回，导致同一资源形成三种矛盾且不能自愈的事实。**RAR-P0-001 保持 Open。**

真实产品链也没有改成有界异步请求。`ProjectAssetManager::ensure_resident` 仍在调用线程持 residency stripe，同步执行 artifact `read()`、解码和 publication；`load_typed<T: Clone>` 随后深 clone。当前 render submission 与 `ResourceStreamer` 范围内仍有 **29 个** `load_*_asset` 调用点，其中 4 个直接位于 frame submission 构建链，另外 25 个位于资源流与其同步依赖解析链。源码中仍找不到 `AssetLoadCoordinator`、`LoadRequestId`、ticket、priority、deadline、cancel、`VersionLease` 或 `CachePolicy`。**RAR-P0-002 保持 Open。**

Runtime64 baseline 后唯一足以改变 finding 状态的增量是 registry staging 写时复制：registry map 与 record/locator 改用 `Arc` 共享，`begin_staging` 不再深 clone 4,096 条记录；但第一次 mutation 仍复制标准 `HashMap` bucket/Arc 指针，ignored release gate 的 Windows P50/P95 仍未进入 managed acceptance，也没有 persistent generation 或 overlay。因此 **RAR-P2-002 从 Open 变为 Partial**，不能 Closed。

当前总账为：**P0 2 Open；P1 66 Open、0 Partial、0 Closed；P2 16 Open、1 Partial、0 Closed；50 项 RAR gate 全部 Fail。** 本文不创建重复 finding；Runtime64 的 `RAR-*` 编号仍是唯一 canonical owner。目标仍是 `ExactAssetTypeCatalog + QualifiedAssetHandle + AssetLoadCoordinator + ResourceVersionSlot + VersionLease + CachePolicy + TypedDependencyGraph + ReloadCandidateTransaction + AssetPublicationReceipt`，并按固定三包架构落入 `zircon_runtime` 内部 spine，而不是新建第四个 root package。

本轮只做静态 review 与计划文档，没有修改 production、tests、Cargo、ABI 或参考源码，没有运行 Cargo、冷盘、真实窗口、100K 资产、OOM、取消风暴或性能基准。当前证据不能支持“已经达到或超过 Unreal”的结论。用户已明确暂不优化 tooling，本文不新增 Python、PowerShell、生成器或 tooling 迁移任务。

## 2. 审查边界、currentness 与 ownership

### 2.1 Canonical owner 与去重

| 主题 | canonical owner | 本轮动作 | 不重复拥有 |
|---|---|---|---|
| live resource authority、exact payload admission、load request、CPU version lease/cache、reload 与产品接线 | Runtime64 | 当前源码逐项刷新 2/66/17 findings 与 50 gates | `RAR-P0/P1/P2`、`RAR-G` 编号 |
| 广义 asset/resource/serialization | Runtime04 | 作为父合同继续引用 | importer/pack/serialization 全域 |
| stable identity/generation/owner epoch | Runtime24 | asset live handle 采用其通用合同 | 通用 ID 分配与耗尽策略 |
| registry/index/persistence/query | Runtime51 | 要求 published row 与 live version 同代 | authoring index/persistence 实现 |
| GPU residency/upload/retirement | Runtime09D、Runtime92、Runtime93 | 只规定其输入必须是已发布 CPU version | GPU 预算、upload 与格式语义 |
| watch/change generation | Runtime88、Runtime54 | asset receipt journal 对接其 cursor/resync | 文件 watch、跨进程 mirror |
| scheduler/cancellation/shutdown | Runtime59 | asset request 声明 owner/priority/deadline/cancel | 通用 task runtime |
| importer/build/DDC/cook | Runtime85、Runtime86、Plugins07 | 只消费 exact artifact/type descriptor | importer 与 cook worker 实现 |
| locator/reference/repair | Runtime87 | live handle 采用 qualified identity | authoring rename/move/repair |
| Editor content browser/preview | Editor04、Editor57 | 必须消费 Runtime ticket/version/catalog generation | Editor 控件与 authoring workflow |

绑定架构规则以后出的《Runtime 吸收层与 Editor/Scene 边界收束计划》为准：`zircon_app` 只做宿主，资源权威属于 `zircon_runtime`；中立合同放 `zircon_runtime::core::framework::asset`，稳定 resolver/handle 入口放 `zircon_runtime::core::manager`，具体 authority/coordinator 放 `zircon_runtime::core::resource` 与 asset runtime absorption 层。旧总方案里的独立非网络 `zircon_server` 不得复活。`zircon_editor` 只拥有作者态 projection、preview intent 与 UI 状态，不得复制 live resource authority。

### 2.2 当前源码物理冻结

| 冻结组 | 文件 | 行 | 非空行 | bytes | `#[test]` | ignored | 含 `unsafe` 行 |
|---|---:|---:|---:|---:|---:|---:|---:|
| Public contracts and typed facade | 27 | 2,393 | 2,080 | 74,073 | 4 | 0 | 0 |
| Resource authority, readiness, lease and event | 57 | 11,747 | 10,779 | 398,823 | 103 | 1 | 5 |
| Project load, publication and lifecycle | 28 | 3,852 | 3,490 | 145,879 | 25 | 1 | 0 |
| Runtime and Editor product consumers | 129 | 24,734 | 22,864 | 937,190 | 204 | 0 | 0 |
| Focused tests | 23 | 4,425 | 4,003 | 159,398 | 89 | 1 | 0 |
| 去重合计 | **258** | **45,267** | **41,520** | **1,646,963** | **377** | **2** | **5** |

Zircon 冻结集 fingerprint 为 SHA-256 `aa10fe2eb59372a98fc6ace37190f8fd71d8bc343b7a27e4f3e996d63ff5d299`。算法：repo-relative 路径转小写 `/`，逐文件计算 lowercase SHA-256，按 `path<TAB>hash` 排序，LF 连接且末尾不追加 LF，再对 UTF-8 payload 计算 SHA-256。

冻结时 coordinator baseline 为 HEAD `bee4c707b714738346b49bba15c59468b8bd9b39`、epoch 339，workspace health 因 2,278 个未接受变化处于 degraded。范围内有 5 个其他会话/用户修改文件：

- `zircon_runtime/src/asset/pipeline/manager/project_asset_manager/project_asset_manager.rs`
- `zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs`
- `zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime/tests.rs`
- `zircon_runtime/src/asset/pipeline/manager/project_asset_manager/watch_dispatch.rs`
- `zircon_runtime/src/core/resource/tests.rs`

本篇按当前 working copy 冻结并只读这些文件。前四个当前差异为格式或 watch/activation 相关变化；`core/resource/tests.rs` 包含 COW staging 测试。两个 P0 的 production 证据不依赖这些未提交差异。实现前必须重新计算 fingerprint、检查这 5 个文件和 baseline epoch，不能假定本文冻结仍是 live source。

参考冻结集为 **37 文件、48,023 行、41,512 非空行、1,829,528 bytes**，fingerprint `f2d4a8dd53c809b99089f3fef752293c71bea632bf8dc1cd9d5673f5bbe0667a`：

| 参考 | 文件 | 行 | 非空行 | bytes |
|---|---:|---:|---:|---:|
| Unreal | 9 | 27,116 | 23,313 | 995,346 |
| Bevy | 6 | 6,092 | 5,440 | 232,963 |
| Fyrox | 7 | 4,753 | 4,241 | 176,336 |
| Godot | 8 | 4,474 | 3,767 | 158,406 |
| Unity Graphics | 7 | 5,588 | 4,751 | 266,477 |

### 2.3 复核方法与 current-source delta

1. 从 public `ResourceMarker/Record/Handle` 沿 mutation、registry、payload、runtime slot、readiness、event、lease 到 facade，逐步核对 identity、exact type、state 和 lifetime 是否是一条事实链。
2. 从 `ProjectAssetManager::load_*` 沿 `ensure_resident -> artifact read/decode -> store_payload -> downcast -> clone` 追到 frame submission、ResourceStreamer、PBR app 与 Editor，而不是只检查 manager API。
3. 读取 focused tests，区分行为测试、源码形状 assertion 与 ignored performance gate；特别检查 wrong payload、old lease、reload failure、project close、COW staging。
4. 比较 Runtime64 baseline `bea1acf91b909525ab1759e2c800858b0eda6528` 到当前 HEAD 的 15 个相关文件、`+787/-293` 行变化。两次提交中只有 COW registry staging 改变本篇 finding 状态；watch wake、render host 迁移和格式变化没有关闭 RAR finding。
5. 按参考引擎真实代码与测试核对 request、type、lifetime、cache、dependency、reload 和 failure boundary；不以名称相似推断能力。
6. 本轮没有运行 Cargo 或 ignored benchmark；测试数量只说明现有 source surface，不能替代执行证据。

## 3. 当前真实链路

```text
serialized Handle<T>
  = ResourceId + PhantomData
  -> untyped conversion adds only broad ResourceKind
  -> no exact type / owner epoch / slot generation / provider generation

public payload mutation
  -> store_payload(id, expected_revision, T)
  -> preflight checks id + revision + Ready
  -> replaces erased Arc<dyn ResourceData>
  -> generic readiness: Ready + any TypeId => Loaded
  -> typed readiness/downcast: wrong TypeId => NotLoaded / None
  -> ensure_resident: any erased payload => Ok, so no artifact repair

product load
  -> load_*_asset(id)
  -> ensure_resident(id)
       -> lock one of 64 residency stripes
       -> synchronous prepare/read/decode full ImportedAsset
       -> project generation recheck
       -> store_runtime_payload
  -> typed Arc downcast
  -> deep clone T

lease/cache
  -> acquire increments current slot ref_count with unchecked += 1
  -> get/snapshot return untracked Arc
  -> last tracked lease drop removes authority payload immediately
  -> old untracked Arc/clone may survive while same revision is loaded again

reload/project replacement
  -> payload replacement installs a new token/ref_count=0 slot
  -> old lease token cannot evict new slot, but retired version has no census
  -> failed reload marks record/runtime Error while last-good payload remains readable
  -> project close deletes row/slot; stale Copy handle has no owner epoch
  -> reopened same locator may derive same ResourceId and alias replacement silently
```

当前应保留但必须深化的基础包括：preflight/apply transaction 入口、同一 `ResourceAuthority` 内的 registry/payload/runtime/readiness、project generation 二次确认、last-good 局部保留、旧 token 不驱逐新 slot、event backlog/gap、prepared artifact read、Editor gap reconcile，以及 COW staging 对不变 record/locator 的共享。这些能力不能用来证明完整；它们是后续 hard cutover 的迁移起点。

## 4. P0 当前证据

### RAR-P0-001：公开 payload mutation 仍允许错误 exact type 并形成不可自愈三重事实

状态：**Open**。

- `zircon_runtime/src/core/resource/manager/payload_ops.rs:30-40` 的 public `store_payload<TData>` 直接构造 erased mutation，没有 exact type/catalog 参数。
- `zircon_runtime/src/core/resource/manager/commit.rs:182-206` 的 `StorePayload` preflight 只验证 record、revision 与 `Ready`，随后直接 `PayloadMutation::Replace(payload)` 并标为 `Loaded`。
- `zircon_runtime/src/core/resource/manager/readiness_projection.rs:287-288` 以 `payload_type_id.is_some()` 判 generic `Loaded`；`readiness_generation.rs:38-45` 再在 typed query 时把 type mismatch 降为 `NotLoaded`。
- `zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/ensure_resident.rs:12-15` 只要 `get_untyped(id).is_some()` 就返回成功，因此 wrong payload 无法触发 artifact 自愈。
- `zircon_runtime/src/asset/tests/facade/load_state_roots.rs:55-79` 明确写入 wrong `ShaderAsset`、`.unwrap()` 成功，然后期待 texture `get=None`、typed state `NotLoaded`。这不是保护测试，而是在固定错误合同。

风险不是普通类型错误：mutation 已提交、generic state 已 Loaded、typed state NotLoaded、repair path 又提前成功。后续 renderer/Editor/cache 可能各自观察不同事实，且事件无法指出 exact type 破坏。关闭条件必须是 M64.1 的原子拒绝矩阵：wrong register/store/reload candidate 在 preflight 返回 typed error，registry/payload/runtime/readiness/event generation 与 bytes 全部不变，立即重试正确类型成功。

### RAR-P0-002：frame/product path 仍可内联执行冷盘读取、解码与深 clone

状态：**Open**。

- `ensure_resident.rs:11-102` 在调用线程持 residency stripe；project asset 路径在 `:66-75` 同步 `.read()` artifact，并在 `:96-100` 同步发布 payload。
- `load_typed.rs:22-26` 调用 `ensure_resident` 后执行 `asset.as_ref().clone()`；`acquire_typed.rs:22-25` 也先走同一同步冷加载。
- frame submission 当前直接调用：`build_frame_submission_context/build.rs:574` model，`material_feature_extract.rs:49/65` material/parent，`target_resolution.rs:88` texture。
- ResourceStreamer 及其同步依赖链另有 25 个 `load_*_asset` 调用，覆盖 model、mesh、scene、material、texture、shader、animation skeleton、mip streaming 与 material capture。
- product source 对 `AssetLoadCoordinator`、`LoadRequestId`、`AssetLoadTicket`、`VersionLease`、`CachePolicy`、`reprioritize`、`priority_inheritance` 的 tracked hit 均为 0。

固定 64 stripe 只能减少同 ID 重复工作，不能给 I/O/CPU 时间上界；锁还会让同 stripe 的无关资源互相阻塞。普通 `load_*` 名称隐藏同步行为，material parent/shader/texture 可串成更深同步链。关闭条件必须同时满足 M64.4 与 M64.8：普通冷 load 只做有界 admission；frame thread 无 read/decode/wait/deep-clone API；required assets 在 activation/preload barrier 前完成，缺失资源只能走显式 fallback/skip receipt。

## 5. Runtime64 P1 状态逐项刷新

状态计数：**Open 66；Partial 0；Closed 0**。

### 5.1 Exact type、schema 与 authority admission

| ID | 状态 | 当前源码证据 |
|---|---|---|
| RAR-P1-001 | Open | `ResourceRecord`仍只有 broad `ResourceKind`、artifact/revision/dependency/importer 字段，无 exact type/schema/codec/provider identity。 |
| RAR-P1-002 | Open | `ResourceMarker`仍只有 `const KIND`，不能证明 Rust payload 或插件 schema。 |
| RAR-P1-003 | Open | Texture/UiIcon、UiLayout/UiV2View、UiWidget/UiV2Component、UiStyle/UiTheme/UiV2Style 仍共享 marker/kind。 |
| RAR-P1-004 | Open | `load_imported_asset.rs:42-75` 仍用 `.or_else` 依次 probe alias type，错误仍充当类型发现控制流。 |
| RAR-P1-005 | Open | `load_asset`/`acquire_asset` 仍是手写 typed 方法清单，没有 catalog operation table 与 inventory completeness gate。 |
| RAR-P1-006 | Open | `ResourceHandle<T>`仍只序列化 `ResourceId`，PhantomData 被跳过；facade handle 同样没有 qualified identity。 |
| RAR-P1-007 | Open | `UntypedResourceHandle::typed`仍只比较 broad kind 后构造 typed handle。 |
| RAR-P1-008 | Open | record/live slot 仍无 project/session owner generation。 |
| RAR-P1-009 | Open | `ResourceData: Any`、public `get_untyped` 与 erased payload 仍可绕过 catalog/codec contract。 |
| RAR-P1-010 | Open | `ResourceSnapshot<T>`仍只有 cloned record + Arc payload，缺 slot/schema/provider/publication receipt。 |

### 5.2 Load request、状态机与并发

| ID | 状态 | 当前源码证据 |
|---|---|---|
| RAR-P1-011 | Open | canonical API 仍无 request ID、ticket、future 或 terminal receipt。 |
| RAR-P1-012 | Open | production residency 在 read 前仍不发布真实 Queued/Reading/Decoding/Publishing 状态。 |
| RAR-P1-013 | Open | `ResourceState::Pending`仍直接投影 Loading，不要求绑定 live task/request。 |
| RAR-P1-014 | Open | `ensure_resident`仍同步完成完整 artifact read/decode/publication。 |
| RAR-P1-015 | Open | residency stripe 仍跨慢 read/decode 持有；无 lock-free waiter fan-out 或 publish CAS。 |
| RAR-P1-016 | Open | 固定 64 stripe/`DefaultHasher` 仍缺 occupancy、queue wait、hold 与 hot-key 诊断。 |
| RAR-P1-017 | Open | mutex 仍只防重复执行，没有 qualified single-flight key、共享结果或 waiter identity。 |
| RAR-P1-018 | Open | 仍无 priority、deadline、progress、stage、start-stalled、reprioritize。 |
| RAR-P1-019 | Open | 仍无 request owner、cancel-on-project-close/module-shutdown 与 callback drain。 |
| RAR-P1-020 | Open | `asset_error`/`asset_error_message` 仍统一压为 `CoreError::Initialization(name, String)`。 |
| RAR-P1-021 | Open | 仍无 retry classification、backoff、negative cache 与 input-generation invalidation。 |
| RAR-P1-022 | Open | 仍无 request stage bytes/items/time、queue wait、dedup waiter、wasted/superseded work 指标。 |

### 5.3 Handle、version lease 与 cache policy

| ID | 状态 | 当前源码证据 |
|---|---|---|
| RAR-P1-023 | Open | `get`/`snapshot`仍返回不增加 slot refcount 的 Arc；长期访问可绕开 lease。 |
| RAR-P1-024 | Open | tracked lease 清零可删 authority payload，外逃 Arc/clone 仍活着并可导致同 revision 第二份对象。 |
| RAR-P1-025 | Open | last lease drop 仍立即卸载，不考虑 budget、reuse、pin、cost 或 pressure。 |
| RAR-P1-026 | Open | Copy handle 仍没有 strong/weak/soft 区分，看似 live 却不保活。 |
| RAR-P1-027 | Open | locator-derived ID 与无代 handle 仍允许 close/reopen、remove/re-register 别名 replacement。 |
| RAR-P1-028 | Open | `ResourceLease`仍只有 ID、u64 token、Arc、release callback；无 revision/type/project/provider。 |
| RAR-P1-029 | Open | token allocator 仍 `wrapping_add().max(1)`，可静默 ABA。 |
| RAR-P1-030 | Open | acquire 仍执行 unchecked `slot.ref_count += 1`。 |
| RAR-P1-031 | Open | payload Replace 仍安装新 slot/token 并把 refcount 置 0，无旧 lease census。 |
| RAR-P1-032 | Open | 仍无 retired generation、quiescence、retired bytes/version 上限。 |
| RAR-P1-033 | Open | CPU payload 仍无 estimated/actual bytes、budget、LRU、pin、priority 与 pressure policy。 |
| RAR-P1-034 | Open | `load_typed<T: Clone>`仍把深 clone 作为基础成功路径。 |
| RAR-P1-035 | Open | `ResourceCacheIdentity`仍只有 revision/state，忽略 exact type、slot、artifact、provider、dependency generation。 |

### 5.4 Dependency、reload 与 publication

| ID | 状态 | 当前源码证据 |
|---|---|---|
| RAR-P1-036 | Open | dependency 仍是 `Vec<ResourceId>`，无 required/optional、hard/soft、type/schema/version/provenance。 |
| RAR-P1-037 | Open | readiness cycle 仍用 cycle hash 并返回聚合 Loaded 语义，没有 publication 前 SCC policy。 |
| RAR-P1-038 | Open | dependency source 仍以任意 payload `TypeId` 存在判断 generic Loaded。 |
| RAR-P1-039 | Open | readiness 仍是派生投影而非 request graph，不能驱动调度、取消和 priority inheritance。 |
| RAR-P1-040 | Open | missing/failed/optional unavailable/version mismatch 仍被压缩，缺 edge/path cause chain。 |
| RAR-P1-041 | Open | production reload 仍无 active/candidate 双版本事务与 validate-before-publish。 |
| RAR-P1-042 | Open | failed reload 仍把 record/runtime 标为 Error，同时 `get/acquire` 可返回 last-good。 |
| RAR-P1-043 | Open | last-good 仍依靠 `Reloading/Error` 特殊分支，不是 formal version policy。 |
| RAR-P1-044 | Open | record 不变的 payload replacement 仍不产生 before/after version publication event。 |
| RAR-P1-045 | Open | typed event 仍只按 broad `resource_kind` 过滤，alias type 会互收事件。 |
| RAR-P1-046 | Open | event 仍无 request/owner/candidate-active/cache/terminal disposition。 |
| RAR-P1-047 | Open | event coalesce/cursor 仍缺完整 publication generation receipt/resync 合同；Runtime54 继续拥有跨进程部分。 |
| RAR-P1-048 | Open | `PreparedResourceMutation`仍持 global commit serial 到 event publication 完成。 |
| RAR-P1-049 | Open | `commit_resource_batch_after_dependencies` 先 `prepare_commit` 持 serial，再调用任意 `commit_dependencies` closure。 |

### 5.5 Runtime、Renderer、Editor 与 project lifecycle 产品接线

| ID | 状态 | 当前源码证据 |
|---|---|---|
| RAR-P1-050 | Open | neutral `core::framework::asset::ResourceManager`仍只有 ID/status/generation/revision/subscribe，无 typed request/snapshot。 |
| RAR-P1-051 | Open | authoring `AssetManager` contract 仍未与中立 `RuntimeAssetService` 明确拆分。 |
| RAR-P1-052 | Open | graphics/scene/Editor 仍解析具体 `ProjectAssetManager` 或复制 `ProjectManager`，未只依赖中立 facade。 |
| RAR-P1-053 | Open | material fallback/parent/shader/texture 仍在多个 ResourceStreamer/frame 路径重复同步加载与 clone。 |
| RAR-P1-054 | Open | model/asset cache 仍保存 clone/value 并只看窄 revision identity，不持 version snapshot/lease。 |
| RAR-P1-055 | Open | camera target、material feature 与 frame submission 仍在 submission 构建时解析资产。 |
| RAR-P1-056 | Open | scene/project/viewport 仍无 required/optional preload set、budget/deadline/admission receipt。 |
| RAR-P1-057 | Open | `EditorAssetState`仍复制 project、catalog maps、reference graph、preview scheduler 与独立 generation。 |
| RAR-P1-058 | Open | Editor preview 使用独立 `EditorJob`/preview token，未暴露 Runtime load ticket/version/candidate failure。 |
| RAR-P1-059 | Open | close_project 仍删除 row/slot，而 outstanding Copy handle 无 owner epoch，下一项目可复用 ID。 |

### 5.6 测试、错误与可运维性

| ID | 状态 | 当前源码证据 |
|---|---|---|
| RAR-P1-060 | Open | wrong-payload 测试仍把成功 mutation 与 typed NotLoaded 当预期，没有原子拒绝/零变化断言。 |
| RAR-P1-061 | Open | 仍无 same-locator close/reopen、remove/re-register 的 stale handle/lease 产品矩阵。 |
| RAR-P1-062 | Open | 仍无 frame path 零文件 I/O、零 decode、零深 clone 的结构与运行双门。 |
| RAR-P1-063 | Open | 仍无 priority inversion、cancel race、deadline、waiter drop、shutdown drain 状态机测试。 |
| RAR-P1-064 | Open | 仍无 CPU payload bytes、eviction、pressure、OOM、retired version 规模门。 |
| RAR-P1-065 | Open | load failure 仍字符串化，测试不能稳定断言 cause/disposition/provenance。 |
| RAR-P1-066 | Open | 仍无 read/decode/dependency/validate/publish 每阶段 fault injection 与零半提交证明。 |

## 6. Runtime64 P2 状态逐项刷新

状态计数：**Open 16；Partial 1（002）；Closed 0**。

| ID | 状态 | 当前源码证据 |
|---|---|---|
| RAR-P2-001 | Open | `ResourceAuthority`仍是单 RwLock，100K/1M row 并发 hold/wait 分布未知。 |
| RAR-P2-002 | Partial | registry/staging 已用 `Arc<HashMap>` 与 Arc record/locator 消除 begin-staging 深 clone；首写仍复制 bucket/Arc 指针，persistent generation/overlay 与 managed Windows P50/P95 未完成。 |
| RAR-P2-003 | Open | residency/readiness shard 仍依赖 `DefaultHasher`，不可跨进程稳定重放。 |
| RAR-P2-004 | Open | readiness/event generation、cursor、dependency revision 仍有多处 `wrapping_add`。 |
| RAR-P2-005 | Open | poison lock 仍多处 `into_inner` 继续运行，无 fault diagnostic/quarantine/rebuild policy。 |
| RAR-P2-006 | Open | event bytes 仍是近似值，不计 String/Vec capacity 与共享 owner。 |
| RAR-P2-007 | Open | event 4,096/4 MiB/60 秒仍是固定全局策略，未按 consumer/resync SLO 配置。 |
| RAR-P2-008 | Open | `load_*` 命名仍隐藏同步 I/O/decode/clone 成本，没有受限 `load_blocking_cloned` 过渡面。 |
| RAR-P2-009 | Open | `ResourceData`仍强制 `Debug`，大型 payload 诊断与 authority trait 耦合。 |
| RAR-P2-010 | Open | `ResourceRuntimeInfo`/management/readiness 仍是多套投影，没有统一 snapshot generation。 |
| RAR-P2-011 | Open | `ResourceLease`仍不可 clone/downgrade/upgrade，调用者易退回裸 Arc。 |
| RAR-P2-012 | Open | alias probing 顺序仍是隐式兼容策略，新增 exact type 可改变结果。 |
| RAR-P2-013 | Open | dependency fingerprint 仍用 `DefaultHasher`，cycle diagnostic 与健康聚合混用。 |
| RAR-P2-014 | Open | readiness row 仍混合 record、payload TypeId、递归状态，consumer projection 较重。 |
| RAR-P2-015 | Open | `Assets::contains`仍只按 broad kind/readiness 判断，wrong exact type 可被当作存在。 |
| RAR-P2-016 | Open | typed load/acquire/event 方法仍无 catalog-generated completeness test。 |
| RAR-P2-017 | Open | 仍无统一 bounded request/version/cache/dependency diagnostic snapshot 供 Editor/profiler 使用。 |

`registry_staging_copy_on_write_release_gate` 是 ignored release test，使用 4,096 records、16 rounds/sample、21 sample pairs，要求 optimized P95 不超过 legacy 的 25%。它证明的只是 staging startup 深 clone 候选优化，不证明 authority lock、cache、request、frame 或 100K asset gate；因此不能让 RAR-G12、G36、G46 或 G50 变成 Partial/Pass。

## 7. 五套参考实现对照

| 参考 | 当前源码与测试中的可验证事实 | Zircon 应吸收的工程合同 | 不应误抄的边界 |
|---|---|---|---|
| Unreal Streamable/AsyncLoading2 | `FStreamableHandle`支持 priority 更新、cancel、start stalled、complete/cancel/update delegate、progress/count、timeout wait 与 request IDs；AsyncLoading2 有 package request/state、I/O priority queue、event graph、recursive dependency、external reads、cancel sets；tests 覆盖 timeout 后续继续完成、Serialize/PostLoad 递归 load、request-scoped/full flush、cycle ordering。 | request identity、owner、priority/deadline/progress/cancel、terminal exactly-once、typed dependency graph、递归 priority、受限 sync wait、failure/flush 测试矩阵。 | 不复制 UObject/global manager 形态；Zircon 仍应保持 Rust ownership、固定三包根结构与无非网络 server 命名。 |
| Bevy Asset | strong handle 用 `Arc<StrongHandle>` 保活并携 runtime `TypeId`；typed/untyped 转换检查 exact `TypeId`；`AssetServer::load`非阻塞并复用既有 path handle，在 `IoTaskPool` spawn；`AssetInfos`跟踪 pending tasks、wakers、loading/failed dependencies 与 recursive state；events 区分 Added/Modified/Removed/Unused/LoadedWithDependencies。 | exact typed handle、strong lifetime、path single-flight、pending task/waker、dependency failure propagation、unused 与 removed 分离。 | Bevy handle/UUID 仍不是 Zircon 的 owner epoch/slot generation 完整答案，也不能替代 deadline/cache/version transaction。 |
| Fyrox Resource | manager request 是非阻塞 shared resource；`ResourceState`有 Unloaded/Pending+Wakers/LoadError/Ok；`UntypedResource`实现 Future；loader 暴露 `data_type_uuid` 与 async `load`；events区分 Loaded/Reloaded/Removed；entry 有 TTL；registry维护 UUID->path。 | stable exact type UUID、pending/waker、shared once-load、typed loader registration、reload event 与 cache TTL 的基础合同。 | Fyrox 文档也承认部分 request 路径缺严格类型保证；Zircon 不能把其局限当目标上限。 |
| Godot ResourceLoader/UID | threaded request 按 path 复用 `LoadToken`，有 status/progress/condition variable/dependency progress；cache mode 区分 ignore/reuse/replace 与 deep variants；format loader 可注册并提供 dependency/rename；ResourceCache 路径接管有原子锁；ResourceUID 有 path mapping 与边界编码测试。 | request token/dedup、worker status/progress、显式 cache policy、loader/provider registry、dependency rename、稳定 soft identity。 | `load_threaded_get`仍可阻塞，global path cache 与字符串 type hint 不应成为 Zircon 的最终并发/类型模型。 |
| Unity Graphics | RenderGraph `ResourceHandle`编码 index/type/version/validity，write 产生新 version；registry区分 imported/shared/transient，校验错误使用并按 lifetime create/release；pool跟踪 frame allocation/purge；tests覆盖 version/validity、transient 越界、release、pool reuse；ResourceReloader用 typed field/path/package metadata修复 null/broken field并区分 AssetDatabase 暂不可用。 | live instance generation、versioned publication、imported/transient/persistent 分类、显式 release/purge、声明式 typed repair 与 retriable failure。 | Unity Graphics 不是通用资产加载 authority；它只能作为 version handle、resource lifetime 与 reload repair 参考，不能证明 request/cancel/dependency 完整。 |

Unreal 是系统规模和 failure-boundary 主参考，但“优于 Unreal”必须由相同 workload 下的 p50/p95/p99/p99.9、I/O、CPU、RSS、取消/重载风暴与正确性门禁证明，不能由 API 名称、少量单测或局部 COW microbenchmark 宣称。

## 8. 目标架构与不变量

### 8.1 固定 ownership 与 public surface

```text
zircon_app
  -> 只选择 profile、创建 CoreRuntime、驱动 loop

zircon_runtime::core::framework::asset
  -> neutral DTO/traits:
     ExactAssetTypeId, QualifiedAssetId, LoadRequestDescriptor,
     LoadTicketView, AssetVersionView, AssetPublicationReceipt

zircon_runtime::core::manager
  -> stable service name / resolver / generation-qualified service handle

zircon_runtime::core::resource
  -> ResourceAuthority, ExactAssetTypeCatalog, ResourceVersionSlot,
     VersionLease, TypedDependencyGraph, receipt journal

zircon_runtime::asset
  -> provider/imported artifact adapters, AssetLoadCoordinator,
     I/O/decode/validation lanes, project owner lifecycle

zircon_editor
  -> catalog/details/preview authoring projection and request intent only;
     no copied live payload/load/version authority
```

### 8.2 Identity、type 与 version

- `QualifiedAssetId = OwnerId + OwnerEpoch + StableAssetId + ExactAssetTypeId`；soft ref 可序列化但不保活。
- `LiveAssetVersionKey = QualifiedAssetId + SlotGeneration + ContentRevision + SchemaId + ProviderGeneration`；任何一项不匹配都返回 typed `Stale/TypeMismatch/SchemaMismatch/ProviderRetired`。
- `ResourceRegistryGeneration`只保存 identity、exact schema、artifact descriptor 与 typed edges；`ResourceVersionSlot`分 `active/candidate/retired`，不能用同一个 record state 同时表达 active 可用与 candidate 失败。
- `VersionLease<T>`是 authority 外长期 payload 访问的唯一 owner。裸 Arc 不跨 facade；refcount=0 只表示可驱逐，是否回收由 cache policy 决定。
- generation/token/refcount 全部 checked；耗尽返回 typed admission failure 且 authority 零变化，不允许 wrap、saturate 后继续或重新别名。

### 8.3 Request、dependency 与 publication

`AssetLoadCoordinator`按 `(QualifiedAssetId, exact type, requested revision/policy)` single-flight。ticket 状态至少为：

```text
Admitted -> Queued -> Resolving -> Reading -> Decoding
         -> WaitingDependencies -> Validating -> Publishing
         -> Ready | Failed | Cancelled | Superseded
```

每个非 terminal state 必须引用真实 queue/task/waiter；每个 terminal 只能发布一次 receipt。I/O、decode、dependency、validation 在 authority lock 外执行，publish 只在锁内做有界 type/schema/owner/revision CAS、active slot swap 与 journal append。priority、deadline、owner cancellation 与 dependency priority inheritance 对接 Runtime59，而不是在 asset manager 内自建第二套 scheduler。

typed dependency edge 至少携 required/optional、hard/soft、expected exact type/schema、version policy、provider/provenance。candidate registry publication 前做 SCC；非法 required cycle 失败，允许环必须有显式 edge policy。parent ticket 聚合 child progress/cause，并在 parent 取消或 optional branch 不再需要时停止无用工作。

### 8.4 Cache、reload 与 product invariant

- compressed artifact section、decoded CPU payload、GPU residency 分别计量；CPU cache 按 bytes/items/cost/priority/owner/pin/admission budget 驱逐，Runtime09D/92/93 只消费已发布 CPU version。
- reload 只构建 candidate；失败发布 candidate failure 而 active 继续 Ready，成功一次交换 active 并产生 before/after receipt；旧 lease 稳定读取旧 version，retired bytes/version 有上限并在 lease 清零后回收。
- scene/project/viewport activation 提交 required/optional asset set、budget、deadline，获得 admission receipt。frame submission 只能读取 ready snapshot 或显式 fallback/skip receipt，不能调用 blocking load/wait/read/decode/clone。
- Editor details/preview 观察同一个 Runtime ticket、version、catalog generation；Editor 的独立 job 只承载作者态工作，不能伪造 Runtime load state。
- project/module/provider close 先停止 admission、取消 owner requests、drain managed callbacks、退休 owner/provider epoch，再释放 active/candidate；旧 handle 永久 stale，旧 lease 仅保活 retired version。

## 9. 依赖序重构里程碑

沿用 Runtime64 的 M64.1-M64.10，避免创建平行实施计划。每个里程碑先写失败测试，实现切片后统一进入自己的 testing stage；测试通过后只在 canonical child plan 记录一次 accepted outcome。

| 里程碑 | 依赖与实现范围 | testing stage / 退出证据 |
|---|---|---|
| M64.1 Exact type admission | 建立 stable exact type/schema/provider catalog；record/artifact/mutation/event 全链绑定 exact type；删除 public arbitrary wrong payload 成功路径。 | wrong register/store/reload/alias matrix；失败后 registry/payload/runtime/readiness/event bytes/generation 不变；关闭 RAR-P0-001。 |
| M64.2 Qualified handle | 依赖 M64.1；实现 owner epoch、slot generation、soft/weak/strong type、serialization 与 typed stale error。 | close/reopen、remove/re-register、provider replacement、generation exhaustion/roundtrip 全矩阵，无 ABA。 |
| M64.3 Version slot/lease | 依赖 M64.2；active/candidate/retired slot、checked refcount/token、唯一长期 lease owner。 | raw Arc escape、old/new lease、replace、drop、overflow、retired census 与 memory accounting；不驱逐 live version。 |
| M64.4 Load coordinator | 依赖 M64.1-M64.3 与 Runtime59；ticket 状态机、single-flight、I/O/CPU lanes、priority/deadline/cancel/receipt。 | deterministic fake I/O/scheduler 覆盖所有 transition、dedup、waiter drop、cancel race、timeout、shutdown；terminal exactly-once。 |
| M64.5 Typed dependency | 依赖 M64.4；typed edge、SCC、child request plan、cause chain、priority inheritance。 | required/optional/type/version/cycle/fan-out/cancel corpus；非法 graph 零发布。 |
| M64.6 Cache policy | 依赖 version lease/request；CPU cache admission/eviction、bytes/cost/LRU/pin/pressure/negative cache。 | 1K/100K assets、large model/texture、OOM/pressure、retired version；预算内稳定且重试按 generation 失效。 |
| M64.7 Reload transaction | 依赖 typed dependency/cache；candidate build、validation、last-good、atomic publish、retirement。 | success/fail/cancel/supersede + live old lease；active/candidate state 不矛盾，payload-only publish 有 receipt。 |
| M64.8 Runtime product migration | 依赖 M64.4-M64.7；renderer/scene/text/streamer 改 preload plan + version snapshot/fallback，删除按值 clone 的普通 load API。 | source guard 禁 blocking load；cold/warm frame instrumentation 证明零文件 I/O/零 decode/零深 clone；关闭 RAR-P0-002。 |
| M64.9 Editor/project lifecycle | 依赖中立 service 与 owner retirement；Editor catalog/preview 消费 Runtime truth，project close 完整 drain。 | project switch、preview cancel、gap/resync、provider unload、old handle/lease；无第二 live authority。 |
| M64.10 Scale/soak | 依赖所有前置；统一 diagnostics、budgets、fault/soak、同 workload baseline。 | 100K assets、高 fan-out reload/cancel storm、OOM、cold/warm frame p50/p95/p99/p99.9；50 gates 全 Pass 后才可接受。 |

M64.1 是不可跳过的第一实施切片。先加 `load_async` 而保留 wrong payload、无代 handle、裸 Arc 和同步 frame fallback，只会把当前不一致扩散到更多线程。

## 10. 50 项门禁当前状态

Runtime64 的 RAR-G01-G50 逐项定义继续有效，本轮没有证据把任何 gate 标为 Partial 或 Pass。为避免范围行掩盖漏项，current-source 状态逐项展开如下。

| Gate | 当前状态 | 当前阻断证据 |
|---|---|---|
| RAR-G01 | Fail | published record 无 stable exact type/schema/provider/owner epoch。 |
| RAR-G02 | Fail | wrong payload 仍可通过 public store，authority 随后进入矛盾状态。 |
| RAR-G03 | Fail | alias asset 仍共享 broad kind 并依赖 marker/probe order。 |
| RAR-G04 | Fail | typed handle 转换仅验 kind，exact type 要到 downcast 才发现。 |
| RAR-G05 | Fail | live handle 无 owner epoch，close/reopen 后不能证明永久 stale。 |
| RAR-G06 | Fail | slot/provider generation 不存在，token wrapping 仍允许 ABA 风险。 |
| RAR-G07 | Fail | soft/weak/strong handle 与 roundtrip 合同未建立。 |
| RAR-G08 | Fail | `get`/snapshot 仍可把裸 `Arc` 带出 authority。 |
| RAR-G09 | Fail | replacement 安装新 slot 并把 census 重置为零。 |
| RAR-G10 | Fail | token wrapping 与 unchecked refcount 未返回 typed exhaustion failure。 |
| RAR-G11 | Fail | last lease drop 仍立即删除 payload。 |
| RAR-G12 | Fail | CPU cache 无 bytes/items/cost/priority/owner admission 与 eviction。 |
| RAR-G13 | Fail | active/candidate/retired version 与 bytes 不存在。 |
| RAR-G14 | Fail | cold load 在调用线程读盘、decode，并可能深 clone。 |
| RAR-G15 | Fail | 无 request ID、owner、priority、deadline、debug source。 |
| RAR-G16 | Fail | `Pending` 可在没有真实 task/queue/waiter 时投影 Loading。 |
| RAR-G17 | Fail | fixed stripe 只防重入，没有 qualified single-flight shared result。 |
| RAR-G18 | Fail | waiter drop、owner close、explicit cancel 无 exactly-once terminal。 |
| RAR-G19 | Fail | 无 reprioritize 与 dependency priority inheritance。 |
| RAR-G20 | Fail | 无 stage progress/bytes/time/queue wait/cancel cause。 |
| RAR-G21 | Fail | load failure 继续压成字符串，typed cause 不可稳定判断。 |
| RAR-G22 | Fail | 无 retry/backoff/negative cache 与 generation invalidation。 |
| RAR-G23 | Fail | I/O、decode、validation、publication 无完整 fault injection。 |
| RAR-G24 | Fail | 无 request/artifact generation ticket，不能证明 superseded work 零发布。 |
| RAR-G25 | Fail | dependency edge 仍是裸 `ResourceId`。 |
| RAR-G26 | Fail | candidate registry publish 前无 typed SCC policy。 |
| RAR-G27 | Fail | optional/required unavailable 与 cause 仍被聚合 readiness 压缩。 |
| RAR-G28 | Fail | 无 parent ticket、child progress/failure 聚合或分支取消。 |
| RAR-G29 | Fail | reload candidate failure 与 active state 仍混写为 Error。 |
| RAR-G30 | Fail | 无一次 active swap 与旧 version lease 稳定性证明。 |
| RAR-G31 | Fail | payload-only replacement 无 before/after version receipt。 |
| RAR-G32 | Fail | typed event 仍只按 broad kind 过滤。 |
| RAR-G33 | Fail | publication generation 与完整 gap resync 合同未闭合。 |
| RAR-G34 | Fail | receipt 无 owner/request/version/terminal disposition。 |
| RAR-G35 | Fail | resource serial 仍跨 dependency closure 与 event publication。 |
| RAR-G36 | Fail | authority/residency lock 可跨慢 I/O/decode，且无 hold/wait profile。 |
| RAR-G37 | Fail | scene/project/viewport 无 required/optional preload receipt。 |
| RAR-G38 | Fail | render submission 范围仍有 29 个 direct synchronous `load_*_asset` 调用。 |
| RAR-G39 | Fail | 无 cold-frame 零文件 I/O、零 decode、零深 clone 运行证据。 |
| RAR-G40 | Fail | missing/cold frame 无统一 fallback/skip receipt。 |
| RAR-G41 | Fail | material parent/shader/texture 仍在 frame/resource streamer 中临时解析。 |
| RAR-G42 | Fail | CPU payload snapshot 与 GPU upload/residency version 未绑定。 |
| RAR-G43 | Fail | Editor preview/catalog 仍复制 project/load projection。 |
| RAR-G44 | Fail | project close 无 cancel/drain/owner epoch retirement 完整事务。 |
| RAR-G45 | Fail | provider/module shutdown 无 late publish/callback 隔离证明。 |
| RAR-G46 | Fail | 100K asset/high-fan-out reload 的 commit/read p95/p99 未运行。 |
| RAR-G47 | Fail | large asset/reload storm/OOM 下 CPU cache 预算门不存在。 |
| RAR-G48 | Fail | cancel/deadline/priority storm 下的 queue/fairness 门不存在。 |
| RAR-G49 | Fail | cold/warm 同 workload p50/p95/p99/p99.9、I/O、CPU、RSS 未记录。 |
| RAR-G50 | Fail | focused/product/fault/soak 与综合 Markdown/link/index 验收未全部通过。 |
| 合计 | **50 Fail** | COW staging 只让 RAR-P2-002 Partial，不满足任何完整 gate。 |

只有在一个 gate 的全部行为、失败边界、规模门和 testing stage 都有证据时才能 Pass；单个结构测试、ignored benchmark 或局部 allocation 降低不能把 gate 标为 Partial 来制造进度。

## 11. 禁止的临时修补

1. 禁止只给同步 `load_*` 套 async/task wrapper，却仍让任务持 residency stripe、深 clone 并缺 request owner/cancel/receipt。
2. 禁止继续用 `ResourceKind`、Rust type name、probe order 或 downcast failure 推断 exact asset type。
3. 禁止为 wrong payload 增加“typed NotLoaded 后再试一次”的自愈分支；mutation 必须在 preflight 原子拒绝。
4. 禁止在 live handle 新增 generation 后保留“旧 handle 回退到 stable ID 最新 slot”的兼容 shim。
5. 禁止保留 naked Arc/get 与 VersionLease 两套并行长期访问合同。
6. 禁止把 last lease drop、TTL 或 LRU 任一单项冒充完整 cache policy；必须有 bytes、budget、owner、pin、pressure 和诊断。
7. 禁止 reload 直接覆盖 active payload 后再验证；candidate 必须先完整验证再一次 publish。
8. 禁止用 record `Error` 同时表示 active 不可用与 candidate 失败。
9. 禁止在 global resource serial/authority lock 内执行 filesystem、decode、arbitrary callback 或 Editor publication。
10. 禁止让 Editor preview/catalog generation 成为第二套 Runtime load/version truth。
11. 禁止把 Unity Graphics RenderGraph handle 当作通用 asset authority，或把 Bevy/Fyrox/Godot 的局限当作 Zircon 的目标上限。
12. 禁止在同 workload 的正确性与 p50/p95/p99/p99.9 证据前宣称性能或表现超过 Unreal。

## 12. 当前状态与下一执行切片

- canonical 总账：P0=2 Open；P1=66 Open/0 Partial/0 Closed；P2=16 Open/1 Partial/0 Closed；RAR gates=50 Fail。
- 唯一状态变化：RAR-P2-002 `Open -> Partial`，证据是 Arc/COW registry staging 与未完成 managed release gate。
- production/tests/Cargo/ABI：本轮未修改、未运行；review-only。
- tooling：按用户要求排除，后续 Rust 迁移另立 owner，不混入 M64。
- 第一实现切片必须是 M64.1 exact type admission，先把 wrong payload 从“成功后产生三重事实”改成“preflight typed rejection + authority 零变化”。
- M64.1 未通过 testing stage 前，不应开始 M64.4 async 包装或 M64.8 renderer 迁移；否则上层只会建立在错误 authority 之上。

本篇是 current-source refresh，不是 accepted milestone 记录。后续实现产出只写入 Runtime64 对应 child plan 的 `## 状态与产出记录`，索引仅保留当前摘要和 canonical/current-review 链接。
